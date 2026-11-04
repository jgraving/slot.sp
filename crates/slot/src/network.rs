//! One-page Wi-Fi and account settings, sharing an encrypted credential vault.
use crate::network_vault::{self, Job, Method, Operation, Response, Secrets, Worker};
use crate::wifi::{self, Wifi};
use crate::update::{self,UpdateJob,UpdateEvent,UpdateJobs,UpdateFail,Kind};
use serde::{Deserialize, Serialize};
use slot_input::{Action, Btn};
use slot_ui::{KeyInput, KeyOutcome, Keyboard, TexId, UndoFace};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use zeroize::{Zeroize, Zeroizing};

#[derive(Serialize, Deserialize)]
struct Preferences {
    #[serde(default)] achievements: bool,
    #[serde(default)] sharing_enabled:bool,
    #[serde(default)] sharing_keep:bool,
    #[serde(default)] wifi_enabled: bool,
    #[serde(default)] lock_device: bool,
    #[serde(default="default_time")] automatic_time: bool,
}
fn default_time()->bool {false}
impl Default for Preferences {fn default()->Self {Self{sharing_enabled:false,sharing_keep:false,achievements:false,wifi_enabled:false,lock_device:false,automatic_time:false}}}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Page { Wifi, Achievements, Security, Updates, Sharing }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Field { Ssid, WifiPassword, Username, AccountPassword, CreateVault, ConfirmVault, Unlock }
#[derive(Clone, Copy)]
enum AfterUnlock { Reconnect, EnableAchievements, ChangePin }

pub struct Menu {
    root: PathBuf,
    sharing:crate::sharing::Sharing,
    sharing_allowed:bool,
    sharing_reveal:bool,
    sharing_library:Vec<slot_store::Cart>,
    shared_snapshot:Option<Vec<slot_store::Cart>>,
    updates: Box<dyn UpdateJobs>,
    releases: Vec<update::Release>,
    update_busy: bool,
    update_installing: Option<update::Release>,
    update_bytes: u64,
    baseos_message:String,
    update_message: String,
    update_staged: bool,
    restart_requested: bool,
    battery: Option<slot_power::Battery>,
    pub visible: bool,
    pub face: Option<TexId>,
    pub progress_face: Option<TexId>,
    page: Page,
    selected: usize,
    revision: u64,
    keyboard: Option<(Field, Keyboard)>,
    code: Option<Field>,
    code_text: Zeroizing<String>,
    button_attempt: Option<Zeroizing<String>>,
    button_pending: Vec<usize>,
    pending_ssid: Option<String>,
    secrets: Secrets,
    first_pass: Zeroizing<String>,
    creating_method: Method,
    method: Option<Method>,
    prefs: Preferences,
    wifi: Wifi,
    diagnostics: Option<crate::wifi_diagnostics::Export>,
    worker: Worker,
    encrypted: bool,
    locked: bool,
    pending_jobs: usize,
    after_unlock: Option<AfterUnlock>,
    forget_confirmation: Option<bool>,
    migrating_plain: bool,
    migration_saving: bool,
    upgrade_pin: bool,
    short_pin: bool,
    notice_tone: slot_ui::NoticeTone,
    toast_tone: slot_ui::NoticeTone,
    startup_pending: bool,
    return_to_shelf: bool,
    parent_unlock: bool,
    parent_authenticated: bool,
    parent_cancelled: bool,
    pin_rejected: Option<Instant>,
    notice: String,
    wifi_on: bool,
    last_wifi: wifi::WifiView,
    last_sync: slot_achievements::SyncDetail,
    pub service: Option<slot_achievements::Service>,
    toast: Option<(String, Instant)>,
}
impl Menu {
    pub fn new(root: PathBuf) -> Self {
        let mut prefs: Preferences = std::fs::read(root.join("Config/network.json")).ok()
            .and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default();
        if !prefs.sharing_keep {prefs.sharing_enabled=false;}
        let encrypted = network_vault::vault_path(&root).exists();
        let method = network_vault::method(&root);
        // Old plaintext is read only after a PIN is created or authenticated.
        let migrating_plain = has_plain_residue(&root);
        let secrets = Secrets::default();
        Self {
            sharing:crate::sharing::Sharing::new(root.clone()),sharing_allowed:true,sharing_reveal:false,sharing_library:Vec::new(),shared_snapshot:None,
            updates:update::update_jobs(root.clone()),releases:Vec::new(),update_busy:false,update_installing:None,
            update_bytes:0,baseos_message:"Not checked".into(),update_message:String::new(),update_staged:false,restart_requested:false,battery:None,
            diagnostics: None,
            wifi: Wifi::for_card(&root), worker: Worker::new(&root), root,
            visible: false, face: None, progress_face: None, page: Page::Wifi,
            selected: 0, revision: 0, keyboard: None, code: None,
            code_text: Zeroizing::new(String::new()), button_attempt:None, button_pending:Vec::new(), pending_ssid: None,
            secrets, first_pass: Zeroizing::new(String::new()),
            creating_method: Method::Buttons, method, prefs, encrypted, locked: encrypted,
            pending_jobs: 0, after_unlock: None, forget_confirmation: None,
            migrating_plain, migration_saving:false, upgrade_pin:false, short_pin:false,
            notice_tone:slot_ui::NoticeTone::Neutral,toast_tone:slot_ui::NoticeTone::Neutral,
            startup_pending: true, return_to_shelf: false,parent_unlock:false,parent_authenticated:false,parent_cancelled:false,pin_rejected:None, notice: String::new(),
            wifi_on: false, last_wifi: Default::default(), last_sync: Default::default(), service: None, toast: None,
        }
    }
    pub fn requires_startup_lock(&self)->bool {self.encrypted && self.prefs.lock_device}
    pub fn unlock_for_parent(&mut self) {
        self.open_security();self.parent_unlock=true;self.parent_authenticated=false;self.parent_cancelled=false;self.return_to_shelf=false;self.unlock(None);
    }
    pub fn take_parent_authenticated(&mut self)->bool {std::mem::take(&mut self.parent_authenticated)}
    pub fn take_parent_cancelled(&mut self)->bool {std::mem::take(&mut self.parent_cancelled)}
    pub fn pin_hint_key(&self)->u64 {u64::from(self.code.is_some_and(|f|f!=Field::Unlock) && self.code_text.len()>=network_vault::MIN_CODE_LEN)}
    fn setup_pin(&mut self) {self.open_security();self.selected=1;self.say("Set a code to encrypt credentials");}
    pub fn open(&mut self) { self.open_wifi(); }
    pub fn open_wifi(&mut self) {
        self.visible = true; self.page = Page::Wifi; self.selected = 0; self.sharing_reveal=false; self.notice.clear(); self.changed();
    }
    pub fn open_achievements(&mut self) {
        self.visible = true; self.page = Page::Achievements; self.selected = 0; self.notice.clear(); self.changed();
    }
    pub fn open_security(&mut self) {
        self.visible = true; self.page = Page::Security; self.selected = 0; self.notice.clear();
        self.creating_method = self.method.unwrap_or(Method::Buttons); self.changed();
    }
    fn credentials_ready(&self) -> bool {self.encrypted && !self.locked && !self.upgrade_pin && !self.migrating_plain}
    fn migrate_plain(&mut self) {
        if !self.migrating_plain || self.migration_saving || self.locked || self.upgrade_pin {return;}
        let Some(secrets)=migration_secrets(&self.root,&self.secrets) else {self.warn("Could not read old credentials",slot_ui::NoticeTone::Danger);return;};
        self.secrets=secrets;self.migration_saving=true;
        self.submit(Job::Save(self.secrets.clone()));self.say("Encrypting saved credentials");
    }
    /// Called once the startup UI is ready. Never initiate a connection before unlock.
    pub fn startup(&mut self) {
        if !self.startup_pending { return; }
        self.startup_pending = false;
        if !self.encrypted {
            self.prefs.wifi_enabled=false;self.prefs.achievements=false;self.store_preferences();return;
        }
        if self.prefs.lock_device || self.prefs.wifi_enabled || self.prefs.achievements || self.prefs.sharing_enabled || self.migrating_plain {
            if self.prefs.wifi_enabled {self.open_wifi();}else{self.open_achievements();}
            self.return_to_shelf=true;
            self.unlock(if self.prefs.wifi_enabled {Some(AfterUnlock::Reconnect)}else{None});
        }
    }
    pub fn wifi_connected(&self) -> bool {
        self.wifi_on && self.last_wifi.on && self.last_wifi.connected.is_some() && self.last_wifi.ip.is_some()
    }
    pub fn achievements_busy(&self) -> bool {
        self.service.as_ref().is_some_and(|s| s.sync_pending() || s.sync_status() == slot_achievements::SyncStatus::Syncing || self.last_sync.progress().is_some() || self.last_sync.pending_awards>0)
    }
    pub fn automatic_time_ready(&self)->bool {self.prefs.automatic_time && self.wifi_connected()}
    pub fn automatic_time_enabled(&self)->bool {self.prefs.automatic_time}
    pub fn toggle_automatic_time(&mut self) {
        if !self.prefs.automatic_time && !self.wifi_connected() {self.require_wifi("automatic time");return;}
        self.prefs.automatic_time=!self.prefs.automatic_time;self.store_preferences();self.changed();
    }
    pub fn require_wifi(&mut self,feature:&str) {
        self.open_wifi();
        if self.last_wifi.busy == wifi::Busy::Addressing {self.say("Getting an IP address…");}
        else if let Some(error)=self.last_wifi.error.clone() {self.warn(&error,slot_ui::NoticeTone::Warning);}
        else {self.say(format!("Wi-Fi needed for {feature}"));}
    }
    fn save_diagnostics(&mut self) {
        if self.diagnostics.is_some() {self.say("Saving Wi-Fi report…");return;}
        self.diagnostics=Some(crate::wifi_diagnostics::start(self.root.clone(),self.last_wifi.clone(),self.sharing.view.clone(),self.last_sync));
        self.say("Saving Wi-Fi report…");
    }
    fn take_diagnostics(&mut self) {
        use std::sync::mpsc::TryRecvError;
        let result=match self.diagnostics.as_ref().map(|rx|rx.try_recv()) {
            Some(Ok(result))=>result,
            Some(Err(TryRecvError::Disconnected))=>Err("Could not save Wi-Fi report"),
            _=>return,
        };
        self.diagnostics=None;
        match result {Ok(path)=>self.say(format!("Saved {path}")),Err(message)=>self.warn(message,slot_ui::NoticeTone::Warning)}
        self.changed();
    }
    pub fn set_battery(&mut self,battery:Option<slot_power::Battery>) {self.battery=battery;}
    pub fn take_update_restart(&mut self)->bool {std::mem::take(&mut self.restart_requested)}
    pub fn open_updates(&mut self) {if !self.wifi_connected() {self.require_wifi("updates");return;}self.visible=true;self.page=Page::Updates;self.selected=0;self.notice.clear();self.changed();}
    fn check_updates(&mut self) {
        if !self.wifi_connected() {self.say("Connect to Wi-Fi first");return;}
        if self.update_busy {return;}
        self.update_busy=true;self.update_message="Checking…".into();self.updates.ask(UpdateJob::Check);self.changed();
    }
    fn install_update(&mut self,index:usize) {
        if self.update_busy || self.update_staged {return;}
        if !self.wifi_connected() {self.say("Connect to Wi-Fi first");return;}
        let Some(release)=self.releases.get(index).cloned() else {return;};
        if release.kind==Kind::BaseOs && !self.battery.is_some_and(|b|b.percent>=20||matches!(b.charge,slot_power::Charge::Charging|slot_power::Charge::Full)) {
            self.say("Plug in before updating BaseOS");return;
        }
        self.update_bytes=0;self.update_busy=true;self.update_message="Downloading…".into();
        self.update_installing=Some(release.clone());self.updates.ask(UpdateJob::Install(release));self.changed();
    }
    fn take_update_events(&mut self) {
        while let Some(event)=self.updates.take() {
            match event {
                UpdateEvent::BaseOsStatus(message)=>{self.baseos_message=message;self.changed();continue;},
                UpdateEvent::Progress(bytes)=> {self.update_bytes=bytes;self.face=None;continue;}
                UpdateEvent::Available(releases)=> {self.releases=releases;self.update_message="Available".into();}
                UpdateEvent::UpToDate=> {self.releases.clear();self.update_message="Up to date".into();}
                UpdateEvent::Staged=> {self.update_staged=true;self.update_message="Restart to install".into();self.say("Update ready · restart to install");}
                UpdateEvent::Cancelled=> {self.update_message="Cancelled".into();}
                UpdateEvent::Failed(error)=> {self.update_message=match error {
                    UpdateFail::Clock=>"Set the clock, then retry".into(),UpdateFail::Checksum=>"Checksum mismatch".into(),
                    UpdateFail::Space=>"Not enough space".into(),UpdateFail::Rejected(why)=>why,
                    UpdateFail::Bad=>"Incompatible update".into(),_=>"Download failed · retry".into(),
                };let message=self.update_message.clone();self.say(message);}
            }
            self.update_busy=false;self.update_installing=None;self.changed();
        }
    }
    pub fn show_progress(&self) -> bool {
        self.visible && ((self.pending_jobs > 0 && self.button_attempt.is_none()) || self.last_wifi.busy != wifi::Busy::Idle || self.diagnostics.is_some() || self.update_busy || self.sharing.view.busy)
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
    pub(crate) fn automatic_time_notice(&mut self) {
        self.toast_tone=slot_ui::NoticeTone::Neutral;
        self.toast=Some(("Automatic time is on".into(),Instant::now()));
    }
    pub(crate) fn say(&mut self, message: impl Into<String>) { self.notice_tone=slot_ui::NoticeTone::Neutral;self.notice = message.into(); self.changed(); }
    fn warn(&mut self,message:&str,tone:slot_ui::NoticeTone) {self.say(message);self.notice_tone=tone;}
    pub fn notice_tone(&self)->slot_ui::NoticeTone {
        if self.visible {self.notice_tone}else{self.toast_tone}
    }
    pub fn notice_text(&self) -> Option<&str> {(!self.notice.is_empty()).then_some(self.notice.as_str())}
    pub fn page_key(&self) -> u64 {
        if let Some(field) = self.code { return 4030 + field as u64; }
        if let Some((field, _)) = self.keyboard { return 4010 + field as u64; }
        if self.forget_confirmation.is_some() { return 4003; }
        match self.page { Page::Wifi => 4000, Page::Achievements => 4001, Page::Security => 4002, Page::Updates => 4004, Page::Sharing=>4005 }
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
    fn resume_unlocked(&mut self) {
        if self.credentials_ready() && self.service.is_none() {self.start_tracking();}
        self.notice.clear(); self.changed();
        if self.parent_unlock {self.parent_unlock=false;self.parent_authenticated=true;self.visible=false;return;}
        match self.after_unlock.take() {
            Some(AfterUnlock::Reconnect) => self.reconnect(),
            Some(AfterUnlock::EnableAchievements) => self.enable_achievements(),
            Some(AfterUnlock::ChangePin) => self.prompt(Field::CreateVault),
            None => {},
        }
        if self.return_to_shelf {self.return_to_shelf=false;self.visible=false;}
    }
    fn save_secrets(&mut self) {
        if self.credentials_ready() {self.submit(Job::Save(self.secrets.clone()));}
    }
    fn start_tracking(&mut self) {
        self.service = None; self.last_sync = Default::default();
        if self.credentials_ready() && self.prefs.achievements && !self.secrets.username.is_empty() {
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
        self.take_update_events();
        if let Some(since)=self.pin_rejected {self.face=None;if since.elapsed().as_secs_f32()>=0.36 {self.pin_rejected=None;}}

        while let Ok(reply) = self.worker.results.try_recv() {
            self.pending_jobs = self.pending_jobs.saturating_sub(1);
            if reply.operation==Operation::Unlock {
                if let Some(attempt)=self.button_attempt.take() {
                    // Cancelling/deleting entry invalidates a late result. Wrong prefixes
                    // remain silent and never clear the typed sequence or disclose length.
                    if self.code!=Some(Field::Unlock) || !self.code_text.starts_with(attempt.as_str()) {continue;}
                    if reply.result.is_err() {continue;}
                    self.short_pin=attempt.len()<8;
                    self.upgrade_pin=attempt.len()<network_vault::MIN_CODE_LEN;
                    self.code=None;self.code_text.zeroize();self.button_pending.clear();
                }
            }
            match reply.result {
                Ok(Response::Unlocked(secrets)) => {
                    self.secrets=secrets;self.locked=false;
                    if self.upgrade_pin && !self.parent_unlock {
                        self.prompt(Field::CreateVault);self.say("Choose a code with 6+ presses");
                    } else if self.migrating_plain {self.migrate_plain();}
                    else {self.resume_unlocked();}
                }
                Ok(Response::Created(method)) => {
                    self.encrypted=true;self.locked=false;self.method=Some(method);self.upgrade_pin=false;

                    if self.migrating_plain {
                        // Create wrote the imported secrets successfully before deletion.
                        if remove_plain(&self.root).is_ok() {self.migrating_plain=false;}
                        else {self.warn("Could not delete unencrypted copy",slot_ui::NoticeTone::Danger);}
                    }
                    if self.credentials_ready() {
                        self.resume_unlocked();
                        if self.short_pin {self.warn("Short code: easy to guess",slot_ui::NoticeTone::Warning);}
                        else {self.say("Device code saved");}
                    }
                }
                Ok(Response::Forgotten) => {
                    self.encrypted=false;self.locked=false;self.method=None;self.upgrade_pin=false;
                    self.prefs.lock_device=false;self.prefs.wifi_enabled=false;self.prefs.achievements=false;self.prefs.sharing_enabled=false;self.sharing.request(None);
                    self.wifi_on=false;self.wifi.turn_off();self.service=None;self.secrets=Secrets::default();self.store_preferences();
                    self.say("Code removed · network disabled");
                }
                Ok(Response::Saved) => {
                    if self.migration_saving {
                        self.migration_saving=false;
                        if remove_plain(&self.root).is_err() {self.warn("Could not delete unencrypted copy",slot_ui::NoticeTone::Danger);}
                        else {self.migrating_plain=false;self.resume_unlocked();self.say("Old credentials encrypted");}
                    } else {self.changed();}
                }
                Err(message) => {
                    let leaving_plain=std::mem::take(&mut self.migration_saving);
                    if reply.operation == Operation::Unlock { self.prompt(Field::Unlock); }
                    if leaving_plain {self.warn(message,slot_ui::NoticeTone::Danger);}else{self.say(message);}
                }
            }
        }
        if let Some(detail) = self.service.as_ref().and_then(|s| s.sync_detail()) {
            if detail != self.last_sync { self.last_sync = detail; self.face = None; }
        }
        self.try_button_unlock();
        if self.pending_jobs == 0 { if let Some((username, token)) = self.service.as_ref().and_then(|s| s.take_token()) {
            self.secrets.username = username; self.secrets.token = token; self.secrets.password.zeroize();
            self.save_secrets(); self.say("Signed in to RetroAchievements");
        }}
        self.apply_wifi_view(self.wifi.view());
        self.take_diagnostics();
        self.step_sharing();
        if let Some(service) = &self.service {
            if let Some(notice) = service.take_notice() {
                if notice.kind == slot_achievements::NoticeKind::Earned {
                    self.toast = Some((format!("{} · {}", notice.title, notice.detail), Instant::now()));
                } else if self.visible && !self.is_keyboard() { self.say(notice.title); }
            }
        }
    }
    fn apply_wifi_view(&mut self, view: wifi::WifiView) {
        if view != self.last_wifi {
            // Revocation follows the Wi-Fi session, even while the SMB process is paused.
            // An initial disconnected boot view must still allow the explicit reboot opt-in.
            let disconnected = self.last_wifi.on && self.last_wifi.connected.is_some() && self.last_wifi.ip.is_some()
                && (!view.on || view.connected.is_none() || view.ip.is_none());
            if self.prefs.sharing_enabled && disconnected {
                self.prefs.sharing_enabled=false;self.store_preferences();self.sharing.request(None);
                self.say("Sharing stopped · disconnected");
            }
            if view.connected.is_some() && view.ip.is_some() && self.last_wifi.ip.is_none() {
                if let Some(service) = &self.service { service.network_available(); }
                self.say("Wi-Fi connected");
            } else if view.busy == wifi::Busy::Addressing && self.last_wifi.busy != wifi::Busy::Addressing {
                self.say("Getting an IP address…");
            } else if view.error != self.last_wifi.error {
                if let Some(error) = &view.error { self.warn(error,slot_ui::NoticeTone::Warning); }
            }
            if self.wifi_on && !view.on && view.error.is_some() { self.wifi_on = false; }
            self.last_wifi = view;
            self.selected = self.selected.min(self.rows().len().saturating_sub(1));
            self.changed();
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
            self.keyboard = None; self.code = Some(field); self.code_text.zeroize(); self.button_pending.clear();self.pin_rejected=None;self.notice.clear(); self.changed();
        } else {
            self.edit(field, match field {
                Field::CreateVault => "Choose device code", Field::ConfirmVault => "Repeat device code", _ => "Enter device code",
            }, "", true, 12, 128);
        }
    }
    fn unlock(&mut self, after: Option<AfterUnlock>) { self.after_unlock = after; self.prompt(Field::Unlock); }
    fn unlocked(&mut self) -> bool {
        if !self.encrypted || self.upgrade_pin {self.setup_pin();return false;}
        if self.locked { self.unlock(None); false } else { true }
    }
    fn cancel_editor(&mut self) {

        if self.parent_unlock {self.parent_unlock=false;self.parent_cancelled=true;self.visible=false;}
        if self.return_to_shelf && self.prefs.lock_device {
            self.code_text.zeroize();self.prompt(Field::Unlock);self.say("Enter your Device code to unlock the device");return;
        }
        self.keyboard = None; self.code = None; self.code_text.zeroize();self.button_pending.clear(); self.first_pass.zeroize();
        self.pending_ssid = None; self.after_unlock = None; self.forget_confirmation = None;
        if self.return_to_shelf {
            self.return_to_shelf = false; self.visible = false;
            self.toast=Some(("Saved credentials are locked".into(),Instant::now()));
        }
        self.notice.clear(); self.changed();
    }
    fn pin_ready(&mut self)->bool {
        if !self.encrypted || self.upgrade_pin {self.setup_pin();false}
        else if self.migrating_plain && !self.locked {self.migrate_plain();false}
        else {true}
    }
    fn reconnect(&mut self) {
        if !self.pin_ready() {return;}
        if self.locked { self.unlock(Some(AfterUnlock::Reconnect)); return; }
        if !self.prefs.wifi_enabled { self.prefs.wifi_enabled=true; self.store_preferences(); }
        if !self.wifi_on { self.wifi_on = true; self.wifi.turn_on(); }
        if self.secrets.ssid.is_empty() { self.say("Choose a network to connect"); }
        else { self.connect(); }
    }
    fn enable_achievements(&mut self) {
        if !self.pin_ready() {return;}
        if self.locked { self.unlock(Some(AfterUnlock::EnableAchievements)); return; }
        self.prefs.achievements = true; self.store_preferences(); self.start_tracking();
        self.say("Enabled for the next inserted game");
    }
    fn choose_network(&mut self, network: wifi::Network) {
        if network.security == wifi::Security::Unsupported { self.say("This network type is not supported"); return; }
        if !self.unlocked() { return; }
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
    fn try_button_unlock(&mut self) {
        if self.code!=Some(Field::Unlock) || self.pending_jobs>0 {return;}
        // Prefer the latest complete input, but retain intermediate prefixes:
        // fast typing must not skip a correct PIN while Argon2 is in flight.
        let Some(length)=self.button_pending.pop() else {return;};
        let text=Zeroizing::new(self.code_text[..length].to_string());
        self.button_attempt=Some(Zeroizing::new(text.to_string()));
        self.submit(Job::Unlock(text));
    }
    pub fn input(&mut self, action: Action) {
        if self.page==Page::Updates && !self.is_keyboard() {
            match action {
                Action::QuickMenu|Action::GbaDown(Btn::B)=> {
                    if self.update_busy {if !self.updates.cancel() {self.say("Finishing update…");}}
                    else {self.visible=false;self.notice.clear();self.changed();}
                }
                Action::GbaDown(Btn::Up|Btn::Down) if !self.update_busy=> {
                    let count=self.rows().len();self.selected=(self.selected+if action==Action::GbaDown(Btn::Down){1}else{count-1})%count;self.changed();
                }
                Action::GbaDown(Btn::A) if !self.update_busy=> {if self.update_staged {self.restart_requested=true;}else if self.selected==0 {self.check_updates();}else {self.install_update(self.selected-1);}}
                _=>{},
            }
            return;
        }
        if let Some(field) = self.code {
            match action {
                Action::QuickMenu => self.cancel_editor(),
                Action::GbaDown(Btn::Select) => {
                    self.code_text.pop();self.button_pending.retain(|&length|length<=self.code_text.len());
                    if field==Field::Unlock && self.code_text.len()>=4 && !self.button_pending.contains(&self.code_text.len()) {self.button_pending.push(self.code_text.len());}
                    self.changed();self.try_button_unlock();
                }
                Action::GbaDown(Btn::Start) if field==Field::Unlock => {if self.code_text.len()<network_vault::MIN_CODE_LEN {self.pin_rejected=Some(Instant::now());self.say("Use 6+ presses");}self.try_button_unlock();}
                Action::GbaDown(Btn::Start) => {
                    if self.code_text.len() < network_vault::MIN_CODE_LEN { self.pin_rejected=Some(Instant::now());self.say("Use 6+ presses"); }
                    else {
                        let text=std::mem::replace(&mut self.code_text,Zeroizing::new(String::new()));
                        self.code=None; self.accept(field,text);
                    }
                }
                Action::GbaDown(btn) => {
                    let digit = match btn { Btn::A=>Some(0), Btn::B=>Some(1), Btn::X=>Some(2), Btn::Y=>Some(3),
                        Btn::Up=>Some(4), Btn::Down=>Some(5), Btn::Left=>Some(6), Btn::Right=>Some(7),
                        Btn::L1=>Some(8), Btn::R1=>Some(9), Btn::L2=>Some(10), Btn::R2=>Some(11), _=>None };
                    if let Some(digit) = digit {
                        self.pin_rejected=None;self.code_text.push((b'a'+digit) as char);
                        if field==Field::Unlock && self.code_text.len()>=4 {self.button_pending.push(self.code_text.len());}
                        self.notice.clear();self.changed();self.try_button_unlock();
                    } else {self.say("PIN_BUTTONS");}
                }
                _ => {},
            }
            return;
        }
        if self.pending_jobs > 0 { return; }
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
                        if remove_plain(&self.root).is_ok() {self.submit(Job::Forget);}
                        else {self.warn("Could not delete unencrypted copy",slot_ui::NoticeTone::Danger);}
                    } else {self.notice.clear();self.notice_tone=slot_ui::NoticeTone::Neutral;}
                }
                _=>{},
            }
            self.changed(); return;
        }
        let Action::GbaDown(btn) = action else {
            if action == Action::QuickMenu { self.visible = false; self.sharing_reveal=false; }
            return;
        };
        if btn == Btn::B && self.page==Page::Sharing {self.page=Page::Wifi;self.selected=4;self.sharing_reveal=false;self.notice.clear();self.changed();return;}
        if btn == Btn::B { self.visible = false; self.notice.clear(); self.changed(); return; }
        if matches!(btn, Btn::Up | Btn::Down) {
            let count = self.rows().len(); self.selected = (self.selected + if btn==Btn::Down {1} else {count-1}) % count;
            self.notice.clear(); self.changed(); return;
        }
        if btn != Btn::A && !((self.selected == 0 || (self.page==Page::Security && self.selected==2) || (self.page==Page::Sharing && self.selected==1)) && matches!(btn, Btn::Left | Btn::Right)) { return; }
        match self.page {
            Page::Security => match self.selected {
                0 => { self.creating_method = match self.creating_method {Method::Buttons=>Method::Passphrase,Method::Passphrase=>Method::Buttons}; self.changed(); }
                1 => { if self.locked { self.unlock(Some(AfterUnlock::ChangePin)); } else { self.first_pass.zeroize(); self.prompt(Field::CreateVault); } }
                2 => {
                    if !self.encrypted {self.say("Set Device code first");}
                    else {self.prefs.lock_device=!self.prefs.lock_device;self.store_preferences();self.changed();}
                }
                3 => { if self.root.join("Config/kids/use-device-pin").exists() {self.say("Set a separate parent code first");}
                    else if self.encrypted {self.forget_confirmation=Some(false);self.changed();} }
                _=>{},
            },
            Page::Wifi => match self.selected {
                0 => {
                    if self.wifi_on {self.prefs.wifi_enabled=false;self.store_preferences();self.wifi_on=false;self.wifi.turn_off();self.changed();}
                    else {self.reconnect();}
                }
                1 => self.reconnect(),
                2 => { if self.wifi_on { self.wifi.scan(); self.say("Scanning for networks"); } else { self.say("Turn Wi-Fi on first"); } }
                3 => { if self.wifi_on && self.unlocked() { self.edit(Field::Ssid, "Network name", "", false, 1, 32); } else if !self.wifi_on { self.say("Turn Wi-Fi on first"); } }
                4 => {self.page=Page::Sharing;self.selected=0;self.sharing_reveal=false;self.notice.clear();self.changed();}
                i if i == 5 + self.last_wifi.networks.len() => self.save_diagnostics(),
                _ => {
                    if !self.wifi_on { self.say("Turn Wi-Fi on first"); return; }
                    if let Some(network) = self.last_wifi.networks.get(self.selected.saturating_sub(5)).cloned() { self.choose_network(network); }
                }
            },
            Page::Sharing=> match self.selected {
                0=>self.toggle_sharing(),
                1=>{self.prefs.sharing_keep=!self.prefs.sharing_keep;self.store_preferences();self.changed();if self.prefs.sharing_keep {self.warn("Keep off on public Wi-Fi",slot_ui::NoticeTone::Warning);}},
                4=>{if self.unlocked(){self.sharing_reveal=!self.sharing_reveal;self.changed();}},
                5=>{if self.unlocked(){match crate::sharing::password(){Ok(password)=>{self.sharing.request(None);self.secrets.sharing_password=password.to_string();self.save_secrets();self.sharing_reveal=true;self.say("Sharing password changed");},Err(error)=>self.say(error)}}},
                _=>{},
            },
            Page::Updates => {},
            Page::Achievements => {
                if self.selected == 0 {
                    if self.prefs.achievements { self.prefs.achievements = false; self.store_preferences(); self.start_tracking(); self.say("Achievements off"); }
                    else { self.enable_achievements(); }
                    return;
                }
                if self.selected>=5 {return;}
                if !self.unlocked() { return; }
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
                if self.first_pass.as_str() != text.as_str() { self.first_pass.zeroize(); self.prompt(Field::CreateVault); self.say("Codes did not match"); return; }
                self.first_pass.zeroize(); let saved=if self.migrating_plain {
                    let Some(saved)=migration_secrets(&self.root,&self.secrets) else {self.warn("Could not read old credentials",slot_ui::NoticeTone::Danger);return;};
                    self.secrets=saved.clone();saved
                } else {self.secrets.clone()};
                self.short_pin=self.creating_method==Method::Buttons && text.len()<8;self.submit(Job::Create(text, saved, self.creating_method)); self.say("Encrypting credentials");
            }
            Field::Unlock => {self.short_pin=self.method==Some(Method::Buttons)&&text.len()<8; self.submit(Job::Unlock(text)); self.say("Unlocking credentials"); }
        }
    }
    fn connect(&mut self) {
        if self.secrets.ssid!=self.secrets.sharing_network {self.prefs.sharing_enabled=false;self.sharing.request(None);}
        if !self.credentials_ready() {return;}
        let pass = (!self.secrets.wifi_password.is_empty()).then_some(self.secrets.wifi_password.as_str());
        self.wifi.join(&self.secrets.ssid, pass); self.save_secrets(); self.say("Connecting to Wi-Fi");
    }
    pub fn rows(&self) -> Vec<(String, String, bool)> {
        if let Some(yes) = self.forget_confirmation {
            return vec![("Keep current storage".into(),String::new(),false),("Forget Device code".into(),if yes {"Confirm with A"}else{""}.into(),false)];
        }
        let status = if self.service.is_some() { sync_label(self.last_sync) }
            else { if self.locked {"Unlock to use account"} else if self.prefs.achievements {"Account needed"} else {"Disabled"}.into() };
        let mut rows = match self.page {
            Page::Security => vec![
                ("Unlock with".into(),match self.creating_method {Method::Buttons=>"Button code",Method::Passphrase=>"Passphrase"}.into(),true),
                (if self.encrypted {"Change Device code"} else {"Set Device code"}.into(),if self.locked {"Locked"} else if self.encrypted {"Set"} else {"Not set"}.into(),false),
                ("Startup access".into(),if self.prefs.lock_device {"Require code"}else{"Code optional"}.into(),true),
                ("Forget Device code".into(),String::new(),false),
            ],
            Page::Wifi => vec![
                ("Wi-Fi".into(), if self.wifi_on {"On"} else {"Off"}.into(), true),
                ("Reconnect".into(), if self.locked {"Unlock saved network".into()} else {self.secrets.ssid.chars().take(18).collect()}, false),
                ("Scan networks".into(), if self.last_wifi.busy == wifi::Busy::Scanning {"Scanning…".into()} else if self.last_wifi.busy == wifi::Busy::Addressing {"Getting IP…".into()} else if self.last_wifi.connected.is_some() && self.last_wifi.ip.is_none() {"No IP address".into()} else {self.last_wifi.ip.clone().unwrap_or_default()}, false),
                ("Other network".into(), String::new(), false),
                ("Network file sharing".into(),String::new(),false),
            ],
            Page::Sharing=>vec![
                ("Sharing".into(),if self.sharing.view.busy {"Working…"}else if self.sharing.view.active {"On"}else{"Off"}.into(),true),
                ("Keep on after reboot".into(),if self.prefs.sharing_keep {"On"}else{"Off"}.into(),true),
                ("Finder".into(),self.last_wifi.ip.as_ref().map(|ip|format!("smb://{ip}/SDCard")).unwrap_or_else(||"Not connected".into()),false),
                ("Username".into(),"slot".into(),false),
                ("Password".into(),if self.locked {"Locked".into()}else if self.sharing_reveal {self.secrets.sharing_password.clone()}else{"Press A to show".into()},false),
                ("New password".into(),String::new(),false),
            ],
            Page::Updates => {
                if self.update_staged {return vec![("Restart to install".into(),String::new(),false)];}
                let status=if self.update_busy {if self.update_installing.is_some() {"Downloading…"}else{""}}
                    else {match self.update_message.as_str() {""=>"","Up to date"=>"Up to date","Available"=>"Available","Cancelled"=>"Cancelled",_=>"Retry"}};
                let mut rows=vec![("Check for updates".into(),status.into(),false)];
                for release in &self.releases {
                    rows.push((if release.kind==Kind::Slot {"slot.sp"}else{"BaseOS"}.into(),release.version.clone(),false));
                }
                rows.push(("Installed slot.sp".into(),crate::build_info::Build::current().version.into(),false));
                rows.push(("Installed BaseOS".into(),crate::baseos::var("BASEOS_VERSION").unwrap_or_else(||"Unknown".into()),false));
                rows.push(("BaseOS check".into(),self.baseos_message.clone(),false));
                rows
            },
            Page::Achievements => vec![
                ("Achievements".into(), if self.prefs.achievements {"On"} else {"Off"}.into(), true),
                ("Account".into(), if self.secrets.username.is_empty() {"Not signed in".into()} else {self.secrets.username.chars().take(16).collect()}, false),
                ("Password".into(), if !self.secrets.token.is_empty() {"Signed in"} else if self.secrets.password.is_empty() {"Enter password"} else {"Entered"}.into(), false),
                ("Sign in / sync".into(), status, false), ("Sign out".into(), String::new(), false),
            ],
        };
        if self.page == Page::Achievements && self.service.is_some() {
            let d = self.last_sync;
            rows.extend([
                (if d.games_total>0 && d.games_done==d.games_total && d.games_failed==0 {"Games ready"}else{"Game data"}.into(), if !d.catalog_ready {"Reading library…".into()}else if d.games_total==0 {"No GBA games".into()}else{format!("{} / {}", d.games_done, d.games_total)}, false),
                ("Artwork".into(), if !d.catalog_ready || !d.artwork_ready {"Reading artwork…".into()}else if d.badges_total==0 {"None queued".into()}else{format!("{} / {}", d.badges_done, d.badges_total)}, false),
                ("Uploads waiting".into(), d.pending_awards.to_string(), false),
                ("Games to retry".into(), d.games_failed.to_string(), false),
            ]);
            if !d.problem.is_empty() {rows.push(("Status".into(),d.problem.into(),false));}
            if d.retry_seconds>0 {rows.push(("Retry in".into(),format!("{}s",d.retry_seconds),false));}
        }
        if self.page == Page::Wifi { for n in &self.last_wifi.networks {
            rows.push((n.ssid.chars().take(22).collect(), n.encryption.clone(), false));
        }
            rows.push(("Save Wi-Fi report".into(),if self.diagnostics.is_some() {"Saving…"}else{"To SD card"}.into(),false));
        }
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
            let instruction = match field { Field::CreateVault => "Choose a code", Field::ConfirmVault => "Repeat your code", _ => "Enter your code" };
            let minimum=network_vault::MIN_CODE_LEN;
            let footer=if field==Field::Unlock {if self.return_to_shelf && self.prefs.lock_device {10}else{1}}else if self.code_text.len()<minimum {12}else{8};
            let u=self.pin_rejected.map_or(0.0,|t|(t.elapsed().as_secs_f32()/0.36).clamp(0.0,1.0));
            let shake=((u*std::f32::consts::TAU*3.0).sin()*10.0*(1.0-u)).round()as i32;
            return slot_ui::masked_pin_state_face_on(canvas,"Device code",instruction,self.code_text.len(),footer,minimum,shake);
        }
        let rows = self.rows();
        let selected = self.focus_info().0;
        let mut face = if self.page == Page::Wifi && self.forget_confirmation.is_none() {
            let signals: Vec<_> = self.last_wifi.networks.iter().map(|n| n.bars()).collect();
            slot_ui::wifi_settings_face_on(canvas, &rows, selected, 5, &signals)
        } else { slot_ui::parent_settings_face_animated_on(canvas, &rows, selected, 0, "", 0.0) };
        let title = slot_ui::quick_heading_face(if self.forget_confirmation.is_some() {"Forget Device code?"} else { match self.page {Page::Wifi=>"Network", Page::Achievements=>"RetroAchievements", Page::Security=>"Device code", Page::Updates=>"Updates", Page::Sharing=>"Network file sharing"} });
        blit(&mut face, &title, (canvas.w as u32).saturating_sub(title.w)/2, 0);

        face
    }
    /// The compositor draws this after the card's clip and footer chrome.
    pub fn floating_progress(&self)->Option<(String,u64,u64)> {
        if !self.visible || self.is_keyboard() || self.forget_confirmation.is_some() {return None;}
        if self.page==Page::Achievements && self.service.is_some() {
            return self.last_sync.progress().map(|(phase,done,total)|(if self.last_sync.retry_seconds>0 {format!("{phase} · retry {}s",self.last_sync.retry_seconds)}else{phase.to_string()},done as u64,total as u64));
        }
        if self.page==Page::Updates {if let Some(r)=&self.update_installing {return Some(("Downloading".into(),self.update_bytes,r.size));}}
        None
    }
    pub fn set_sharing_allowed(&mut self,allowed:bool) {self.sharing_allowed=allowed;if !allowed {self.sharing.request(None);}}
    pub fn take_shared_snapshot(&mut self)->Option<Vec<slot_store::Cart>> {self.shared_snapshot.take()}
    pub fn take_shared_games(&mut self)->Vec<slot_store::Cart> {std::mem::take(&mut self.sharing_library)}
    fn toggle_sharing(&mut self) {
        if self.prefs.sharing_enabled||self.sharing.view.active {self.prefs.sharing_enabled=false;self.store_preferences();self.sharing.request(None);self.say("File sharing stopped");return;}
        if !self.wifi_connected(){self.say("Connect to Wi-Fi first");return;}
        if !self.unlocked(){return;}
        if !self.sharing_allowed{self.say("Return to the carousel first");return;}
        if self.secrets.sharing_password.is_empty(){match crate::sharing::password(){Ok(pass)=>{self.secrets.sharing_password=pass.to_string();self.save_secrets();},Err(error)=>{self.say(error);return;}}}
        self.secrets.sharing_network=self.last_wifi.connected.clone().unwrap_or_default();self.save_secrets();
        self.prefs.sharing_enabled=true;self.store_preferences();self.sharing_reveal=false;
        self.warn("Only share on trusted Wi-Fi",slot_ui::NoticeTone::Warning);
    }
    fn step_sharing(&mut self) {
        for event in self.sharing.poll() {match event {
            crate::sharing::Event::Library(carts)=>self.shared_snapshot=Some(carts),
            crate::sharing::Event::Stopped(import)=>{self.changed();if let Some(import)=import {
                let text=if import.added.is_empty()&&import.wallpapers==0&&import.skipped==0 {"No new uploads".into()}
                    else {format!("{} added · {} left in Inbox",import.added.len()+import.wallpapers,import.skipped)};
                self.sharing_library.extend(import.added);self.say(text);
            }},
            crate::sharing::Event::Failed(error)=>{self.prefs.sharing_enabled=false;self.store_preferences();self.warn(error,slot_ui::NoticeTone::Danger);},
            _=>self.changed(),
        }}
        // Explicit network changes and disconnects revoke this session rather than re-enable elsewhere.
        if self.prefs.sharing_enabled && self.wifi_on && self.last_wifi.connected.as_deref().is_some_and(|s|s!=self.secrets.sharing_network) {
            self.prefs.sharing_enabled=false;self.store_preferences();self.warn("Sharing stopped · network changed",slot_ui::NoticeTone::Warning);
        }
        if self.sharing.view.active && !self.wifi_connected() {self.prefs.sharing_enabled=false;self.store_preferences();self.say("Sharing stopped · disconnected");}
        let start=if self.prefs.sharing_enabled && self.sharing_allowed && self.credentials_ready() && self.wifi_connected()
            && self.last_wifi.connected.as_deref()==Some(self.secrets.sharing_network.as_str()) {
            self.last_wifi.ip.as_ref().and_then(|ip|ip.parse::<std::net::Ipv4Addr>().ok()).filter(|ip|!ip.is_unspecified()&&!ip.is_multicast()&&!ip.is_loopback())
                .map(|ip|crate::sharing::Start{connection:crate::sharing::Connection{ssid:self.secrets.sharing_network.clone(),ip},password:Zeroizing::new(self.secrets.sharing_password.clone())})
        }else{None};
        // Initial enablement and password rotation wait for encrypted storage. Unrelated
        // credential work (such as an RA token refresh) must not interrupt an existing upload.
        let start=start.filter(|s|self.pending_jobs==0 || self.sharing.requested_connection()==Some(&s.connection));
        let old=self.sharing.view.clone();self.sharing.request(start);if old!=self.sharing.view{self.changed();}
    }
    pub fn flush_ready(&self) -> bool { if self.update_busy {self.updates.cancel();} !self.update_busy && self.sharing.ready() && self.pending_jobs == 0 && self.service.as_ref().is_none_or(|s| s.flush_ready()) }

}
fn sync_label(d: slot_achievements::SyncDetail) -> String {
    use slot_achievements::SyncStatus::*;
    match d.status {
        Disabled => "Starting…".into(),
        Offline | Attention => if d.problem=="Check username or password" {"Check account"}else{"Waiting to retry"}.into(),
        Ready => if d.authenticated { "Up to date" } else { "Signing in…" }.into(),
        Syncing => if !d.authenticated {"Signing in…"}else{"Syncing…"}.into(),
    }
}
fn plain_path(root:&std::path::Path)->PathBuf {root.join("Config/network-credentials.json")}
fn plain_temp(name:&str)->bool {
    let suffix=name.strip_prefix(".network-credentials.json.").and_then(|n|n.strip_suffix(".tmp"));
    suffix.is_some_and(|s|{let parts:Vec<_>=s.split('.').collect();parts.len()==2&&parts.iter().all(|p|!p.is_empty()&&p.bytes().all(|b|b.is_ascii_digit()))})
}
fn has_plain_residue(root:&std::path::Path)->bool {
    // The generic boot sweep skips the current PID and ignores removal failures.
    // Credential cleanup must also cover PID reuse and a missing primary file.
    match std::fs::symlink_metadata(plain_path(root)) {
        Ok(_)=>return true,
        Err(e) if e.kind()==std::io::ErrorKind::NotFound=>{},
        Err(_)=>return true,
    }
    match std::fs::read_dir(root.join("Config")) {
        Ok(mut entries)=>entries.any(|entry|match entry {
            Ok(entry)=>plain_temp(&entry.file_name().to_string_lossy()),Err(_)=>true,
        }),
        Err(e)=>e.kind()!=std::io::ErrorKind::NotFound,
    }
}
fn migration_secrets(root:&std::path::Path,current:&Secrets)->Option<Secrets> {
    use std::io::Read;
    let file=match std::fs::File::open(plain_path(root)) {
        Ok(file)=>file,
        // Orphan writes may be incomplete or stale; never import them over the
        // authenticated vault. Save its contents first, then remove the residue.
        Err(e) if e.kind()==std::io::ErrorKind::NotFound=>return Some(current.clone()),
        Err(_)=>return None,
    };
    let mut bytes=Zeroizing::new(Vec::new());
    file.take(32*1024+1).read_to_end(&mut bytes).ok()?;
    if bytes.len()>32*1024 {return None;}
    serde_json::from_slice(&bytes).ok()
}
#[cfg(test)]
fn read_plain(root:&std::path::Path)->Option<Secrets> {
    migration_secrets(root,&Secrets::default())
}
#[cfg(test)]
fn write_plain(root:&std::path::Path,secrets:&Secrets)->std::io::Result<()> {
    let bytes=Zeroizing::new(serde_json::to_vec(secrets)?);
    std::fs::create_dir_all(root.join("Config"))?;
    slot_store::atomic_write(&plain_path(root),&bytes)
}
fn remove_plain(root:&std::path::Path)->std::io::Result<()> {
    // Include interrupted atomic writes of this credential file, not unrelated files.
    let config=root.join("Config");
    let entries=match std::fs::read_dir(&config) {Ok(entries)=>entries,Err(e) if e.kind()==std::io::ErrorKind::NotFound=>return Ok(()),Err(e)=>return Err(e)};
    {
        for entry in entries {
            let entry=entry?;let name=entry.file_name();let name=name.to_string_lossy();
            if plain_temp(&name) {std::fs::remove_file(entry.path())?;}
        }
    }
    match std::fs::remove_file(plain_path(root)) {Err(e) if e.kind()==std::io::ErrorKind::NotFound=>Ok(()),result=>result}
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
    #[cfg(target_os="linux")]
    #[test]
    #[ignore = "requires Mesa EGL; produces production-compositor Wi-Fi review captures"]
    fn native_wifi_progress_review() {
        use slot_gfx::Compositor;
        struct Empty;impl slot_input::InputSource for Empty {fn poll(&mut self,_:u64)->Vec<slot_input::RawEvent>{vec![]}}
        let output=std::path::PathBuf::from(std::env::var("SLOT_WIFI_PREVIEW").unwrap());
        for (light,width) in [(true,640),(false,720)] {
            let dir=output.join(if light {"light"}else{"dark"});std::fs::create_dir_all(&dir).unwrap();
            let root=tempfile::tempdir().unwrap();crate::root::ensure(root.path());
            let mut state=slot_store::SlotState::default();state.clock_set=true;state.appearance=if light {slot_store::Appearance::Light}else{slot_store::Appearance::Dark};slot_store::write_slot_state(root.path(),&state).unwrap();
            let surface=super::review_surface::Egl::new(light);let mut g=Compositor::new(&surface).unwrap();g.fit((width,480));
            let mut f=crate::frontend::Frontend::boot(Box::new(slot_power::SimPlatform::at(root.path().into())));f.upload_faces(&mut g);f.render(&mut g,(width,480));
            let mut tick=0;
            for (name,busy,ip,error) in [
                ("getting-address",wifi::Busy::Addressing,None,None),
                ("address-timeout",wifi::Busy::Idle,None,Some("No IP address · retry")),
                ("late-success",wifi::Busy::Idle,Some("192.0.2.3"),None),
                ("disconnected",wifi::Busy::Idle,None,Some("Wi-Fi disconnected · retry"))] {
                f.app_mut_for_review().network.as_mut().unwrap().open_wifi();
                for _ in 0..90 {
                    tick+=17;f.advance_at(&mut Empty,tick,1.0/60.0);
                    let m=f.app_mut_for_review().network.as_mut().unwrap();m.wifi_on=true;
                    m.apply_wifi_view(wifi::WifiView{on:true,connected:Some("Home".into()),busy:busy.clone(),ip:ip.map(str::to_string),error:error.map(str::to_string),..Default::default()});
                    f.compose(&mut g);
                }
                let mut encoder=png::Encoder::new(std::fs::File::create(dir.join(format!("{name}.png"))).unwrap(),width,480);
                encoder.set_color(png::ColorType::Rgba);encoder.set_depth(png::BitDepth::Eight);encoder.write_header().unwrap().write_image_data(&g.read_frame()).unwrap();
            }
        }
    }

    #[test]
    fn address_progress_timeout_and_late_success_have_distinct_notices() {
        let root=tempfile::tempdir().unwrap();let mut menu=Menu::new(root.path().into());menu.open_wifi();menu.wifi_on=true;
        let mut view=wifi::WifiView{on:true,connected:Some("Home".into()),busy:wifi::Busy::Addressing,..Default::default()};
        menu.apply_wifi_view(view.clone());assert!(menu.show_progress());assert_eq!(menu.notice_text(),Some("Getting an IP address…"));
        menu.open_updates();assert_eq!(menu.notice_text(),Some("Getting an IP address…"));assert_eq!(menu.page,Page::Wifi);
        view.busy=wifi::Busy::Idle;view.error=Some("No IP address · retry".into());menu.apply_wifi_view(view.clone());
        assert!(!menu.show_progress());assert!(menu.notice_text().unwrap().contains("No IP address"));assert!(!menu.wifi_connected());
        view.ip=Some("192.0.2.3".into());view.error=None;menu.apply_wifi_view(view);assert_eq!(menu.notice_text(),Some("Wi-Fi connected"));assert!(menu.wifi_connected());
    }
    #[test]
    fn report_action_works_offline_and_reports_save_result_without_enabling_wifi() {
        let root=tempfile::tempdir().unwrap();let mut menu=Menu::new(root.path().into());menu.open_wifi();
        menu.input(Action::GbaDown(Btn::Up));assert_eq!(menu.rows()[menu.selected].0,"Save Wi-Fi report");
        // Controller selection starts one asynchronous export, including with Wi-Fi off.
        menu.input(Action::GbaDown(Btn::A));assert!(menu.diagnostics.is_some());assert!(menu.show_progress());assert!(!menu.wifi_on);
        let deadline=Instant::now()+Duration::from_secs(15);
        while menu.diagnostics.is_some(){menu.take_diagnostics();assert!(Instant::now()<deadline);std::thread::sleep(Duration::from_millis(10));}
        assert!(menu.notice_text().unwrap().starts_with("Saved Diagnostics/wifi-"));assert!(root.path().join("Diagnostics/wifi-0001.txt").exists());assert!(!menu.show_progress());
        let (tx,rx)=std::sync::mpsc::channel();tx.send(Err("Could not save report · check SD card")).unwrap();menu.diagnostics=Some(rx);menu.take_diagnostics();
        assert_eq!(menu.notice_text(),Some("Could not save report · check SD card"));assert!(!menu.show_progress());
    }

    #[test]
    fn default_has_no_credentials_or_radio_and_one_wifi_page() {
        let root = tempfile::tempdir().unwrap();
        let menu = Menu::new(root.path().into());
        assert!(!menu.prefs.achievements);
        assert!(!menu.wifi_on);
        assert!(menu.service.is_none());
        assert_eq!(menu.rows().len(), 6);
        assert!(!root.path().join("Config/retroachievements.toml").exists());
        assert!(!network_vault::vault_path(root.path()).exists());
    }

    #[test]
    fn choosing_another_network_never_prefills_or_reassigns_the_saved_password() {
        let root = tempfile::tempdir().unwrap();
        let mut menu = Menu::new(root.path().into());pin(&mut menu);
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
        assert!(menu.keyboard.is_none());wait(&mut menu);
        menu.input(Action::QuickMenu);

        menu.last_wifi.networks = vec![wifi::Network {ssid:"Other".into(),signal:-50,security:wifi::Security::Personal,encryption:"WPA2".into(),saved:false}];
        menu.open_wifi();menu.selected = 5;
        menu.input(Action::GbaDown(Btn::A));
        menu.accept(Field::WifiPassword, Zeroizing::new("other-secret".into()));
        assert_eq!(menu.secrets.ssid, "Other");
        assert_eq!(menu.secrets.wifi_password, "other-secret");
        assert!(menu.pending_ssid.is_none());
    }


    #[test]
    fn hidden_and_unsupported_networks_preserve_credentials_until_confirmed() {
        let root = tempfile::tempdir().unwrap();
        let mut menu = Menu::new(root.path().into());pin(&mut menu);
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
    #[test]
    fn sharing_is_password_protected_and_resets_on_reboot_unless_opted_in() {
        let root=tempfile::tempdir().unwrap();let mut menu=Menu::new(root.path().into());pin(&mut menu);
        assert!(!menu.prefs.sharing_enabled&&!menu.prefs.sharing_keep);
        menu.wifi_on=true;menu.last_wifi=wifi::WifiView{on:true,connected:Some("Home".into()),ip:Some("192.168.1.10".into()),..Default::default()};
        menu.toggle_sharing();assert!(menu.prefs.sharing_enabled);assert_eq!(menu.secrets.sharing_password.len(),16);
        wait(&mut menu);
        let password=menu.secrets.sharing_password.clone();let encrypted=std::fs::read(network_vault::vault_path(root.path())).unwrap();
        assert!(!encrypted.windows(password.len()).any(|b|b==password.as_bytes()));
        let reboot=Menu::new(root.path().into());assert!(!reboot.prefs.sharing_enabled);assert!(reboot.locked);assert!(!reboot.sharing.view.active);
        menu.prefs.sharing_keep=true;menu.prefs.sharing_enabled=true;menu.store_preferences();
        let mut reboot=Menu::new(root.path().into());assert!(reboot.prefs.sharing_enabled&&reboot.prefs.sharing_keep);reboot.step_sharing();assert!(!reboot.sharing.view.active,"vault must be unlocked before persistent sharing starts");
        menu.wifi_on=true;menu.last_wifi=wifi::WifiView{on:true,connected:Some("Public".into()),ip:Some("10.0.0.2".into()),..Default::default()};
        menu.step_sharing();assert!(!menu.prefs.sharing_enabled,"another SSID must revoke enablement");
    }
    fn sharing_fixture(root: &std::path::Path) -> Menu {
        let mut menu=Menu::new(root.into());
        menu.encrypted=true;menu.locked=false;menu.wifi_on=true;
        menu.secrets.sharing_password="ABCDEFGH23456789".into();
        menu.secrets.sharing_network="Home".into();menu.prefs.sharing_enabled=true;
        menu
    }
    fn home_wifi() -> wifi::WifiView {
        wifi::WifiView{on:true,connected:Some("Home".into()),ip:Some("192.168.1.10".into()),..Default::default()}
    }
    fn wait_for_sharing(menu: &mut Menu) {
        let until=Instant::now()+Duration::from_secs(10);
        while !menu.sharing.view.active {
            menu.step_sharing();assert!(Instant::now()<until,"sharing did not start");
            std::thread::sleep(Duration::from_millis(5));
        }
    }
    #[test]
    fn credential_jobs_preserve_requested_sharing_but_new_sessions_wait_for_storage() {
        let root=tempfile::tempdir().unwrap();let mut menu=sharing_fixture(root.path());
        menu.apply_wifi_view(home_wifi());menu.pending_jobs=1;menu.step_sharing();
        assert!(menu.sharing.requested_connection().is_none(),"first start must await encrypted storage");
        menu.pending_jobs=0;menu.step_sharing();
        assert!(menu.sharing.requested_connection().is_some());
        // An unrelated credential save can arrive while the server is starting or already active.
        menu.pending_jobs=1;menu.step_sharing();
        assert!(menu.sharing.requested_connection().is_some(),"credential work cancelled server startup");
        wait_for_sharing(&mut menu);menu.step_sharing();
        assert!(menu.sharing.view.active,"credential work interrupted an active upload");
        // Password rotation explicitly closes the old session and must await its new saved password.
        menu.sharing.request(None);menu.secrets.sharing_password="23456789ABCDEFGH".into();
        menu.step_sharing();assert!(menu.sharing.requested_connection().is_none());
        menu.pending_jobs=0;wait_for_sharing(&mut menu);
    }
    #[test]
    fn disconnect_revokes_sharing_during_startup_and_while_paused() {
        for paused in [false,true] {
            let root=tempfile::tempdir().unwrap();let mut menu=sharing_fixture(root.path());
            menu.prefs.sharing_keep=true;menu.apply_wifi_view(home_wifi());menu.step_sharing();
            if paused {wait_for_sharing(&mut menu);menu.set_sharing_allowed(false);}
            menu.apply_wifi_view(wifi::WifiView::default());
            assert!(!menu.prefs.sharing_enabled,"a dropped connection must revoke even a paused session");
            assert!(menu.sharing.requested_connection().is_none());
            let reboot=Menu::new(root.path().into());assert!(!reboot.prefs.sharing_enabled);
            menu.apply_wifi_view(home_wifi());menu.set_sharing_allowed(true);menu.step_sharing();
            assert!(menu.sharing.requested_connection().is_none(),"reconnect must not silently restore sharing");
        }
    }
    #[test]
    fn reboot_opt_in_waits_for_first_connection_but_lost_lease_revokes_it() {
        let root=tempfile::tempdir().unwrap();let mut menu=sharing_fixture(root.path());
        menu.prefs.sharing_keep=true;menu.locked=true;
        menu.apply_wifi_view(wifi::WifiView::default());menu.step_sharing();
        assert!(menu.prefs.sharing_enabled);assert!(menu.sharing.requested_connection().is_none());
        menu.locked=false;menu.apply_wifi_view(home_wifi());wait_for_sharing(&mut menu);
        menu.set_sharing_allowed(false);
        let mut lost_lease=home_wifi();lost_lease.ip=None;menu.apply_wifi_view(lost_lease);
        assert!(!menu.prefs.sharing_enabled,"losing the address revokes a paused session too");
        menu.apply_wifi_view(home_wifi());menu.set_sharing_allowed(true);menu.step_sharing();
        assert!(menu.sharing.requested_connection().is_none());
    }
    #[test]
    fn sharing_page_keeps_password_hidden_until_requested_and_warns_on_persistence() {
        let root=tempfile::tempdir().unwrap();let mut menu=Menu::new(root.path().into());
        menu.open_wifi();menu.selected=4;menu.input(Action::GbaDown(Btn::A));assert_eq!(menu.page,Page::Sharing);
        menu.secrets.sharing_password="ABCDEFGH23456789".into();assert!(!menu.rows()[4].1.contains("ABCDEFGH"));
        menu.selected=1;menu.input(Action::GbaDown(Btn::A));assert!(menu.prefs.sharing_keep);assert_eq!(menu.notice_tone(),slot_ui::NoticeTone::Warning);
        menu.input(Action::GbaDown(Btn::B));assert_eq!(menu.page,Page::Wifi);assert!(!menu.sharing_reveal);
        menu.sharing_reveal=true;menu.input(Action::QuickMenu);menu.open_wifi();menu.selected=4;menu.input(Action::GbaDown(Btn::A));assert!(!menu.sharing_reveal,"reopening must hide the password after Menu closes the page");
    }
    #[test]
    fn direct_card_sharing_has_a_finder_url_and_refreshes_added_and_removed_games() {
        let root=tempfile::tempdir().unwrap();let mut menu=sharing_fixture(root.path());
        menu.apply_wifi_view(home_wifi());wait_for_sharing(&mut menu);
        menu.page=Page::Sharing;
        assert!(menu.rows()[2].1.ends_with("/SDCard"));
        assert!(menu.rows().iter().all(|r|r.0!="Import uploads"));
        std::fs::create_dir_all(root.path().join("Games/GBA")).unwrap();
        std::fs::write(root.path().join("Games/GBA/New.gba"),[0;192]).unwrap();
        menu.toggle_sharing();
        let until=Instant::now()+Duration::from_secs(10);
        while !menu.sharing.ready(){menu.step_sharing();assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(5));}
        assert_eq!(menu.take_shared_snapshot().unwrap().iter().map(|c|c.stem.as_str()).collect::<Vec<_>>(),["New"]);
        menu.prefs.sharing_enabled=true;wait_for_sharing(&mut menu);
        std::fs::remove_file(root.path().join("Games/GBA/New.gba")).unwrap();menu.toggle_sharing();
        while !menu.sharing.ready(){menu.step_sharing();assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(5));}
        assert!(menu.take_shared_snapshot().unwrap().is_empty());
        assert_eq!(menu.notice,"File sharing stopped");
    }
    #[test]
    fn automatic_sharing_stop_does_not_report_an_empty_import() {
        let root=tempfile::tempdir().unwrap();let mut menu=sharing_fixture(root.path());
        menu.apply_wifi_view(home_wifi());wait_for_sharing(&mut menu);
        menu.set_sharing_allowed(false);menu.say("Sharing stopped · disconnected");
        let until=Instant::now()+Duration::from_secs(10);
        while !menu.sharing.ready(){menu.step_sharing();assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(5));}
        assert_eq!(menu.notice,"Sharing stopped · disconnected","automatic teardown must not replace other feedback");
        // Switching off uses direct files and does not run an import.
        menu.set_sharing_allowed(true);wait_for_sharing(&mut menu);menu.toggle_sharing();
        assert_eq!(menu.notice,"File sharing stopped");
        while !menu.sharing.ready(){menu.step_sharing();assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(5));}
        assert_eq!(menu.notice,"File sharing stopped");
    }
    const CODE: [Btn;6] = [Btn::A,Btn::X,Btn::Left,Btn::R1,Btn::Y,Btn::L1];
    fn buttons(menu: &mut Menu, code: &[Btn]) { for &b in code {menu.input(Action::GbaDown(b));} }
    fn submit_code(menu: &mut Menu, code: &[Btn]) { buttons(menu,code);menu.input(Action::GbaDown(Btn::Start)); }
    fn wait(menu: &mut Menu) {
        let until=Instant::now()+Duration::from_secs(10);
        loop {menu.update();if menu.pending_jobs==0{return;}assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(5));}
    }
    fn pin(menu: &mut Menu) {
        menu.open_security();menu.selected=1;menu.input(Action::GbaDown(Btn::A));
        assert_eq!(menu.code,Some(Field::CreateVault));submit_code(menu,&CODE);
        assert_eq!(menu.code,Some(Field::ConfirmVault));submit_code(menu,&CODE);wait(menu);
        assert!(menu.encrypted && !menu.locked);assert!(menu.encrypted);
    }
    fn saved_wifi(root: &std::path::Path) {
        let mut menu=Menu::new(root.into());pin(&mut menu);
        menu.secrets.ssid="Home-5G".into();menu.secrets.wifi_password="password".into();
        menu.open_wifi();menu.save_secrets();wait(&mut menu);
        assert!(menu.encrypted);menu.prefs.wifi_enabled=true;menu.store_preferences();
    }
    #[test]
    fn encrypted_is_default_and_missing_pin_opens_its_settings() {
        let root=tempfile::tempdir().unwrap();let mut menu=Menu::new(root.path().into());menu.open_wifi();
        assert!(!menu.rows().iter().any(|r|r.0=="Credentials"));assert!(!plain_path(root.path()).exists());
        menu.input(Action::GbaDown(Btn::A));assert_eq!(menu.page,Page::Security);
        assert!(!menu.wifi_on);assert!(menu.notice_text().unwrap().contains("code to encrypt credentials"));
        assert_eq!(menu.selected,1);
        menu.open_achievements();menu.input(Action::GbaDown(Btn::A));assert_eq!(menu.page,Page::Security);
        assert!(!menu.prefs.achievements);
    }
    #[test]
    fn startup_unlock_retries_and_resumes_saved_connection_without_a_second_press() {
        let root=tempfile::tempdir().unwrap();saved_wifi(root.path());
        let mut menu=Menu::new(root.path().into());menu.startup();
        assert!(menu.visible && menu.locked && !menu.wifi_on);assert_eq!(menu.code,Some(Field::Unlock));
        submit_code(&mut menu,&[Btn::A;4]);wait(&mut menu);
        assert!(menu.locked && !menu.wifi_on);assert_eq!(menu.code,Some(Field::Unlock));
        assert_eq!(menu.code_text.as_str(),"aaaa");assert_eq!(menu.notice_text(),Some("Use 6+ presses"));
        for _ in 0..4 {menu.input(Action::GbaDown(Btn::Select));}
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
        menu.update();assert_eq!(menu.take_toast(),Some("Saved credentials are locked"));assert!(!menu.visible && !menu.wifi_on && menu.locked);assert!(menu.after_unlock.is_none());
        menu.startup();assert!(!menu.visible,"the cancelled card must not reopen this boot");
        menu.open_wifi();menu.input(Action::GbaDown(Btn::A));assert_eq!(menu.code,Some(Field::Unlock));
        menu.input(Action::QuickMenu);assert!(!menu.wifi_on && menu.after_unlock.is_none());
    }
    #[test]
    fn achievements_enabling_uses_the_same_pin_and_resumes_after_unlock() {
        let root=tempfile::tempdir().unwrap();saved_wifi(root.path());
        let mut menu=Menu::new(root.path().into());menu.open_achievements();menu.input(Action::GbaDown(Btn::A));
        assert!(!menu.prefs.achievements);assert_eq!(menu.code,Some(Field::Unlock));
        submit_code(&mut menu,&CODE);wait(&mut menu);assert!(menu.prefs.achievements && !menu.locked);assert!(!menu.wifi_on);
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
        wait(menu);assert!(!menu.locked&&menu.wifi_on);
        assert!(menu.code.is_none(),"the correct raw sequence unlocks without Start");
    }



    #[test]
    fn button_pin_length_is_chosen_by_start_and_long_sequences_survive_restart() {
        let root=tempfile::tempdir().unwrap();let mut menu=Menu::new(root.path().into());menu.open_security();menu.selected=1;menu.input(Action::GbaDown(Btn::A));
        buttons(&mut menu,&CODE[..3]);menu.input(Action::GbaDown(Btn::Start));assert_eq!(menu.code,Some(Field::CreateVault));
        menu.code_text.zeroize();let long:Vec<_>=CODE.into_iter().cycle().take(384).collect();buttons(&mut menu,&long);
        assert_eq!(menu.code,Some(Field::CreateVault));assert_eq!(menu.code_text.len(),384);
        menu.input(Action::GbaDown(Btn::Start));assert_eq!(menu.code,Some(Field::ConfirmVault));
        submit_code(&mut menu,&long);wait(&mut menu);assert!(!menu.short_pin);assert_eq!(menu.notice_tone(),slot_ui::NoticeTone::Neutral);
        let mut restarted=Menu::new(root.path().into());restarted.open_security();restarted.unlock(None);submit_code(&mut restarted,&long);wait(&mut restarted);assert!(!restarted.locked);
    }
    #[test]
    fn required_startup_pin_cannot_be_skipped_even_without_network_features() {
        let root=tempfile::tempdir().unwrap();let mut menu=Menu::new(root.path().into());pin(&mut menu);
        menu.prefs.lock_device=true;menu.store_preferences();drop(menu);
        let mut menu=Menu::new(root.path().into());menu.startup();assert!(menu.visible&&menu.locked);
        menu.input(Action::QuickMenu);assert!(menu.visible&&menu.code==Some(Field::Unlock));assert!(menu.take_toast().is_none());
        submit_code(&mut menu,&CODE);wait(&mut menu);assert!(!menu.visible&&!menu.locked&&!menu.wifi_on);
    }
    #[test]
    fn wifi_indicator_requires_an_actual_connection_and_clears_when_disabled() {
        let root=tempfile::tempdir().unwrap();let mut menu=Menu::new(root.path().into());assert!(!menu.wifi_connected());
        menu.wifi_on=true;menu.last_wifi.on=true;menu.last_wifi.connected=Some("Home".into());assert!(!menu.wifi_connected());
        menu.last_wifi.ip=Some("192.0.2.3".into());assert!(menu.wifi_connected());
        menu.suspend_for_link();assert!(!menu.wifi_connected());
    }
    #[test]
    fn shared_parent_pin_is_verified_even_if_credentials_are_already_unlocked() {
        let root=tempfile::tempdir().unwrap();let mut menu=Menu::new(root.path().into());pin(&mut menu);
        std::fs::create_dir_all(root.path().join("Config/kids")).unwrap();std::fs::write(root.path().join("Config/kids/use-device-pin"),b"1\n").unwrap();
        crate::parent_code::Gate::new(root.path().join("Config/kids/code")).set(&[0,1,2,3]).unwrap();
        menu.visible=false;let mut app=crate::app::App::new(vec![]);app.network=Some(menu);
        app.kid_menu=Some(crate::kid_menu::Menu::new(root.path().into(),crate::kid_policy::Policy::default()));
        app.apply(Action::GbaDown(Btn::A));assert!(app.network.as_ref().unwrap().parent_unlock);
        for b in [Btn::A,Btn::B,Btn::X,Btn::Y] {app.apply(Action::GbaDown(b));}app.apply(Action::GbaDown(Btn::Start));
        let mut now=0;
        while app.network.as_ref().unwrap().pending_jobs>0 {now+=10;app.tick_ms(now);std::thread::sleep(Duration::from_millis(5));assert!(now<20000);}
        assert!(app.kid_menu.as_ref().unwrap().focus_info().is_none());
        for _ in 0..4 {app.apply(Action::GbaDown(Btn::Select));}
        for b in CODE {app.apply(Action::GbaDown(b));}
        while app.network.as_ref().unwrap().pending_jobs>0 {now+=10;app.tick_ms(now);std::thread::sleep(Duration::from_millis(5));assert!(now<20000);}
        assert!(!app.network.as_ref().unwrap().visible);assert!(app.kid_menu.as_ref().unwrap().focus_info().is_some());
    }



    #[test]
    fn variable_pin_checks_silently_and_accepts_more_input_while_worker_is_busy() {
        let root=tempfile::tempdir().unwrap();let mut menu=Menu::new(root.path().into());
        let long:Vec<_>=CODE.into_iter().cycle().take(8).collect();
        menu.open_security();menu.selected=1;menu.input(Action::GbaDown(Btn::A));
        submit_code(&mut menu,&long);submit_code(&mut menu,&long);wait(&mut menu);
        let mut menu=Menu::new(root.path().into());menu.open_security();menu.unlock(None);
        buttons(&mut menu,&long[..4]);assert!(!menu.show_progress());wait(&mut menu);
        assert!(menu.locked);assert_eq!(menu.code_text.len(),4);assert!(menu.notice_text().is_none());
        buttons(&mut menu,&long[4..]);wait(&mut menu);
        assert!(!menu.locked);assert!(menu.code.is_none());
        // Fast typing must preserve all presses while an older prefix is being derived.
        let mut menu=Menu::new(root.path().into());menu.open_security();menu.unlock(None);
        buttons(&mut menu,&long);assert_eq!(menu.code_text.len(),8);wait(&mut menu);
        assert!(!menu.locked);assert!(menu.code_text.is_empty());
        let mut menu=Menu::new(root.path().into());menu.open_security();menu.unlock(None);
        buttons(&mut menu,&long);buttons(&mut menu,&[Btn::B,Btn::Y]);wait(&mut menu);
        assert!(!menu.locked,"correct prefixes typed while checking must not be skipped");
    }
    #[test]
    fn cancelling_a_pending_automatic_pin_check_cannot_unlock_later() {
        let root=tempfile::tempdir().unwrap();let mut menu=Menu::new(root.path().into());pin(&mut menu);
        let mut menu=Menu::new(root.path().into());menu.open_security();menu.unlock(None);
        buttons(&mut menu,&CODE);assert!(menu.pending_jobs>0);
        menu.input(Action::QuickMenu);wait(&mut menu);
        assert!(menu.locked);assert!(menu.code.is_none());assert!(!menu.parent_authenticated);
    }


    #[test]
    fn network_actions_without_a_pin_open_setup_without_enabling_services() {
        let root=tempfile::tempdir().unwrap();let mut m=Menu::new(root.path().into());
        m.open_wifi();m.input(Action::GbaDown(Btn::A));
        assert_eq!(m.page,Page::Security);assert!(!m.wifi_on);assert!(m.service.is_none());
        m.open_achievements();m.input(Action::GbaDown(Btn::A));
        assert_eq!(m.page,Page::Security);assert!(!m.prefs.achievements);
        assert!(!m.rows().iter().any(|r|r.0=="Credentials"));
    }
    #[test]
    fn old_plaintext_is_not_used_until_pin_creation_and_encrypted_save() {
        let root=tempfile::tempdir().unwrap();let mut old=Secrets::default();old.wifi_password="migration-secret".into();
        write_plain(root.path(),&old).unwrap();let mut m=Menu::new(root.path().into());m.startup();
        assert!(!m.wifi_on && m.service.is_none());assert!(m.secrets.wifi_password.is_empty());
        pin(&mut m);assert!(!plain_path(root.path()).exists());assert!(m.encrypted);
        let (_,saved)=network_vault::test_decrypt(root.path(),"acgjdi");assert_eq!(saved.wifi_password,"migration-secret");
    }
    #[test]
    fn orphan_plaintext_cleanup_preserves_authenticated_vault_and_unrelated_files() {
        let root=tempfile::tempdir().unwrap();let mut menu=Menu::new(root.path().into());
        menu.secrets.token="current-encrypted-token".into();pin(&mut menu);drop(menu);
        let orphan=root.path().join(format!("Config/.network-credentials.json.{}.0.tmp",std::process::id()));
        std::fs::write(&orphan,b"{\"token\":\"stale-or-incomplete-plaintext").unwrap();
        let unrelated=root.path().join("Config/.network-credentials.json.backup.tmp");
        std::fs::write(&unrelated,b"unrelated file").unwrap();
        crate::root::sweep_temps(root.path());assert!(orphan.exists(),"fixture must exercise PID reuse");
        let mut menu=Menu::new(root.path().into());assert!(menu.migrating_plain&&!menu.credentials_ready());
        menu.open_security();menu.unlock(None);buttons(&mut menu,&CODE);wait(&mut menu);
        assert!(menu.credentials_ready());assert!(!orphan.exists());assert!(unrelated.exists());
        let (_,saved)=network_vault::test_decrypt(root.path(),"acgjdi");
        assert_eq!(saved.token,"current-encrypted-token","orphan content must not replace the authenticated vault");
    }
    #[test]
    fn new_vault_cleans_orphan_plaintext_without_importing_uncommitted_secrets() {
        let root=tempfile::tempdir().unwrap();std::fs::create_dir(root.path().join("Config")).unwrap();
        let orphan=root.path().join("Config/.network-credentials.json.123.0.tmp");
        std::fs::write(&orphan,b"unfinished secret").unwrap();
        let mut menu=Menu::new(root.path().into());assert!(menu.migrating_plain);pin(&mut menu);
        assert!(menu.credentials_ready());assert!(!orphan.exists());
        assert!(network_vault::test_decrypt(root.path(),"acgjdi").1.token.is_empty());
    }
    #[test]
    fn failed_orphan_plaintext_cleanup_keeps_network_gated() {
        let root=tempfile::tempdir().unwrap();let mut menu=Menu::new(root.path().into());pin(&mut menu);drop(menu);
        let blocked=root.path().join("Config/.network-credentials.json.123.0.tmp");
        std::fs::create_dir(&blocked).unwrap();
        let mut menu=Menu::new(root.path().into());menu.open_security();menu.unlock(None);buttons(&mut menu,&CODE);wait(&mut menu);
        assert!(menu.migrating_plain&&!menu.credentials_ready());assert!(!menu.wifi_on&&menu.service.is_none());
        assert!(menu.notice_text().unwrap().contains("Could not delete"));assert!(blocked.is_dir());
    }
    #[test]
    fn inaccessible_plaintext_directory_is_not_reported_clean() {
        let root=tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("Config"),b"not a directory").unwrap();
        assert!(has_plain_residue(root.path()),"scan errors must preserve the network gate");
        assert!(remove_plain(root.path()).is_err(),"directory errors must not masquerade as completed cleanup");
        assert!(!Menu::new(root.path().into()).credentials_ready());
    }
    #[test]
    fn old_four_button_pin_unlocks_for_upgrade_but_not_for_network() {
        let root=tempfile::tempdir().unwrap();network_vault::test_legacy(root.path(),"acgj",Secrets::default());
        let mut m=Menu::new(root.path().into());m.open_wifi();m.reconnect();
        buttons(&mut m,&CODE[..4]);wait(&mut m);
        assert!(m.upgrade_pin);assert!(!m.wifi_on && m.service.is_none());assert_eq!(m.code,Some(Field::CreateVault));
        submit_code(&mut m,&CODE);submit_code(&mut m,&CODE);wait(&mut m);
        assert!(!m.upgrade_pin && m.wifi_on);
    }
    #[test]
    fn opening_network_does_not_replay_a_previous_unlock_notice() {
        let root=tempfile::tempdir().unwrap();let mut m=Menu::new(root.path().into());
        m.say("Credentials unlocked");m.open_wifi();assert!(m.notice_text().is_none());
        m.say("Credentials unlocked");m.open_achievements();assert!(m.notice_text().is_none());
    }

    #[test]
    fn failed_plaintext_migration_preserves_both_data_and_network_gate() {
        let root=tempfile::tempdir().unwrap();let mut old=Secrets::default();old.token="migration-token".into();
        write_plain(root.path(),&old).unwrap();
        let blocked=root.path().join("Config/.network-credentials.json.123.4.tmp");std::fs::create_dir(&blocked).unwrap();
        let mut m=Menu::new(root.path().into());pin(&mut m);
        assert!(plain_path(root.path()).is_file());assert!(m.migrating_plain && !m.credentials_ready());
        assert!(!m.wifi_on && m.service.is_none());
        let (_,saved)=network_vault::test_decrypt(root.path(),"acgjdi");assert_eq!(saved.token,"migration-token");
        assert!(m.notice_text().unwrap().contains("Could not delete"));
    }
    #[test]
    fn failed_vault_creation_never_deletes_the_old_plaintext() {
        let root=tempfile::tempdir().unwrap();let mut old=Secrets::default();old.token="keep-token".into();write_plain(root.path(),&old).unwrap();
        std::fs::create_dir(network_vault::vault_path(root.path())).unwrap();
        let mut m=Menu::new(root.path().into());m.locked=false;m.open_security();m.selected=1;m.input(Action::GbaDown(Btn::A));
        submit_code(&mut m,&CODE);submit_code(&mut m,&CODE);wait(&mut m);
        assert_eq!(read_plain(root.path()).unwrap().token,"keep-token");assert!(!m.wifi_on && m.service.is_none());
    }

    #[cfg(target_os="linux")]
    #[test]
    #[ignore = "requires Mesa EGL; produces production-compositor review captures"]
    fn native_ra_progress_review() {
        use slot_gfx::Compositor;
        use slot_input::{InputSource,RawEvent,Millis};
        struct Empty;impl InputSource for Empty {fn poll(&mut self,_:Millis)->Vec<RawEvent>{vec![]}}
        let dir=std::path::PathBuf::from(std::env::var_os("SLOT_QOL_PREVIEW").expect("preview directory"));
        for (name,width,light) in [("ra-light-gles",640,true),("ra-dark-gl",720,false)] {
            let output=dir.join(name);std::fs::create_dir_all(&output).unwrap();let root=tempfile::tempdir().unwrap();crate::root::ensure(root.path());
            let mut state=slot_store::SlotState::default();state.clock_set=true;state.appearance=if light {slot_store::Appearance::Light}else{slot_store::Appearance::Dark};slot_store::write_slot_state(root.path(),&state).unwrap();
            let surface=super::review_surface::Egl::new(light);let mut g=Compositor::new(&surface).unwrap();g.fit((width,480));
            let mut f=crate::frontend::Frontend::boot(Box::new(slot_power::SimPlatform::at(root.path().into())));f.upload_faces(&mut g);f.render(&mut g,(width,480));
            for i in 0..90 {f.advance_at(&mut Empty,i*17,1.0/60.0);f.compose(&mut g);}
            let m=f.app_mut_for_review().network.as_mut().unwrap();m.open_achievements();m.prefs.achievements=true;m.selected=6;m.secrets.username="Preview".into();m.secrets.token="fixture".into();
            let mut review_tick=2000;
            m.service=Some(slot_achievements::Service::configured(root.path().into(),slot_achievements::Config::default()));
            for (shot,done,total,badges,status,problem,retry) in [
                ("game-data",6,900,5,slot_achievements::SyncStatus::Syncing,"",0),
                ("artwork-downloading",900,900,175,slot_achievements::SyncStatus::Syncing,"",0),
                ("artwork-retry",900,900,175,slot_achievements::SyncStatus::Attention,"Artwork server unavailable",20),
                ("up-to-date",900,900,800,slot_achievements::SyncStatus::Ready,"",0)] {
                for _ in 0..60 {
                review_tick+=17;f.advance_at(&mut Empty,review_tick,1.0/60.0);
                let m=f.app_mut_for_review().network.as_mut().unwrap();m.last_sync=slot_achievements::SyncDetail{catalog_ready:true,artwork_ready:true,authenticated:true,games_done:done,games_total:total,badges_done:badges,badges_total:800,status,problem,retry_seconds:retry,..Default::default()};m.face=None;m.notice.clear();
                f.compose(&mut g);
                }
                let mut e=png::Encoder::new(std::fs::File::create(output.join(format!("{shot}.png"))).unwrap(),width,480);e.set_color(png::ColorType::Rgba);e.set_depth(png::BitDepth::Eight);e.write_header().unwrap().write_image_data(&g.read_frame()).unwrap();
            }
        }
    }
    #[test]
    fn general_network_tools_route_to_wifi_and_return_without_enabling_it() {
        let root=tempfile::tempdir().unwrap();let mut app=crate::app::App::new(vec![]);app.network=Some(Menu::new(root.path().into()));
        app.apply(Action::QuickMenu);app.apply(Action::GbaDown(Btn::A));
        assert_eq!(app.quick_menu(),Some(slot_ui::QuickRow::SecurityPin));app.apply(Action::GbaDown(Btn::Down));
        assert_eq!(app.quick_menu(),Some(slot_ui::QuickRow::Updates));app.apply(Action::GbaDown(Btn::A));
        let m=app.network.as_ref().unwrap();assert_eq!(m.page,Page::Wifi);assert_eq!(m.notice_text(),Some("Wi-Fi needed for updates"));assert!(!m.wifi_on);
        app.apply(Action::GbaDown(Btn::B));assert_eq!(app.quick_menu(),Some(slot_ui::QuickRow::Updates));app.apply(Action::GbaDown(Btn::Down));app.apply(Action::GbaDown(Btn::A));
        let m=app.network.as_ref().unwrap();assert_eq!(m.notice_text(),Some("Wi-Fi needed for automatic time"));assert!(!m.prefs.automatic_time);
        app.apply(Action::GbaDown(Btn::B));
        let m=app.network.as_mut().unwrap();m.wifi_on=true;m.last_wifi.on=true;m.last_wifi.connected=Some("Home".into());m.last_wifi.ip=Some("192.0.2.1".into());
        app.apply(Action::GbaDown(Btn::A));assert!(app.network.as_ref().unwrap().prefs.automatic_time);assert!(!app.network.as_ref().unwrap().visible);
        app.apply(Action::GbaDown(Btn::Up));app.apply(Action::GbaDown(Btn::A));assert_eq!(app.network.as_ref().unwrap().page,Page::Updates);app.apply(Action::GbaDown(Btn::B));assert!(!app.network.as_ref().unwrap().visible);assert_eq!(app.quick_menu(),Some(slot_ui::QuickRow::Updates));
    }
    #[derive(Default)]
    struct UpdateFake {asked:std::sync::Arc<std::sync::Mutex<Vec<UpdateJob>>>,events:std::collections::VecDeque<UpdateEvent>}
    impl UpdateJobs for UpdateFake {
        fn ask(&mut self,job:UpdateJob) {self.asked.lock().unwrap().push(job);}
        fn take(&mut self)->Option<UpdateEvent> {self.events.pop_front()}
        fn cancel(&self)->bool {true}
    }
    #[test]
    fn checking_updates_animates_without_a_checking_text_and_keeps_baseos_detail() {
        let root=tempfile::tempdir().unwrap();let mut m=Menu::new(root.path().into());
        m.wifi_on=true;m.last_wifi=home_wifi();m.open_updates();m.updates=Box::new(UpdateFake::default());
        m.check_updates();assert!(m.show_progress());assert_eq!(m.rows()[0].1,"");
        m.updates=Box::new(UpdateFake{events:[UpdateEvent::BaseOsStatus("Check failed · retry".into()),UpdateEvent::UpToDate].into(),..Default::default()});
        m.take_update_events();assert!(!m.show_progress());
        assert!(m.rows().iter().any(|r|r.0=="BaseOS check" && r.1=="Check failed · retry"));
    }
    #[test]
    fn automatic_time_can_be_disabled_offline_and_refuses_manual_clock_with_a_pill() {
        let root=tempfile::tempdir().unwrap();let mut app=crate::App::boot(root.path());
        app.update(10.0);app.confirm_clock();app.update(10.0);
        let menu=app.network.as_mut().unwrap();menu.prefs.automatic_time=true;menu.wifi_on=false;
        menu.toggle_automatic_time();assert!(!menu.automatic_time_enabled());
        menu.prefs.automatic_time=true;
        app.apply(Action::QuickMenu);app.apply(Action::GbaDown(Btn::A));
        for _ in 0..7 {app.apply(Action::GbaDown(Btn::Down));}
        assert_eq!(app.quick_menu(),Some(slot_ui::QuickRow::DateTime));
        app.apply(Action::GbaDown(Btn::A));assert!(app.picker().is_none());
        assert_eq!(app.settings_notice().unwrap().1,"Automatic time is on");
    }
    #[test]
    fn updater_is_explicit_connected_and_battery_gated() {
        let root=tempfile::tempdir().unwrap();let mut m=Menu::new(root.path().into());
        let fake=UpdateFake::default();let asked=fake.asked.clone();m.updates=Box::new(fake);
        m.open_updates();m.check_updates();assert!(asked.lock().unwrap().is_empty());
        m.wifi_on=true;m.last_wifi.on=true;m.last_wifi.connected=Some("Home".into());m.last_wifi.ip=Some("192.0.2.1".into());
        assert!(!m.automatic_time_ready());m.prefs.automatic_time=true;assert!(m.automatic_time_ready());m.prefs.automatic_time=false;assert!(!m.automatic_time_ready());
        m.releases.push(update::Release{kind:Kind::BaseOs,running:"1.3.0".into(),version:"1.4.0".into(),url:"fixture".into(),sha256:[0;32],size:100,source:String::new()});
        m.set_battery(Some(slot_power::Battery{percent:10,charge:slot_power::Charge::Discharging}));m.install_update(0);
        assert!(asked.lock().unwrap().is_empty());assert!(m.notice_text().unwrap().contains("Plug in"));
        m.set_battery(Some(slot_power::Battery{percent:10,charge:slot_power::Charge::Charging}));m.install_update(0);
        assert!(matches!(asked.lock().unwrap().last(),Some(UpdateJob::Install(_))));assert!(m.update_busy);
    }
    #[test]
    fn update_progress_and_completion_refresh_the_settings_face() {
        let root=tempfile::tempdir().unwrap();let mut m=Menu::new(root.path().into());m.wifi_on=true;m.last_wifi.on=true;m.last_wifi.connected=Some("Home".into());m.last_wifi.ip=Some("192.0.2.1".into());m.open_updates();m.update_busy=true;
        m.updates=Box::new(UpdateFake{events:[UpdateEvent::Progress(50),UpdateEvent::Staged].into(),..Default::default()});
        m.face=Some(TexId::from_raw(1));m.take_update_events();
        assert!(m.face.is_none());assert_eq!(m.update_bytes,50);assert!(m.update_staged&&!m.update_busy);
        assert_eq!(m.update_message,"Restart to install");
        assert_eq!(m.rows()[0].0,"Restart to install");m.input(Action::GbaDown(Btn::A));assert!(m.take_update_restart());assert!(!m.take_update_restart());
    }
    #[test]
    fn early_done_preserves_entry_and_enables_hint_only_at_minimum() {
        let root=tempfile::tempdir().unwrap();let mut m=Menu::new(root.path().into());m.prompt(Field::CreateVault);
        assert_eq!(m.code_text.len(),0);assert_eq!(m.pin_hint_key(),0);
        buttons(&mut m,&CODE[..3]);m.input(Action::GbaDown(Btn::Start));
        assert_eq!(m.code_text.len(),3);assert!(m.pin_rejected.is_some());assert_eq!(m.notice_text(),Some("Use 6+ presses"));
        buttons(&mut m,&CODE[3..]);assert_eq!(m.code_text.len(),6);assert_eq!(m.pin_hint_key(),1);assert!(m.pin_rejected.is_none());
    }
    #[test]
    fn parent_device_pin_cancel_returns_directly_to_previous_menu() {
        let root=tempfile::tempdir().unwrap();let mut network=Menu::new(root.path().into());pin(&mut network);
        std::fs::create_dir_all(root.path().join("Config/kids")).unwrap();std::fs::write(root.path().join("Config/kids/use-device-pin"),b"1").unwrap();
        let mut app=crate::app::App::new(vec![]);app.network=Some(network);
        app.kid_menu=Some(crate::kid_menu::Menu::new(root.path().into(),crate::kid_policy::Policy::default()));
        app.network.as_mut().unwrap().unlock_for_parent();app.apply(Action::QuickMenu);app.update(0.02);
        assert!(!app.network.as_ref().unwrap().visible);assert!(app.kid_menu.is_none());
        let mut policy=crate::kid_policy::Policy::default();policy.active=true;
        app.kid_menu=Some(crate::kid_menu::Menu::new(root.path().into(),policy));
        app.network.as_mut().unwrap().unlock_for_parent();app.apply(Action::QuickMenu);app.update(0.02);
        assert!(app.kid_menu.as_ref().unwrap().settings_grid());assert!(!app.network.as_ref().unwrap().parent_authenticated);
    }
    #[test]
    fn ra_meter_is_a_separate_compositor_overlay() {
        let root=tempfile::tempdir().unwrap();let mut m=Menu::new(root.path().into());m.open_achievements();
        m.service=Some(slot_achievements::Service::configured(root.path().into(),slot_achievements::Config::default()));
        m.last_sync=slot_achievements::SyncDetail{games_done:6,games_total:900,..Default::default()};
        assert_eq!(m.floating_progress(),Some(("Game data".into(),6,900)));
        let face=m.screen_face_on(slot_ui::Canvas::FRAMED,false);
        assert!(face.rgba[(350*face.w*4)as usize..(378*face.w*4)as usize].iter().all(|&v|v==0));
        let mut app=crate::app::App::new(vec![]);app.network=Some(m);app.network_meter_face=Some((slot_ui::TexId::from_raw(60000),400,40));
        let mut out=Vec::new();app.draw(&mut out);
        assert!(out.iter().any(|d|matches!(d,slot_ui::Draw::Tex{tex,y,h,..} if *tex==slot_ui::TexId::from_raw(60000) && *y==slot_ui::expressive::FOOTER_Y-2.0 && *h==40.0)));
        app.network.as_mut().unwrap().prompt(Field::Unlock);assert!(app.network.as_ref().unwrap().floating_progress().is_none());

    }

}

#[cfg(all(test,target_os="linux"))]
#[path="../../slot-ui/examples/support/mod.rs"]
mod review_surface;
