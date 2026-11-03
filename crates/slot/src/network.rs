//! One-page Wi-Fi and account settings, sharing an encrypted credential vault.
use crate::network_vault::{self, Job, Method, Operation, Response, Secrets, Worker};
use crate::wifi::{self, Wifi};
use serde::{Deserialize, Serialize};
use slot_input::{Action, Btn};
use slot_ui::{KeyInput, KeyOutcome, Keyboard, TexId, UndoFace};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use zeroize::{Zeroize, Zeroizing};

#[derive(Default, Serialize, Deserialize)]
struct Preferences {
    #[serde(default)] achievements: bool,
    #[serde(default)] wifi_enabled: bool,
    #[serde(default)] remember: Option<bool>,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Page { Wifi, Achievements, Security }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Field { Ssid, WifiPassword, Username, AccountPassword, CreateVault, ConfirmVault, Unlock }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Storage { Session, Encrypted }
impl Storage {
    fn label(self) -> &'static str { match self {Self::Session=>"Session only",Self::Encrypted=>"Encrypted"} }
    fn next(self, _: i32) -> Self { match self {Self::Session=>Self::Encrypted,Self::Encrypted=>Self::Session} }
}
#[derive(Clone, Copy)]
enum AfterUnlock { Reconnect, EnableAchievements, Storage(Storage), ChangePin }

pub struct Menu {
    root: PathBuf,
    pub visible: bool,
    pub face: Option<TexId>,
    pub progress_face: Option<TexId>,
    page: Page,
    selected: usize,
    revision: u64,
    keyboard: Option<(Field, Keyboard)>,
    code: Option<Field>,
    code_text: Zeroizing<String>,
    pending_ssid: Option<String>,
    secrets: Secrets,
    first_pass: Zeroizing<String>,
    creating_method: Method,
    method: Option<Method>,
    prefs: Preferences,
    wifi: Wifi,
    worker: Worker,
    encrypted: bool,
    locked: bool,
    pending_jobs: usize,
    after_unlock: Option<AfterUnlock>,
    forget_confirmation: Option<bool>,
    forget_pin: bool,
    pending_storage: Option<bool>,
    startup_pending: bool,
    return_to_shelf: bool,
    notice: String,
    wifi_on: bool,
    last_wifi: wifi::WifiView,
    pub service: Option<slot_achievements::Service>,
    toast: Option<(String, Instant)>,
}
impl Menu {
    pub fn new(root: PathBuf) -> Self {
        let mut prefs: Preferences = std::fs::read(root.join("Config/network.json")).ok()
            .and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default();
        let encrypted = network_vault::vault_path(&root).exists();
        let method = network_vault::method(&root);
        if prefs.remember.is_none() { prefs.remember = Some(encrypted); }
        if !encrypted { prefs.remember=Some(false); }
        Self {
            wifi: Wifi::for_card(&root), worker: Worker::new(&root), root,
            visible: false, face: None, progress_face: None, page: Page::Wifi,
            selected: 0, revision: 0, keyboard: None, code: None,
            code_text: Zeroizing::new(String::new()), pending_ssid: None,
            secrets: Secrets::default(), first_pass: Zeroizing::new(String::new()),
            creating_method: Method::Buttons, method, prefs, encrypted, locked: encrypted,
            pending_jobs: 0, after_unlock: None, forget_confirmation: None,
            forget_pin: false, pending_storage: None,
            startup_pending: true, return_to_shelf: false, notice: String::new(),
            wifi_on: false, last_wifi: Default::default(), service: None, toast: None,
        }
    }
    pub fn open(&mut self) { self.open_wifi(); }
    pub fn open_wifi(&mut self) {
        self.visible = true; self.page = Page::Wifi; self.selected = 0; self.changed();
    }
    pub fn open_achievements(&mut self) {
        self.visible = true; self.page = Page::Achievements; self.selected = 0; self.changed();
    }
    pub fn open_security(&mut self) {
        self.visible = true; self.page = Page::Security; self.selected = 0;
        self.creating_method = self.method.unwrap_or(Method::Buttons); self.changed();
    }
    fn remembers(&self) -> bool { self.prefs.remember == Some(true) }
    /// Called once the startup UI is ready. Never initiate a connection before unlock.
    pub fn startup(&mut self) {
        if !self.startup_pending { return; }
        self.startup_pending = false;
        if self.encrypted && self.remembers() && (self.prefs.wifi_enabled || self.prefs.achievements) {
            if self.prefs.wifi_enabled { self.open_wifi(); } else { self.open_achievements(); }
            self.return_to_shelf = true;
            let after = self.prefs.wifi_enabled.then_some(AfterUnlock::Reconnect);
            self.unlock(after);
        } else if self.prefs.wifi_enabled {
            self.wifi_on = true; self.wifi.turn_on();
        }
    }
    pub fn show_progress(&self) -> bool {
        self.visible && (self.pending_jobs > 0 || self.last_wifi.busy != wifi::Busy::Idle)
    }
    pub fn repeats(&self, btn: Btn) -> bool {
        self.code.is_none() && (matches!(btn, Btn::Up | Btn::Down | Btn::Left | Btn::Right) || self.keyboard.is_some())
    }
    pub fn switches(&self) -> Vec<slot_ui::MenuSwitch> {
        if self.is_keyboard() || self.forget_confirmation.is_some() { return vec![]; }
        self.rows().iter().enumerate()
            .filter(|(_, (_, value, _))| matches!(value.as_str(), "On" | "Off"))
            .filter(|(i, _)| *i >= self.focus_info().1 && *i < self.focus_info().1 + 5)
            .map(|(i, (_, value, _))| slot_ui::MenuSwitch {
                id: self.page_key() * 16 + i as u64, on: value == "On",
                y: slot_ui::expressive::ROW_TOP + 6.0 + (i - self.focus_info().1) as f32 * slot_ui::QUICK_PITCH,
            }).collect()
    }
    fn changed(&mut self) { self.face = None; self.revision = self.revision.wrapping_add(1); }
    fn say(&mut self, message: impl Into<String>) { self.notice = message.into(); self.changed(); }
    pub fn notice_text(&self) -> Option<&str> { (!self.notice.is_empty()).then_some(self.notice.as_str()) }
    pub fn page_key(&self) -> u64 {
        if let Some(field) = self.code { return 4030 + field as u64; }
        if let Some((field, _)) = self.keyboard { return 4010 + field as u64; }
        if self.forget_confirmation.is_some() { return 4003; }
        match self.page { Page::Wifi => 4000, Page::Achievements => 4001, Page::Security => 4002 }
    }
    pub fn focus_info(&self) -> (usize, usize) {
        let count = self.rows().len(); let selected = if let Some(yes) = self.forget_confirmation { usize::from(yes) } else { self.selected.min(count.saturating_sub(1)) };
        (selected, selected.saturating_sub(count.min(5) / 2).min(count.saturating_sub(5)))
    }
    /// Modal editors suppress list focus and scrolling; they use the shared card spring.
    pub fn is_keyboard(&self) -> bool { self.keyboard.is_some() || self.code.is_some() }
    pub fn take_toast(&self) -> Option<&str> {
        self.toast.as_ref().filter(|(_, at)| at.elapsed() < Duration::from_secs(4)).map(|(s, _)| s.as_str())
    }
    fn store_preferences(&mut self) {
        if serde_json::to_vec(&self.prefs).ok().and_then(|b| slot_store::atomic_write(&self.root.join("Config/network.json"), &b).ok()).is_none() {
            self.say("Could not save network settings");
        }
    }
    fn submit(&mut self, job: Job) {
        if self.worker.jobs.send(job).is_ok() { self.pending_jobs += 1; }
        else { self.after_unlock = None; self.say("Credential storage is unavailable"); }
        self.changed();
    }
    fn save_secrets(&mut self) {
        if self.encrypted && self.remembers() && !self.locked && self.pending_storage.is_none() { self.submit(Job::Save(self.secrets.clone())); }
    }
    fn start_tracking(&mut self) {
        self.service = None;
        if self.prefs.achievements && !self.secrets.username.is_empty() {
            self.service = Some(slot_achievements::Service::configured(self.root.clone(), slot_achievements::Config {
                enabled: true, username: self.secrets.username.clone(), password: self.secrets.password.clone(), token: self.secrets.token.clone(),
            }));
        }
    }
    pub fn suspend_for_link(&mut self) {
        // A late unlock response must not reclaim the radio from link play.
        if matches!(self.after_unlock, Some(AfterUnlock::Reconnect)) { self.after_unlock = None; }
        if self.wifi_on { self.wifi_on = false; self.wifi.turn_off(); self.say("Wi-Fi paused for link play"); }
    }
    pub fn update(&mut self) {
        while let Ok(reply) = self.worker.results.try_recv() {
            self.pending_jobs = self.pending_jobs.saturating_sub(1);
            match reply.result {
                Ok(Response::Unlocked(secrets)) => {
                    if self.remembers() { self.secrets = secrets; }
                    self.locked = false; self.start_tracking();
                    self.say("Credentials unlocked");
                    let after = self.after_unlock.take();
                    match after {
                        Some(AfterUnlock::Reconnect) => self.reconnect(),
                        Some(AfterUnlock::EnableAchievements) => self.enable_achievements(),
                        Some(AfterUnlock::Storage(choice)) => self.set_storage(choice),
                        Some(AfterUnlock::ChangePin) => self.prompt(Field::CreateVault),
                        None => {},
                    }
                    if self.return_to_shelf { self.return_to_shelf = false; self.visible = false; }
                }
                Ok(Response::Created(method)) => {
                    self.encrypted = true; self.locked = false; self.method = Some(method);
                    self.say("Security PIN set; shared by Wi-Fi and achievements");
                }
                Ok(Response::Forgotten) => {
                    self.encrypted = false; self.locked = false; self.method = None;
                    self.prefs.remember = Some(false); self.store_preferences();
                    self.say("Security PIN and saved credentials removed");
                }
                Ok(Response::Saved) => {
                    if let Some(remember) = self.pending_storage.take() {
                        self.prefs.remember = Some(remember); self.store_preferences();
                        self.say(if remember {"Credentials saved encrypted"} else {"Session only; Security PIN kept"});
                    } else { self.changed(); }
                },
                Err(message) => {
                    self.pending_storage = None;
                    if reply.operation == Operation::Unlock { self.prompt(Field::Unlock); }
                    self.say(message);
                }
            }
        }
        if self.pending_jobs == 0 { if let Some((username, token)) = self.service.as_ref().and_then(|s| s.take_token()) {
            self.secrets.username = username; self.secrets.token = token; self.secrets.password.zeroize();
            self.save_secrets(); self.say("Signed in to RetroAchievements");
        }}
        let view = self.wifi.view();
        if view != self.last_wifi {
            if view.connected.is_some() && view.ip.is_some() && self.last_wifi.ip.is_none() {
                if let Some(service) = &self.service { service.network_available(); }
                self.say("Wi-Fi connected");
            } else if let Some(error) = &view.error { self.say(error.clone()); }
            if self.wifi_on && !view.on && view.error.is_some() { self.wifi_on = false; }
            self.last_wifi = view;
            self.selected = self.selected.min(self.rows().len().saturating_sub(1));
            self.changed();
        }
        if let Some(service) = &self.service {
            if let Some(notice) = service.take_notice() {
                if notice.kind == slot_achievements::NoticeKind::Earned {
                    self.toast = Some((format!("{} · {}", notice.title, notice.detail), Instant::now()));
                } else if self.visible && !self.is_keyboard() { self.say(notice.title); }
            }
        }
    }
    fn edit(&mut self, field: Field, title: &str, text: &str, secret: bool, min: usize, max: usize) {
        self.code = None; self.code_text.zeroize();
        self.keyboard = Some((field, Keyboard::new(title, text, secret, min, max)));
        self.notice.clear(); self.changed();
    }
    fn prompt(&mut self, field: Field) {
        let method = if field == Field::Unlock { self.method.unwrap_or(Method::Passphrase) } else { self.creating_method };
        if method == Method::Buttons {
            self.keyboard = None; self.code = Some(field); self.code_text.zeroize(); self.notice.clear(); self.changed();
        } else {
            self.edit(field, match field {
                Field::CreateVault => "Create unlock passphrase", Field::ConfirmVault => "Repeat unlock passphrase", _ => "Unlock credentials",
            }, "", true, 12, 128);
        }
    }
    fn unlock(&mut self, after: Option<AfterUnlock>) { self.after_unlock = after; self.prompt(Field::Unlock); }
    fn unlocked(&mut self) -> bool {
        if self.locked { self.unlock(None); false } else { true }
    }
    fn cancel_editor(&mut self) {
        self.keyboard = None; self.code = None; self.code_text.zeroize(); self.first_pass.zeroize();
        self.pending_ssid = None; self.after_unlock = None; self.forget_confirmation = None;
        if self.return_to_shelf { self.return_to_shelf = false; self.visible = false; }
        self.notice.clear(); self.changed();
    }
    fn storage(&self) -> Storage { if self.remembers() {Storage::Encrypted} else {Storage::Session} }
    fn set_storage(&mut self, storage: Storage) {
        if !self.encrypted {
            if storage == Storage::Encrypted { self.say("Set Security PIN in General first"); }
            return;
        }
        if self.locked { self.unlock(Some(AfterUnlock::Storage(storage))); return; }
        if storage == Storage::Session {
            self.forget_pin = false; self.forget_confirmation = Some(false); self.changed();
        } else {
            self.pending_storage = Some(true); self.submit(Job::Save(self.secrets.clone()));
        }
    }
    fn reconnect(&mut self) {
        if self.locked && self.remembers() { self.unlock(Some(AfterUnlock::Reconnect)); return; }
        if !self.prefs.wifi_enabled { self.prefs.wifi_enabled=true; self.store_preferences(); }
        if !self.wifi_on { self.wifi_on = true; self.wifi.turn_on(); }
        if self.secrets.ssid.is_empty() { self.say("Choose a network to connect"); }
        else { self.connect(); }
    }
    fn enable_achievements(&mut self) {
        if self.locked && self.remembers() { self.unlock(Some(AfterUnlock::EnableAchievements)); return; }
        self.prefs.achievements = true; self.store_preferences(); self.start_tracking();
        self.say("Enabled for the next inserted game");
    }
    fn choose_network(&mut self, network: wifi::Network) {
        if network.security == wifi::Security::Unsupported { self.say("This network type is not supported"); return; }
        if self.remembers() && !self.unlocked() { return; }
        let saved_security_matches = match network.security {
            wifi::Security::Open => self.secrets.wifi_password.is_empty(),
            wifi::Security::Personal => !self.secrets.wifi_password.is_empty(),
            wifi::Security::Unsupported => false,
        };
        if self.secrets.ssid == network.ssid && saved_security_matches {
            self.connect(); return;
        }
        match network.security {
            wifi::Security::Open => { self.secrets.ssid = network.ssid; self.secrets.wifi_password.zeroize(); self.connect(); }
            wifi::Security::Personal => { self.pending_ssid = Some(network.ssid); self.edit(Field::WifiPassword, "Wi-Fi password", "", true, 8, 64); }
            wifi::Security::Unsupported => {},
        }
    }
    pub fn input(&mut self, action: Action) {
        if self.pending_jobs > 0 { return; }
        if let Some(field) = self.code {
            match action {
                Action::QuickMenu => self.cancel_editor(),
                Action::GbaDown(Btn::Select) => { self.code_text.pop(); self.changed(); }
                Action::GbaDown(btn) => {
                    let digit = match btn { Btn::A=>Some(0), Btn::B=>Some(1), Btn::X=>Some(2), Btn::Y=>Some(3),
                        Btn::Up=>Some(4), Btn::Down=>Some(5), Btn::Left=>Some(6), Btn::Right=>Some(7),
                        Btn::L1=>Some(8), Btn::R1=>Some(9), Btn::L2=>Some(10), Btn::R2=>Some(11), _=>None };
                    if let Some(digit) = digit {
                        self.code_text.push((b'a'+digit) as char); self.changed();
                        if self.code_text.len() == network_vault::CODE_LEN {
                            let text = std::mem::replace(&mut self.code_text, Zeroizing::new(String::new()));
                            self.code = None; self.accept(field, text);
                        }
                    }
                }
                _ => {},
            }
            return;
        }
        if self.keyboard.is_some() {
            let input = match action {
                Action::QuickMenu => Some(KeyInput::Cancel),
                Action::GbaDown(btn) => match btn {
                    Btn::Up=>Some(KeyInput::Up), Btn::Down=>Some(KeyInput::Down), Btn::Left=>Some(KeyInput::Left), Btn::Right=>Some(KeyInput::Right),
                    Btn::A=>Some(KeyInput::Pick), Btn::B=>Some(KeyInput::Delete), Btn::X=>Some(KeyInput::Shift), Btn::Y=>Some(KeyInput::Space),
                    Btn::L1|Btn::R1=>Some(KeyInput::Page), Btn::Start=>Some(KeyInput::Done), Btn::Select=>Some(KeyInput::Reveal), _=>None,
                }, _=>None,
            };
            if let Some(input) = input {
                let (field, keyboard) = self.keyboard.as_mut().unwrap(); let field = *field;
                match keyboard.press(input) {
                    KeyOutcome::Done(text) => { self.keyboard = None; self.accept(field, Zeroizing::new(text)); }
                    KeyOutcome::Cancelled => self.cancel_editor(), KeyOutcome::Typing => {},
                }
                self.changed();
            }
            return;
        }
        if let Some(yes) = self.forget_confirmation.as_mut() {
            match action {
                Action::QuickMenu | Action::GbaDown(Btn::B) => { self.forget_confirmation = None; }
                Action::GbaDown(Btn::Up | Btn::Down | Btn::Left | Btn::Right) => *yes = !*yes,
                Action::GbaDown(Btn::A) => {
                    let remove = *yes; self.forget_confirmation = None;
                    if remove {
                        if self.forget_pin { self.submit(Job::Forget); }
                        else { self.pending_storage = Some(false); self.submit(Job::Save(Secrets::default())); }
                    }
                }
                _=>{},
            }
            self.changed(); return;
        }
        let Action::GbaDown(btn) = action else {
            if action == Action::QuickMenu { self.visible = false; }
            return;
        };
        if btn == Btn::B { self.visible = false; self.notice.clear(); self.changed(); return; }
        if matches!(btn, Btn::Up | Btn::Down) {
            let count = self.rows().len(); self.selected = (self.selected + if btn==Btn::Down {1} else {count-1}) % count;
            self.notice.clear(); self.changed(); return;
        }
        let storage_row = match self.page {Page::Wifi=>2,Page::Achievements=>5,Page::Security=>usize::MAX};
        if self.selected == storage_row && matches!(btn, Btn::A | Btn::Left | Btn::Right) {
            self.set_storage(self.storage().next(if btn == Btn::Left {-1} else {1})); return;
        }
        if btn != Btn::A && !(self.selected == 0 && matches!(btn, Btn::Left | Btn::Right)) { return; }
        match self.page {
            Page::Security => match self.selected {
                0 => { self.creating_method = match self.creating_method {Method::Buttons=>Method::Passphrase,Method::Passphrase=>Method::Buttons}; self.changed(); }
                1 => { if self.locked { self.unlock(Some(AfterUnlock::ChangePin)); } else { self.first_pass.zeroize(); self.prompt(Field::CreateVault); } }
                2 => { if self.encrypted {self.forget_pin=true;self.forget_confirmation=Some(false);self.changed();} }
                _=>{},
            },
            Page::Wifi => match self.selected {
                0 => {
                    self.prefs.wifi_enabled = !self.wifi_on; self.store_preferences();
                    if self.wifi_on { self.wifi_on = false; self.wifi.turn_off(); self.changed(); }
                    else { self.reconnect(); }
                }
                1 => self.reconnect(),
                3 => { if self.wifi_on { self.wifi.scan(); self.say("Scanning for networks"); } else { self.say("Turn Wi-Fi on first"); } }
                4 => { if self.wifi_on && (!self.remembers() || self.unlocked()) { self.edit(Field::Ssid, "Network name", "", false, 1, 32); } else if !self.wifi_on { self.say("Turn Wi-Fi on first"); } }
                _ => {
                    if !self.wifi_on { self.say("Turn Wi-Fi on first"); return; }
                    if let Some(network) = self.last_wifi.networks.get(self.selected.saturating_sub(5)).cloned() { self.choose_network(network); }
                }
            },
            Page::Achievements => {
                if self.selected == 0 {
                    if self.prefs.achievements { self.prefs.achievements = false; self.store_preferences(); self.start_tracking(); self.say("Achievements off"); }
                    else { self.enable_achievements(); }
                    return;
                }
                if self.remembers() && !self.unlocked() { return; }
                match self.selected {
                    1 => { let name = self.secrets.username.clone(); self.edit(Field::Username, "RetroAchievements username", &name, false, 1, 64); }
                    2 => self.edit(Field::AccountPassword, "RetroAchievements password", "", true, 1, 128),
                    3 => {
                        if !self.prefs.achievements { self.say("Enable achievements first"); }
                        else if self.secrets.username.is_empty() || (self.secrets.password.is_empty() && self.secrets.token.is_empty()) { self.say("Enter your username and password"); }
                        else { if let Some(service) = &self.service { service.network_available(); } else { self.start_tracking(); } self.say("Signing in / syncing"); }
                    }
                    4 => { self.service = None; self.secrets.username.zeroize(); self.secrets.password.zeroize(); self.secrets.token.zeroize(); self.save_secrets(); self.say("Signed out; offline progress kept"); }
                    _ => {},
                }
            }
        }
    }
    fn accept(&mut self, field: Field, text: Zeroizing<String>) {
        match field {
            Field::Ssid => { self.pending_ssid = Some(text.to_string()); self.edit(Field::WifiPassword, "Wi-Fi password (blank for open)", "", true, 0, 64); }
            Field::WifiPassword => {
                if !text.is_empty() && wifi::psk_value(&text).is_err() { self.pending_ssid = None; self.say("Use 8–63 characters or a 64-digit key"); return; }
                if let Some(ssid) = self.pending_ssid.take() { self.secrets.ssid = ssid; }
                self.secrets.wifi_password = text.to_string(); self.connect();
            }
            Field::Username => { self.secrets.username = text.to_string(); self.secrets.password.zeroize(); self.secrets.token.zeroize(); self.service = None; self.save_secrets(); }
            Field::AccountPassword => { self.secrets.password = text.to_string(); self.secrets.token.zeroize(); self.service = None; self.save_secrets(); }
            Field::CreateVault => { self.first_pass = text; self.prompt(Field::ConfirmVault); }
            Field::ConfirmVault => {
                if self.first_pass.as_str() != text.as_str() { self.first_pass.zeroize(); self.prompt(Field::CreateVault); self.say("Codes did not match; choose again"); return; }
                self.first_pass.zeroize(); let saved=if self.remembers() {self.secrets.clone()} else {Secrets::default()};
                self.submit(Job::Create(text, saved, self.creating_method)); self.say("Encrypting credentials");
            }
            Field::Unlock => { self.submit(Job::Unlock(text)); self.say("Unlocking credentials"); }
        }
    }
    fn connect(&mut self) {
        let pass = (!self.secrets.wifi_password.is_empty()).then_some(self.secrets.wifi_password.as_str());
        self.wifi.join(&self.secrets.ssid, pass); self.save_secrets(); self.say("Connecting to Wi-Fi");
    }
    pub fn rows(&self) -> Vec<(String, String, bool)> {
        if let Some(yes) = self.forget_confirmation {
            return vec![("Keep encrypted".into(), String::new(), false), (if self.forget_pin {"Forget Security PIN"} else {"Use session only"}.into(), if yes {"Remove saved copy"} else {""}.into(), false)];
        }
        let storage = self.storage().label().to_string();
        let status = self.service.as_ref().map(|s| {
            if let Some(p) = s.sync_progress() { format!("Preparing {p}%") }
            else if s.sync_pending() { "Unlocks waiting to sync".into() }
            else { match s.sync_status() {
                slot_achievements::SyncStatus::Disabled=>"Disabled", slot_achievements::SyncStatus::Ready=>"Up to date",
                slot_achievements::SyncStatus::Syncing=>"Preparing / syncing", slot_achievements::SyncStatus::Offline=>"Waiting for internet",
                slot_achievements::SyncStatus::Attention=>"Needs attention",
            }.into() }
        }).unwrap_or_else(|| if self.locked {"Unlock to use account"} else if self.prefs.achievements {"Account needed"} else {"Disabled"}.into());
        let mut rows = match self.page {
            Page::Security => vec![
                ("Unlock with".into(),match self.creating_method {Method::Buttons=>"Button code",Method::Passphrase=>"Passphrase"}.into(),true),
                (if self.encrypted {"Change Security PIN"} else {"Set Security PIN"}.into(),if self.locked {"Locked"} else if self.encrypted {"Set"} else {"Not set"}.into(),false),
                ("Forget Security PIN".into(),String::new(),false),
            ],
            Page::Wifi => vec![
                ("Wi-Fi".into(), if self.wifi_on {"On"} else {"Off"}.into(), true),
                ("Reconnect".into(), if self.locked {"Unlock saved network".into()} else {self.secrets.ssid.chars().take(18).collect()}, false),
                ("Credential storage".into(), storage, true),
                ("Scan networks".into(), if self.last_wifi.busy == wifi::Busy::Scanning {"Scanning…".into()} else {self.last_wifi.ip.clone().unwrap_or_default()}, false),
                ("Other network".into(), String::new(), false),
            ],
            Page::Achievements => vec![
                ("Achievements".into(), if self.prefs.achievements {"On"} else {"Off"}.into(), true),
                ("Account".into(), if self.secrets.username.is_empty() {"Not signed in".into()} else {self.secrets.username.chars().take(16).collect()}, false),
                ("Password".into(), if !self.secrets.token.is_empty() {"Signed in"} else if self.secrets.password.is_empty() {"Enter password"} else {"Entered"}.into(), false),
                ("Sign in / sync".into(), status, false), ("Sign out".into(), String::new(), false),
                ("Credential storage".into(), storage, true),
            ],
        };
        if self.page == Page::Wifi { for n in &self.last_wifi.networks {
            rows.push((n.ssid.chars().take(22).collect(), n.encryption.clone(), false));
        }}
        rows
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
        if let Some(field) = self.code {
            let instruction = match field { Field::CreateVault => "Choose 4 button presses", Field::ConfirmVault => "Repeat the same 4 buttons", _ => "Enter your 4-button PIN" };
            return slot_ui::credential_code_face_on(canvas, instruction, self.code_text.len());
        }
        let rows = self.rows();
        let selected = self.focus_info().0;
        let mut face = if self.page == Page::Wifi && self.forget_confirmation.is_none() {
            let signals: Vec<_> = self.last_wifi.networks.iter().map(|n| n.bars()).collect();
            slot_ui::wifi_settings_face_on(canvas, &rows, selected, 5, &signals)
        } else { slot_ui::parent_settings_face_animated_on(canvas, &rows, selected, 0, "", 0.0) };
        let title = slot_ui::quick_heading_face(if self.forget_confirmation.is_some() {if self.forget_pin {"Forget Security PIN?"} else {"Use session only?"}} else { match self.page {Page::Wifi=>"Wi-Fi", Page::Achievements=>"RetroAchievements", Page::Security=>"Security PIN"} });
        blit(&mut face, &title, (canvas.w as u32).saturating_sub(title.w)/2, 0);
        if self.page == Page::Security && self.forget_confirmation.is_none() {
            let lines: &[&str] = match self.creating_method {
                Method::Buttons => &["Four presses for quick unlocking", "Use a passphrase for stronger card protection"],
                Method::Passphrase => &["At least 12 characters", "Stronger protection if the card is copied"],
            };
            let detail = slot_ui::quick_description_face(lines);
            let x=face.w.saturating_sub(detail.w)/2; blit(&mut face, &detail, x, 300);
        }
        if self.forget_confirmation.is_some() {
            let detail = slot_ui::quick_description_face(&["Current session stays signed in", "Next restart needs the credentials again"]);
            let x=face.w.saturating_sub(detail.w)/2; blit(&mut face, &detail, x, 300);
        }
        face
    }
    pub fn flush_ready(&self) -> bool { self.pending_jobs == 0 && self.service.as_ref().is_none_or(|s| s.flush_ready()) }
}
fn blit(dst: &mut UndoFace, src: &UndoFace, x: u32, y: u32) {
    for yy in 0..src.h.min(dst.h.saturating_sub(y)) { for xx in 0..src.w.min(dst.w.saturating_sub(x)) {
        let from=((yy*src.w+xx)*4) as usize; let to=(((y+yy)*dst.w+x+xx)*4) as usize;
        let a=u32::from(src.rgba[from+3]); if a==0 {continue;}
        let behind=u32::from(dst.rgba[to+3])*(255-a)/255; let alpha=a+behind;
        for c in 0..3 {dst.rgba[to+c]=((u32::from(src.rgba[from+c])*a+u32::from(dst.rgba[to+c])*behind)/alpha) as u8;}
        dst.rgba[to+3]=alpha as u8;
    }}
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_has_no_credentials_or_radio_and_one_wifi_page() {
        let root = tempfile::tempdir().unwrap();
        let menu = Menu::new(root.path().into());
        assert!(!menu.prefs.achievements);
        assert!(!menu.wifi_on);
        assert!(menu.service.is_none());
        assert_eq!(menu.rows().len(), 5);
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
            encryption: "WPA2".into(),
            saved: false,
        }];
        menu.selected = 5;
        menu.input(Action::GbaDown(Btn::A));
        assert_eq!(menu.keyboard.as_ref().unwrap().1.text(), "");
        assert_eq!(menu.secrets.ssid, "Home");
        assert_eq!(menu.secrets.wifi_password, "home-secret");
        menu.input(Action::QuickMenu);
        assert!(menu.pending_ssid.is_none());
        assert_eq!(menu.secrets.ssid, "Home");

        menu.last_wifi.networks[0].ssid = "Home".into();
        menu.input(Action::GbaDown(Btn::A));
        assert!(menu.keyboard.is_none());
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
            encryption: "WPA3".into(),
            saved: false,
        }];
        assert_eq!(menu.rows()[5].1, "WPA3");
        menu.selected = 5;
        menu.input(Action::GbaDown(Btn::A));
        assert!(menu.keyboard.is_none());
        assert_eq!(menu.secrets.ssid, "Home");
        assert_eq!(menu.secrets.wifi_password, "home-secret");
    }
    const CODE: [Btn;4] = [Btn::A,Btn::X,Btn::Left,Btn::R1];
    fn buttons(menu: &mut Menu, code: &[Btn]) { for &b in code {menu.input(Action::GbaDown(b));} }
    fn wait(menu: &mut Menu) {
        let until=Instant::now()+Duration::from_secs(10);
        loop {menu.update();if menu.pending_jobs==0{return;}assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(5));}
    }
    fn pin(menu: &mut Menu) {
        menu.open_security();menu.selected=1;menu.input(Action::GbaDown(Btn::A));
        assert_eq!(menu.code,Some(Field::CreateVault));buttons(menu,&CODE);
        assert_eq!(menu.code,Some(Field::ConfirmVault));buttons(menu,&CODE);wait(menu);
        assert!(menu.encrypted && !menu.locked);assert!(!menu.remembers());
    }
    fn saved_wifi(root: &std::path::Path) {
        let mut menu=Menu::new(root.into());pin(&mut menu);
        menu.secrets.ssid="Home-5G".into();menu.secrets.wifi_password="password".into();
        menu.open_wifi();menu.selected=2;menu.input(Action::GbaDown(Btn::Right));wait(&mut menu);
        assert!(menu.remembers());menu.prefs.wifi_enabled=true;menu.store_preferences();
    }
    #[test]
    fn storage_is_inline_and_requires_general_security_pin() {
        let root=tempfile::tempdir().unwrap();let mut menu=Menu::new(root.path().into());menu.open_wifi();
        menu.selected=2;let before=menu.page_key();menu.input(Action::GbaDown(Btn::Right));
        assert_eq!(menu.page_key(),before);assert_eq!(menu.rows()[2].1,"Session only");assert!(menu.rows()[2].2);
        assert_eq!(menu.notice_text(),Some("Set Security PIN in General first"));
        menu.open_achievements();menu.selected=5;menu.input(Action::GbaDown(Btn::Right));
        assert_eq!(menu.notice_text(),Some("Set Security PIN in General first"));
        assert!(!network_vault::vault_path(root.path()).exists());
    }
    #[test]
    fn startup_unlock_retries_and_resumes_saved_connection_without_a_second_press() {
        let root=tempfile::tempdir().unwrap();saved_wifi(root.path());
        let mut menu=Menu::new(root.path().into());menu.startup();
        assert!(menu.visible && menu.locked && !menu.wifi_on);assert_eq!(menu.code,Some(Field::Unlock));
        buttons(&mut menu,&[Btn::A;16]);wait(&mut menu);
        assert!(menu.locked && !menu.wifi_on);assert_eq!(menu.code,Some(Field::Unlock));
        buttons(&mut menu,&CODE);wait(&mut menu);
        assert!(!menu.locked && menu.wifi_on && !menu.visible);
        let until=Instant::now()+Duration::from_secs(5);
        while menu.last_wifi.connected.is_none() {menu.update();assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(5));}
        assert_eq!(menu.last_wifi.connected.as_deref(),Some("Home-5G"));
        // The same unlock covers achievements; there is no second PIN prompt.
        menu.open_achievements();menu.input(Action::GbaDown(Btn::A));
        assert!(menu.prefs.achievements);assert!(menu.code.is_none());
    }
    #[test]
    fn cancelling_startup_or_connection_never_turns_on_the_radio() {
        let root=tempfile::tempdir().unwrap();saved_wifi(root.path());
        let mut menu=Menu::new(root.path().into());menu.startup();menu.input(Action::QuickMenu);
        menu.update();assert!(!menu.visible && !menu.wifi_on && menu.locked);assert!(menu.after_unlock.is_none());
        menu.startup();assert!(!menu.visible,"the cancelled card must not reopen this boot");
        menu.open_wifi();menu.input(Action::GbaDown(Btn::A));assert_eq!(menu.code,Some(Field::Unlock));
        menu.input(Action::QuickMenu);assert!(!menu.wifi_on && menu.after_unlock.is_none());
    }
    #[test]
    fn achievements_enabling_uses_the_same_pin_and_resumes_after_unlock() {
        let root=tempfile::tempdir().unwrap();saved_wifi(root.path());
        let mut menu=Menu::new(root.path().into());menu.open_achievements();menu.input(Action::GbaDown(Btn::A));
        assert!(!menu.prefs.achievements);assert_eq!(menu.code,Some(Field::Unlock));
        buttons(&mut menu,&CODE);wait(&mut menu);assert!(menu.prefs.achievements && !menu.locked);assert!(!menu.wifi_on);
    }
    #[test]
    fn session_storage_erases_saved_secrets_but_keeps_the_shared_pin() {
        let root=tempfile::tempdir().unwrap();saved_wifi(root.path());
        let mut menu=Menu::new(root.path().into());menu.open_wifi();menu.selected=2;menu.input(Action::GbaDown(Btn::Left));
        buttons(&mut menu,&CODE);wait(&mut menu);assert!(menu.forget_confirmation.is_some());
        menu.input(Action::GbaDown(Btn::Down));menu.input(Action::GbaDown(Btn::A));wait(&mut menu);
        assert_eq!(menu.secrets.wifi_password,"password");assert!(!menu.remembers());
        let mut restarted=Menu::new(root.path().into());assert!(restarted.encrypted);assert!(!restarted.remembers());
        restarted.unlock(None);buttons(&mut restarted,&CODE);wait(&mut restarted);
        assert!(restarted.secrets.wifi_password.is_empty());
        let reader=Worker::new(root.path());
        reader.jobs.send(Job::Unlock(Zeroizing::new("acgj".into()))).unwrap();
        let reply=reader.results.recv_timeout(Duration::from_secs(10)).unwrap();
        match reply.result.unwrap() {Response::Unlocked(secrets)=>assert!(secrets.wifi_password.is_empty()),_=>panic!("expected unlocked payload")}

    }
    #[test]
    fn held_buttons_cannot_fill_the_pin_and_scans_use_progress() {
        let root=tempfile::tempdir().unwrap();let mut menu=Menu::new(root.path().into());menu.open_security();
        menu.selected=1;menu.input(Action::GbaDown(Btn::A));
        for b in CODE {assert!(!menu.repeats(b));}
        buttons(&mut menu,&CODE[..2]);assert_eq!(menu.code_text.len(),2);
        menu.input(Action::GbaDown(Btn::Select));assert_eq!(menu.code_text.len(),1);
        menu.input(Action::QuickMenu);assert!(menu.code_text.is_empty()&&menu.first_pass.is_empty());
        menu.open_wifi();assert!(!menu.show_progress());menu.last_wifi.busy=wifi::Busy::Scanning;assert!(menu.show_progress());
        menu.last_wifi.busy=wifi::Busy::Idle;assert!(!menu.show_progress());
    }

    #[test]
    fn a_startup_pin_captures_all_raw_buttons_including_shelf_shortcuts() {
        use crate::audio::AudioSink;
        use slot_input::{Context,RawEvent};
        let root=tempfile::tempdir().unwrap();saved_wifi(root.path());
        let mut sink=crate::audio::StubSink::new();sink.open(48000).unwrap();
        let mut state=slot_store::SlotState::default();state.clock_set=true;
        slot_store::write_slot_state(root.path(),&state).unwrap();
        let mut session=crate::session::Session::boot_with(root.path().into(),Box::new(sink));
        assert!(matches!(session.app().phase(),crate::app::Phase::Shelf));
        session.app_mut().network.as_mut().unwrap().startup();
        assert_eq!(session.app().input_context(),Context::Menu);
        assert!(session.app().literal_menu_buttons());
        for (i,b) in CODE.into_iter().enumerate() {
            session.feed([RawEvent::Down(b),RawEvent::Up(b)],i as u64*40);
        }
        let menu=session.app_mut().network.as_mut().unwrap();
        assert!(menu.code.is_none(),"all four physical presses must submit the code");
        wait(menu);assert!(!menu.locked&&menu.wifi_on);
    }

}
