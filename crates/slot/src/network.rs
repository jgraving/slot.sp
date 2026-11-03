//! Settings-owned network service. All secrets originate in RAM or the encrypted vault.
use crate::network_vault::{self, Job, Secrets, Worker};
use crate::wifi::{self, Wifi};
use serde::{Deserialize, Serialize};
use slot_input::{Action, Btn};
use slot_ui::{KeyInput, KeyOutcome, Keyboard, TexId, UndoFace};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use zeroize::{Zeroize, Zeroizing};

#[derive(Default, Serialize, Deserialize)]
struct Preferences {
    #[serde(default)]
    achievements: bool,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Home,
    Wifi,
    Achievements,
    Storage,
}
#[derive(Clone, Copy)]
enum Field {
    Ssid,
    WifiPassword,
    Username,
    AccountPassword,
    CreateVault,
    ConfirmVault,
    Unlock,
}

pub struct Menu {
    root: PathBuf,
    pub visible: bool,
    pub face: Option<TexId>,
    page: Page,
    entry_page: Page,
    selected: usize,
    first: usize,
    revision: u64,
    keyboard: Option<(Field, Keyboard)>,
    pending_ssid: Option<String>,
    secrets: Secrets,
    first_pass: Zeroizing<String>,
    prefs: Preferences,
    wifi: Wifi,
    worker: Worker,
    encrypted: bool,
    locked: bool,
    saving: bool,
    pending_create: bool,
    pending_forget: bool,
    notice: String,
    wifi_on: bool,
    last_wifi: wifi::WifiView,
    pub service: Option<slot_achievements::Service>,
    toast: Option<(String, Instant)>,
}
impl Menu {
    pub fn new(root: PathBuf) -> Self {
        let prefs = std::fs::read(root.join("Config/network.json"))
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        let encrypted = network_vault::vault_path(&root).exists();
        Self {
            wifi: Wifi::for_card(&root),
            worker: Worker::new(&root),
            root,
            visible: false,
            face: None,
            page: Page::Home,
            entry_page: Page::Home,
            selected: 0,
            first: 0,
            revision: 0,
            keyboard: None,
            pending_ssid: None,
            secrets: Secrets::default(),
            first_pass: Zeroizing::new(String::new()),
            prefs,
            encrypted,
            locked: encrypted,
            saving: false,
            pending_create: false,
            pending_forget: false,
            notice: String::new(),
            wifi_on: false,
            last_wifi: Default::default(),
            service: None,
            toast: None,
        }
    }
    pub fn open_wifi(&mut self) {
        self.open();
        self.page = Page::Wifi;
        self.entry_page = Page::Wifi;
    }
    pub fn open_achievements(&mut self) {
        self.open();
        self.page = Page::Achievements;
        self.entry_page = Page::Achievements;
    }
    pub fn switches(&self) -> Vec<slot_ui::MenuSwitch> {
        if self.keyboard.is_some() {
            return vec![];
        }
        self.rows()
            .iter()
            .enumerate()
            .filter(|(_, (_, value, _))| matches!(value.as_str(), "On" | "Off"))
            .filter(|(i, _)| *i >= self.focus_info().1 && *i < self.focus_info().1 + 5)
            .map(|(i, (_, value, _))| slot_ui::MenuSwitch {
                id: self.page_key() * 16 + i as u64,
                on: value == "On",
                y: slot_ui::expressive::ROW_TOP
                    + 6.0
                    + (i - self.focus_info().1) as f32 * slot_ui::QUICK_PITCH,
            })
            .collect()
    }
    pub fn open(&mut self) {
        self.visible = true;
        self.page = Page::Home;
        self.selected = 0;
        self.first = 0;
        self.changed();
    }
    fn changed(&mut self) {
        self.face = None;
        self.revision = self.revision.wrapping_add(1);
    }
    fn say(&mut self, message: impl Into<String>) {
        self.notice = message.into();
        self.changed();
    }
    pub fn notice_text(&self) -> Option<&str> {
        (!self.notice.is_empty()).then_some(self.notice.as_str())
    }
    pub fn page_key(&self) -> u64 {
        4000 + self.page as u64 + if self.keyboard.is_some() { 16 } else { 0 }
    }
    pub fn focus_info(&self) -> (usize, usize) {
        let n = self.rows().len();
        let visible = n.min(5);
        (
            self.selected,
            self.selected.saturating_sub(visible / 2).min(n - visible),
        )
    }
    pub fn is_keyboard(&self) -> bool {
        self.keyboard.is_some()
    }
    pub fn take_toast(&self) -> Option<&str> {
        self.toast
            .as_ref()
            .filter(|(_, at)| at.elapsed() < Duration::from_secs(4))
            .map(|(s, _)| s.as_str())
    }
    fn store_preferences(&mut self) {
        let result = serde_json::to_vec(&self.prefs).ok().and_then(|b| {
            slot_store::atomic_write(&self.root.join("Config/network.json"), &b).ok()
        });
        if result.is_none() {
            self.say("Could not save network settings");
        }
    }
    fn save_secrets(&mut self) {
        if self.encrypted && !self.locked {
            self.saving = self
                .worker
                .jobs
                .send(Job::Save(self.secrets.clone()))
                .is_ok();
            if !self.saving {
                self.say("Could not save credentials");
            }
        }
    }
    fn start_tracking(&mut self) {
        self.service = None;
        if self.prefs.achievements && !self.secrets.username.is_empty() {
            self.service = Some(slot_achievements::Service::configured(
                self.root.clone(),
                slot_achievements::Config {
                    enabled: true,
                    username: self.secrets.username.clone(),
                    password: self.secrets.password.clone(),
                    token: self.secrets.token.clone(),
                },
            ));
        }
    }
    pub fn suspend_for_link(&mut self) {
        if self.wifi_on {
            self.wifi_on = false;
            self.wifi.turn_off();
            self.say("Wi-Fi paused for link play");
        }
    }
    pub fn update(&mut self) {
        while let Ok(result) = self.worker.results.try_recv() {
            self.saving = false;
            match result {
                Ok(Some(secrets)) => {
                    self.secrets = secrets;
                    self.locked = false;
                    self.start_tracking();
                    self.say("Credentials unlocked");
                }
                Ok(None) => {
                    if self.pending_create {
                        self.pending_create = false;
                        self.encrypted = true;
                        self.locked = false;
                        self.say("Credentials saved encrypted");
                    } else if self.pending_forget {
                        self.pending_forget = false;
                        self.encrypted = false;
                        self.locked = false;
                        self.say("Saved credentials removed; session only");
                    } else {
                        self.changed();
                    }
                }
                Err(message) => {
                    self.pending_create = false;
                    self.pending_forget = false;
                    self.say(message);
                }
            }
        }
        if let Some((username, token)) = self.service.as_ref().and_then(|s| s.take_token()) {
            self.secrets.username = username;
            self.secrets.token = token;
            self.secrets.password.zeroize();
            self.save_secrets();
            self.say("Signed in to RetroAchievements");
        }
        let view = self.wifi.view();
        if view != self.last_wifi {
            let connected = view.connected.is_some() && view.ip.is_some();
            if connected && self.last_wifi.ip.is_none() {
                if let Some(s) = &self.service {
                    s.network_available();
                }
                self.say("Wi-Fi connected");
            } else if let Some(error) = &view.error {
                self.say(error.clone());
            }
            self.last_wifi = view;
            self.changed();
        }
        if let Some(service) = &self.service {
            if let Some(notice) = service.take_notice() {
                if notice.kind == slot_achievements::NoticeKind::Earned {
                    self.toast = Some((
                        format!("{} · {}", notice.title, notice.detail),
                        Instant::now(),
                    ));
                } else if self.visible {
                    self.say(notice.title);
                }
            }
        }
    }
    fn edit(
        &mut self,
        field: Field,
        title: &str,
        text: &str,
        secret: bool,
        min: usize,
        max: usize,
    ) {
        self.keyboard = Some((field, Keyboard::new(title, text, secret, min, max)));
        self.notice.clear();
        self.changed();
    }
    fn choose_page(&mut self, page: Page) {
        self.page = page;
        self.selected = 0;
        self.first = 0;
        self.notice.clear();
        if page == Page::Storage {
            self.notice = "Encrypted storage needs a passphrase after restart".into();
        }
        self.changed();
    }
    fn unlocked(&mut self) -> bool {
        if self.locked {
            self.edit(Field::Unlock, "Unlock credentials", "", true, 12, 128);
            false
        } else {
            true
        }
    }
    pub fn input(&mut self, action: Action) {
        if self.saving {
            return;
        }
        if self.keyboard.is_some() {
            let input = match action {
                Action::QuickMenu => Some(KeyInput::Cancel),
                Action::GbaDown(btn) => match btn {
                    Btn::Up => Some(KeyInput::Up),
                    Btn::Down => Some(KeyInput::Down),
                    Btn::Left => Some(KeyInput::Left),
                    Btn::Right => Some(KeyInput::Right),
                    Btn::A => Some(KeyInput::Pick),
                    Btn::B => Some(KeyInput::Delete),
                    Btn::X => Some(KeyInput::Shift),
                    Btn::Y => Some(KeyInput::Space),
                    Btn::L1 | Btn::R1 => Some(KeyInput::Page),
                    Btn::Start => Some(KeyInput::Done),
                    Btn::Select => Some(KeyInput::Reveal),
                    _ => None,
                },
                _ => None,
            };
            if let Some(input) = input {
                let (field, keyboard) = self.keyboard.as_mut().unwrap();
                let field = *field;
                match keyboard.press(input) {
                    KeyOutcome::Done(text) => {
                        self.keyboard = None;
                        self.accept(field, Zeroizing::new(text));
                    }
                    KeyOutcome::Cancelled => {
                        self.keyboard = None;
                        self.pending_ssid = None;
                        self.first_pass.zeroize();
                    }
                    KeyOutcome::Typing => {}
                }
                self.changed();
            }
            return;
        }
        let Action::GbaDown(btn) = action else {
            if action == Action::QuickMenu {
                self.visible = false;
            }
            return;
        };
        if btn == Btn::B {
            if self.page == Page::Storage {
                self.choose_page(self.entry_page);
            } else {
                self.visible = false;
            }
            self.notice.clear();
            self.changed();
            return;
        }
        let count = self.rows().len();
        if matches!(btn, Btn::Up | Btn::Down) {
            self.selected = (self.selected + if btn == Btn::Down { 1 } else { count - 1 }) % count;
            if self.selected < self.first {
                self.first = self.selected;
            } else if self.selected >= self.first + 5 {
                self.first = self.selected - 4;
            }
            self.notice.clear();
            self.changed();
            return;
        }
        if btn != Btn::A
            && !(matches!(btn, Btn::Left | Btn::Right)
                && self.selected == 0
                && matches!(self.page, Page::Wifi | Page::Achievements))
        {
            return;
        }
        match self.page {
            Page::Home => match self.selected {
                0 => self.choose_page(Page::Wifi),
                1 => self.choose_page(Page::Achievements),
                _ => self.choose_page(Page::Storage),
            },
            Page::Storage => match self.selected {
                0 => {
                    if self.encrypted {
                        self.say("Choose Forget saved credentials first");
                    } else {
                        self.say("Credentials disappear when powered off");
                    }
                }
                1 => {
                    if self.encrypted {
                        if self.locked {
                            self.unlocked();
                        } else {
                            self.say("Already saved encrypted");
                        }
                    } else {
                        self.edit(
                            Field::CreateVault,
                            "New unlock passphrase (12+ characters)",
                            "",
                            true,
                            12,
                            128,
                        );
                    }
                }
                _ => {
                    if self.encrypted {
                        self.pending_forget = true;
                        self.saving = self.worker.jobs.send(Job::Forget).is_ok();
                        self.say("Removing saved credentials");
                    }
                }
            },
            Page::Wifi => {
                if self.selected == 3 {
                    self.choose_page(Page::Storage);
                    return;
                }
                if !self.unlocked() {
                    return;
                }
                match self.selected {
                    0 => {
                        self.wifi_on = !self.wifi_on;
                        if self.wifi_on {
                            self.wifi.turn_on();
                        } else {
                            self.wifi.turn_off();
                        }
                        self.changed();
                    }
                    1 => {
                        if self.wifi_on {
                            self.wifi.scan();
                            self.say("Scanning for networks");
                        } else {
                            self.say("Turn Wi-Fi on first");
                        }
                    }
                    3 => self.choose_page(Page::Storage),
                    2 => {
                        if self.wifi_on {
                            self.edit(Field::Ssid, "Network name", "", false, 1, 32);
                        } else {
                            self.say("Turn Wi-Fi on first");
                        }
                    }
                    _ => {
                        if !self.wifi_on {
                            self.say("Turn Wi-Fi on first");
                            return;
                        }
                        let network = self.last_wifi.networks.get(self.selected - 4).cloned();
                        if let Some(network) = network {
                            match network.security {
                                wifi::Security::Unsupported => {
                                    self.say("This network type is not supported")
                                }
                                wifi::Security::Open => {
                                    self.secrets.ssid = network.ssid;
                                    self.secrets.wifi_password.zeroize();
                                    self.connect();
                                }
                                wifi::Security::Personal => {
                                    let pass = Zeroizing::new(if self.secrets.ssid == network.ssid {
                                        self.secrets.wifi_password.clone()
                                    } else {
                                        String::new()
                                    });
                                    self.pending_ssid = Some(network.ssid);
                                    self.edit(
                                        Field::WifiPassword,
                                        "Wi-Fi password",
                                        &pass,
                                        true,
                                        8,
                                        64,
                                    );
                                }
                            }
                        }
                    }
                }
            }
            Page::Achievements => {
                if self.selected == 5 {
                    self.choose_page(Page::Storage);
                    return;
                }
                if !self.unlocked() {
                    return;
                }
                match self.selected {
                    0 => {
                        self.prefs.achievements = !self.prefs.achievements;
                        self.store_preferences();
                        self.start_tracking();
                        self.say(if self.prefs.achievements {
                            "Enabled for the next inserted game"
                        } else {
                            "Achievements off"
                        });
                    }
                    1 => {
                        let name = self.secrets.username.clone();
                        self.edit(
                            Field::Username,
                            "RetroAchievements username",
                            &name,
                            false,
                            1,
                            64,
                        );
                    }
                    2 => self.edit(
                        Field::AccountPassword,
                        "RetroAchievements password",
                        "",
                        true,
                        1,
                        128,
                    ),
                    3 => {
                        if !self.prefs.achievements {
                            self.say("Enable achievements first");
                        } else if self.secrets.username.is_empty()
                            || (self.secrets.password.is_empty() && self.secrets.token.is_empty())
                        {
                            self.say("Enter your username and password");
                        } else {
                            if let Some(service) = &self.service {
                                service.network_available();
                            } else {
                                self.start_tracking();
                            }
                            self.say("Signing in / syncing");
                        }
                    }
                    5 => self.choose_page(Page::Storage),
                    _ => {
                        self.service = None;
                        self.secrets.username.zeroize();
                        self.secrets.password.zeroize();
                        self.secrets.token.zeroize();
                        self.save_secrets();
                        self.say("Signed out; offline progress kept");
                    }
                }
            }
        }
    }
    fn accept(&mut self, field: Field, text: Zeroizing<String>) {
        match field {
            Field::Ssid => {
                self.pending_ssid = Some(text.to_string());
                self.edit(
                    Field::WifiPassword,
                    "Wi-Fi password (blank for open)",
                    "",
                    true,
                    0,
                    64,
                );
            }
            Field::WifiPassword => {
                if !text.is_empty() && wifi::psk_value(&text).is_err() {
                    self.pending_ssid = None;
                    self.say("Use 8–63 characters or a 64-digit key");
                    return;
                }
                if let Some(ssid) = self.pending_ssid.take() {
                    self.secrets.ssid = ssid;
                }
                self.secrets.wifi_password = text.to_string();
                self.connect();
            }
            Field::Username => {
                self.secrets.username = text.to_string();
                self.secrets.token.zeroize();
                self.service = None;
                self.save_secrets();
            }
            Field::AccountPassword => {
                self.secrets.password = text.to_string();
                self.secrets.token.zeroize();
                self.service = None;
                self.save_secrets();
            }
            Field::CreateVault => {
                self.first_pass = text;
                self.edit(
                    Field::ConfirmVault,
                    "Repeat unlock passphrase",
                    "",
                    true,
                    12,
                    128,
                );
            }
            Field::ConfirmVault => {
                if self.first_pass.as_str() != text.as_str() {
                    self.first_pass.zeroize();
                    self.say("Passphrases did not match");
                    return;
                }
                self.first_pass.zeroize();
                self.pending_create = true;
                self.saving = self
                    .worker
                    .jobs
                    .send(Job::Create(text, self.secrets.clone()))
                    .is_ok();
                self.say("Encrypting credentials");
            }
            Field::Unlock => {
                self.saving = self.worker.jobs.send(Job::Unlock(text)).is_ok();
                self.say("Unlocking credentials");
            }
        }
    }
    fn connect(&mut self) {
        let pass =
            (!self.secrets.wifi_password.is_empty()).then_some(self.secrets.wifi_password.as_str());
        self.wifi.join(&self.secrets.ssid, pass);
        self.save_secrets();
        self.say("Connecting to Wi-Fi");
    }
    pub fn rows(&self) -> Vec<(String, String, bool)> {
        let storage = if self.encrypted {
            if self.locked {
                "Locked"
            } else {
                "Encrypted"
            }
        } else {
            "This session only"
        };
        let service_status = self
            .service
            .as_ref()
            .map(|s| match s.sync_status() {
                slot_achievements::SyncStatus::Disabled => "Disabled",
                slot_achievements::SyncStatus::Ready => "Up to date",
                slot_achievements::SyncStatus::Syncing => "Preparing / syncing",
                slot_achievements::SyncStatus::Offline => "Waiting for internet",
                slot_achievements::SyncStatus::Attention => "Needs attention",
            })
            .unwrap_or(if self.prefs.achievements {
                "Account needed"
            } else {
                "Disabled"
            });
        let service_status =
            if let Some(progress) = self.service.as_ref().and_then(|s| s.sync_progress()) {
                format!("Preparing {progress}%")
            } else if self.service.as_ref().is_some_and(|s| s.sync_pending()) {
                "Unlocks waiting to sync".into()
            } else {
                service_status.to_string()
            };
        let rows: Vec<(&str, String)> = match self.page {
            Page::Home => vec![
                (
                    "Wi-Fi",
                    self.last_wifi.connected.clone().unwrap_or("Off".into()),
                ),
                ("RetroAchievements", service_status.into()),
                ("Credential storage", storage.into()),
            ],
            Page::Storage => vec![
                (
                    "This session only",
                    if !self.encrypted { "Selected" } else { "" }.into(),
                ),
                (
                    "Save encrypted",
                    if self.encrypted {
                        storage
                    } else {
                        "Unlock after restart"
                    }
                    .into(),
                ),
                ("Forget saved credentials", String::new()),
            ],
            Page::Wifi => {
                let mut rows = vec![
                    ("Wi-Fi", if self.wifi_on { "On" } else { "Off" }.into()),
                    (
                        "Scan networks",
                        self.last_wifi.ip.clone().unwrap_or_default(),
                    ),
                    ("Other network", String::new()),
                    ("Credential storage", storage.into()),
                ];
                for network in &self.last_wifi.networks {
                    rows.push((
                        &network.ssid,
                        match network.security {
                            wifi::Security::Open => "Open",
                            wifi::Security::Personal => "Password",
                            wifi::Security::Unsupported => "Unsupported",
                        }
                        .into(),
                    ));
                }
                rows
            }
            Page::Achievements => vec![
                (
                    "Achievements",
                    if self.prefs.achievements { "On" } else { "Off" }.into(),
                ),
                (
                    "Account",
                    if self.secrets.username.is_empty() {
                        "Not signed in".into()
                    } else {
                        self.secrets.username.chars().take(16).collect()
                    },
                ),
                (
                    "Password",
                    if self.secrets.password.is_empty() {
                        "Enter password"
                    } else {
                        "Entered"
                    }
                    .into(),
                ),
                ("Sign in / sync", service_status.into()),
                ("Sign out", String::new()),
                ("Credential storage", storage.into()),
            ],
        };
        rows.into_iter()
            .map(|(name, value)| {
                let toggle = matches!(value.as_str(), "On" | "Off");
                (name.chars().take(22).collect(), value, toggle)
            })
            .collect()
    }
    pub fn screen_face_on(&self, canvas: slot_ui::Canvas, _light: bool) -> UndoFace {
        if let Some((_, keyboard)) = &self.keyboard {
            let src = keyboard.face();
            let mut face = UndoFace {
                rgba: vec![0; (canvas.w as u32 * 480 * 4) as usize],
                w: canvas.w as u32,
                h: 480,
            };
            let x = (face.w - src.w) / 2;
            // Body fits within the normal card; footer remains outside its clipping area.
            for (from_y, to_y, height) in [(40, 100, 252), (360, 380, 40)] {
                for yy in 0..height {
                    for xx in 0..src.w {
                        let from = (((from_y + yy) * src.w + xx) * 4) as usize;
                        let to = (((to_y + yy) * face.w + x + xx) * 4) as usize;
                        face.rgba[to..to + 4].copy_from_slice(&src.rgba[from..from + 4]);
                    }
                }
            }
            let title = slot_ui::quick_heading_face(keyboard.title());
            let tx = face.w.saturating_sub(title.w) / 2;
            for y in 0..title.h.min(face.h) {
                for xx in 0..title.w.min(face.w - tx) {
                    let from = ((y * title.w + xx) * 4) as usize;
                    let to = ((y * face.w + tx + xx) * 4) as usize;
                    if title.rgba[from + 3] > 0 {
                        face.rgba[to..to + 4].copy_from_slice(&title.rgba[from..from + 4]);
                    }
                }
            }
            return face;
        }
        let rows = self.rows();
        let mut face =
            slot_ui::parent_settings_face_animated_on(canvas, &rows, self.selected, 0, "", 0.0);
        let title = slot_ui::quick_heading_face(match self.page {
            Page::Home => "Network",
            Page::Wifi => "Wi-Fi",
            Page::Achievements => "RetroAchievements",
            Page::Storage => "Credential storage",
        });
        let x = face.w.saturating_sub(title.w) / 2;
        for y in 0..title.h.min(face.h) {
            for xx in 0..title.w.min(face.w - x) {
                let from = ((y * title.w + xx) * 4) as usize;
                let to = ((y * face.w + x + xx) * 4) as usize;
                if title.rgba[from + 3] > 0 {
                    face.rgba[to..to + 4].copy_from_slice(&title.rgba[from..from + 4]);
                }
            }
        }
        face
    }
    pub fn flush_ready(&self) -> bool {
        !self.saving && self.service.as_ref().is_none_or(|s| s.flush_ready())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_has_no_credentials_or_radio_and_three_network_rows() {
        let root = tempfile::tempdir().unwrap();
        let menu = Menu::new(root.path().into());
        assert!(!menu.prefs.achievements);
        assert!(!menu.wifi_on);
        assert!(menu.service.is_none());
        assert_eq!(menu.rows().len(), 3);
        assert!(!root.path().join("Config/retroachievements.toml").exists());
        assert!(!network_vault::vault_path(root.path()).exists());
    }

    #[test]
    fn choosing_another_network_never_prefills_or_reassigns_the_saved_password() {
        let root = tempfile::tempdir().unwrap();
        let mut menu = Menu::new(root.path().into());
        menu.open_wifi();
        menu.wifi_on = true;
        menu.secrets.ssid = "Home".into();
        menu.secrets.wifi_password = "home-secret".into();
        menu.last_wifi.networks = vec![wifi::Network {
            ssid: "Other".into(),
            signal: -50,
            security: wifi::Security::Personal,
            saved: false,
        }];
        menu.selected = 4;
        menu.input(Action::GbaDown(Btn::A));
        assert_eq!(menu.keyboard.as_ref().unwrap().1.text(), "");
        assert_eq!(menu.secrets.ssid, "Home");
        assert_eq!(menu.secrets.wifi_password, "home-secret");
        menu.input(Action::QuickMenu);
        assert!(menu.pending_ssid.is_none());
        assert_eq!(menu.secrets.ssid, "Home");

        menu.last_wifi.networks[0].ssid = "Home".into();
        menu.input(Action::GbaDown(Btn::A));
        assert_eq!(menu.keyboard.as_ref().unwrap().1.text(), "home-secret");
        menu.input(Action::QuickMenu);

        menu.last_wifi.networks[0].ssid = "Other".into();
        menu.input(Action::GbaDown(Btn::A));
        menu.accept(Field::WifiPassword, Zeroizing::new("other-secret".into()));
        assert_eq!(menu.secrets.ssid, "Other");
        assert_eq!(menu.secrets.wifi_password, "other-secret");
        assert!(menu.pending_ssid.is_none());
    }

    #[test]
    fn hidden_and_unsupported_networks_preserve_credentials_until_confirmed() {
        let root = tempfile::tempdir().unwrap();
        let mut menu = Menu::new(root.path().into());
        menu.secrets.ssid = "Home".into();
        menu.secrets.wifi_password = "home-secret".into();
        menu.accept(Field::Ssid, Zeroizing::new("Hidden".into()));
        assert_eq!(menu.secrets.ssid, "Home");
        assert_eq!(menu.keyboard.as_ref().unwrap().1.text(), "");
        menu.input(Action::QuickMenu);
        assert!(menu.pending_ssid.is_none());

        menu.open_wifi();
        menu.wifi_on = true;
        menu.last_wifi.networks = vec![wifi::Network {
            ssid: "Unsupported".into(),
            signal: -50,
            security: wifi::Security::Unsupported,
            saved: false,
        }];
        assert_eq!(menu.rows()[4].1, "Unsupported");
        menu.selected = 4;
        menu.input(Action::GbaDown(Btn::A));
        assert!(menu.keyboard.is_none());
        assert_eq!(menu.secrets.ssid, "Home");
        assert_eq!(menu.secrets.wifi_password, "home-secret");
    }
}
