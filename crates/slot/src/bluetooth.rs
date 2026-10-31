//! Asynchronous headphone management through BaseOS's existing BlueZ/BlueALSA stack.
use std::path::PathBuf;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::sync::mpsc::{self, Receiver};

use slot_input::{Action, Btn};
use slot_ui::{TexId, UndoFace, OUT_H};
#[cfg(test)]
use slot_ui::OUT_W;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Headphones {
    pub address: String,
    pub name: String,
    pub paired: bool,
    pub connected: bool,
}

pub fn parse_devices(text: &str) -> Vec<Headphones> {
    let mut devices = Vec::new();
    for line in text.lines() {
        let Some(rest) = line.strip_prefix("DEVICE ") else {
            continue;
        };
        let Some((address, name)) = rest.split_once(' ') else {
            continue;
        };
        let raw = name.trim().replace([':', '-'], "");
        let named = !(name.trim().is_empty()
            || raw.len() == 12 && raw.bytes().all(|b| b.is_ascii_hexdigit()));
        if named
            && valid_address(address)
            && !devices.iter().any(|d: &Headphones| d.address == address)
        {
            devices.push(Headphones {
                address: address.to_owned(),
                name: name.to_owned(),
                paired: false,
                connected: false,
            });
        }
    }
    for line in text.lines() {
        let words: Vec<_> = line.split_whitespace().collect();
        if words.len() == 4 && words[0] == "STATE" {
            if let Some(d) = devices.iter_mut().find(|d| d.address == words[1]) {
                d.paired = words[2] == "yes";
                d.connected = words[3] == "yes";
            }
        }
    }
    devices
}

pub fn valid_address(address: &str) -> bool {
    let parts: Vec<_> = address.split(':').collect();
    parts.len() == 6
        && parts
            .iter()
            .all(|p| p.len() == 2 && p.bytes().all(|b| b.is_ascii_hexdigit()))
}

fn bluetooth_enabled(root: &std::path::Path) -> bool {
    !root.join("Config/bluetooth-off").exists()
        && (root.join("Config/bluetooth-enabled").exists()
            || root.join("Config/bluetooth-headphones").exists())
}

fn startup_verb(root: &std::path::Path) -> &'static str {
    if !bluetooth_enabled(root) {
        "off"
    } else if root.join("Config/bluetooth-headphones").exists() {
        "restore"
    } else {
        "on"
    }
}

struct ResultMessage {
    // None means this operation did not return an authoritative device list.
    devices: Option<Vec<Headphones>>,
    status: String,
    output: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Page {
    Home,
    Pair,
}

pub struct Manager {
    root: PathBuf,
    pub page: Page,
    stack: Vec<Page>,
    pub devices: Vec<Headphones>,
    pub usb_devices: Vec<crate::usb_audio::Device>,
    pub selected: usize,
    pub status: String,
    output: String,
    pending: Option<Receiver<ResultMessage>>,
    recovering: Arc<AtomicBool>,
    cancel_path: Option<PathBuf>,
    pub face: Option<TexId>,
    pub progress_face: Option<TexId>,
    pub wired: bool,
    pub bluetooth_available: bool,
    pub disconnect_volume: u8,
    pub current_volume: u8,
    pub pause_on_disconnect: bool,
    pub disconnect_notice: bool,
    pub route_choice: Option<String>,
    choosing_route: bool,
    forget_confirmation: Option<String>,
}

impl Manager {
    pub fn new(root: PathBuf, bluetooth_available: bool) -> Self {
        let disconnect_volume = disconnect_volume(&root);
        let pause_on_disconnect = pause_on_disconnect(&root);
        let mut manager = Self {
            disconnect_volume,
            current_volume:50,
            pause_on_disconnect,
            bluetooth_available,
            root,
            page: Page::Home,
            stack: Vec::new(),
            devices: Vec::new(),
            usb_devices: Vec::new(),
            selected: 0,
            status: String::new(),
            output: "speaker".into(),
            pending: None,
            recovering: Arc::default(),
            cancel_path: None,
            face: None,
            progress_face: None,
            wired: false,
            disconnect_notice: false,
            route_choice: None,
            choosing_route: false,
            forget_confirmation: None,
        };
        if bluetooth_available {
            manager.ask("list", None);
        }
        manager
    }

    pub fn set_available(&mut self, available: bool) {
        if available && !self.bluetooth_available {
            self.bluetooth_available = true;
            self.ask("list", None);
        }
    }

    pub fn busy(&self) -> bool {
        self.pending.is_some()
    }

    pub fn select_usb(&mut self, route: String) {
        self.ask("usb", Some(route));
    }

    // An automatic switch can start with the menu closed. Hand its worker to
    // the visible menu when it opens, so user actions cannot race that switch.
    pub fn adopt_usb_switch(&mut self, other: &mut Self) {
        self.cancel_pending();
        self.pending = other.pending.take();
        self.cancel_path = other.cancel_path.take();
        self.recovering = other.recovering.clone();
        self.choosing_route = other.choosing_route;
        self.status = std::mem::take(&mut other.status);
        self.route_choice = other.route_choice.take();
    }

    pub fn cancel_pending(&mut self) {
        if self.pending.take().is_some() {
            if let Some(path) = self.cancel_path.take() { let _ = std::fs::write(path, b"cancel"); }
            self.recovering.store(false, Ordering::Relaxed);
            self.choosing_route = false;
            self.progress_face = None;
            self.status = "Connection cancelled".into();
            self.face = None;
        }
    }

    fn ask(&mut self, verb: &'static str, address: Option<String>) {
        if self.pending.is_some() {
            return;
        }
        self.forget_confirmation = None;
        self.choosing_route = matches!(verb, "speaker" | "connect" | "off" | "usb" | "local");
        let root = self.root.clone();
        let (tx, rx) = mpsc::channel();
        self.pending = Some(rx);
        self.status = match verb {
            "scan" => "Scanning…",
            "connect"
                if self
                    .devices
                    .iter()
                    .any(|d| Some(&d.address) == address.as_ref() && d.paired) =>
            {
                "Connecting..."
            }
            "connect" => "Pairing and connecting...",
            "forget" => "Forgetting device...",
            "speaker" | "local" | "usb" => "Switching output...",
            "off" => "Turning Bluetooth off...",
            "on" => "Turning Bluetooth on...",
            "diagnose" => "Saving report...",
            _ => "",
        }
        .into();
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let cancel = std::env::temp_dir().join(format!("slot-bt-{}-{}.cancel", std::process::id(), NEXT.fetch_add(1,Ordering::Relaxed)));
        let _ = std::fs::remove_file(&cancel);
        self.cancel_path = Some(cancel.clone());
        let bluetooth_available = self.bluetooth_available;
        self.recovering.store(false, Ordering::Relaxed);
        let recovering = self.recovering.clone();
        std::thread::spawn(move || {
            let result = if verb == "usb" || verb == "local" {
                run_usb(root, address, bluetooth_available)
            } else {
                run_with_recovery(verb, address,
                    || recovering.store(true, Ordering::Relaxed),
                    |verb, address| run_cancel(root.clone(), verb, address, Some(&cancel)))
            };
            let _ = std::fs::remove_file(cancel);
            let _ = tx.send(result);
        });
    }

    pub fn poll(&mut self) {
        let Some(rx) = &self.pending else { return };
        match rx.try_recv() {
            Ok(message) => {
                self.recovering.store(false, Ordering::Relaxed);
                if !message.status.starts_with("Error:")
                    && !message.status.starts_with("Saved Bluetooth/")
                {
                    if let Some(devices) = message.devices {
                        self.devices = devices;
                    }
                }
                if let Some(output) = message.output {
                    if self.choosing_route && !message.status.starts_with("Error:") {
                        self.route_choice = Some(output.clone());
                        self.page = Page::Home;
                        self.stack.clear();
                        self.selected = self
                            .device_indices()
                            .iter()
                            .position(|i| self.devices[*i].address == output)
                            .map(|i| self.device_start() + i)
                            .or_else(|| {
                                self.usb_devices
                                    .iter()
                                    .position(|d| d.route == output)
                                    .map(|i| self.device_start() + self.device_indices().len() + i)
                            })
                            .unwrap_or(self.local_row());
                    }
                    self.choosing_route = false;
                    self.output = output;
                }
                self.status = if message.status.starts_with("Error:")
                    || message.status.starts_with("Saved Bluetooth/")
                {
                    message.status
                } else {
                    String::new()
                };
                self.selected = self.selected.min(self.rows().len().saturating_sub(1));
                self.pending = None;
                self.cancel_path = None;
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                self.status = "Error: Bluetooth worker stopped".into();
                self.pending = None;
                self.cancel_path = None;
            }
            Err(mpsc::TryRecvError::Empty) => {}
        }
    }

    pub fn observe_output(&mut self, address: Option<&str>, connected: bool) {
        self.output = address.unwrap_or("speaker").to_owned();
        for device in &mut self.devices {
            if Some(device.address.as_str()) == address {
                device.connected = connected;
            }
        }
    }

    fn open(&mut self, page: Page) {
        self.forget_confirmation = None;
        self.stack.push(self.page.clone());
        self.page = page;
        self.selected = 0;
        self.status.clear();
    }

    fn local_row(&self) -> usize { 1 + usize::from(self.bluetooth_available) }
    fn device_start(&self) -> usize { self.local_row() + 1 }
    fn pair_row(&self) -> usize { self.device_start() + self.device_indices().len() + self.usb_devices.len() }
    fn disconnect_row(&self) -> usize { self.pair_row() + usize::from(self.bluetooth_available) }
    fn pause_row(&self) -> usize { self.disconnect_row() + 1 }
    pub fn slider_selected(&self)->bool {self.volume_selected() || (self.page==Page::Home && self.selected==self.disconnect_row())}
    pub fn volume_selected(&self)->bool {self.page==Page::Home && self.selected==0}

    fn device_indices(&self) -> Vec<usize> {
        self.devices
            .iter()
            .enumerate()
            .filter(|(_, d)| self.bluetooth_available && (self.page == Page::Pair || d.paired))
            .map(|(i, _)| i)
            .collect()
    }

    fn active_device(&self) -> Option<usize> {
        self.devices
            .iter()
            .position(|d| d.address == self.output && d.connected)
    }

    pub fn rows(&self) -> Vec<String> {
        match self.page {
            Page::Home => {
                let local = if self.wired {
                    "WIRED_DEVICE Wired headphones"
                } else {
                    "SPEAKER_DEVICE Internal speaker"
                };
                let mut rows = vec![format!("VOLUME {}%",self.current_volume)];
                if self.bluetooth_available {
                    rows.push(format!("BLUETOOTH {}", if bluetooth_enabled(&self.root) { "On" } else { "Off" }));
                }
                rows.push(format!(
                    "{}{}",
                    if valid_address(&self.output) || crate::usb_audio::valid_route(&self.output) {
                        ""
                    } else {
                        "ACTIVE "
                    },
                    local
                ));
                rows.extend(self.device_indices().iter().map(|i| {
                    format!(
                        "{}DEVICE {}",
                        if self.active_device() == Some(*i) {
                            "ACTIVE "
                        } else {
                            ""
                        },
                        self.devices[*i].name
                    )
                }));
                rows.extend(self.usb_devices.iter().map(|d| {
                    format!(
                        "{}USB_DEVICE {}",
                        if d.route == self.output {
                            "ACTIVE "
                        } else {
                            ""
                        },
                        d.name
                    )
                }));
                if self.bluetooth_available {
                    rows.push("Pair audio device".into());
                }
                rows.push(format!("DISCONNECT {}%", self.disconnect_volume));
                rows.push(format!("PAUSE_DISCONNECT {}", if self.pause_on_disconnect {"On"} else {"Off"}));
                rows
            }
            Page::Pair => {
                let mut rows = vec!["Scan for devices".into()];
                rows.extend(
                    self.device_indices()
                        .iter()
                        .map(|i| format!("DEVICE {}", self.devices[*i].name)),
                );
                rows
            }
        }
    }

    pub fn input(&mut self, action: Action) -> bool {
        if action == Action::GbaDown(Btn::B) && self.forget_confirmation.take().is_some() {
            return false;
        }
        if !matches!(action, Action::GbaDown(Btn::Y))
            && matches!(action, Action::GbaDown(_) | Action::QuickMenu)
        {
            self.forget_confirmation = None;
        }
        if action == Action::QuickMenu {
            self.cancel_pending();
            return true;
        }
        if action == Action::GbaDown(Btn::B) {
            self.cancel_pending();
            if let Some(page) = self.stack.pop() {
                self.page = page;
                self.selected = 0;
            } else {
                return true;
            }
            return false;
        }
        if self.pending.is_some() {
            return false;
        }
        let count = self.rows().len();
        match action {
            Action::GbaDown(Btn::Up) => self.selected = (self.selected + count - 1) % count,
            Action::GbaDown(Btn::Down) => self.selected = (self.selected + 1) % count,
            Action::GbaDown(Btn::A | Btn::Left | Btn::Right)
                if self.page == Page::Home && self.selected == self.pause_row() =>
            { self.toggle_pause_on_disconnect(); }
            Action::GbaDown(Btn::Left | Btn::Right)
                if self.page == Page::Home && self.selected == self.disconnect_row() =>
            {
                self.change_disconnect_volume(action == Action::GbaDown(Btn::Right));
            }
            Action::GbaDown(Btn::Left | Btn::Right)
                if self.page == Page::Home
                    && self.bluetooth_available
                    && self.selected == 1 =>
            {
                self.toggle_bluetooth();
            }
            Action::GbaDown(Btn::X) if self.bluetooth_available => self.ask("diagnose", None),
            Action::GbaDown(Btn::Y) if self.page == Page::Home => {
                if let Some(i) = self
                    .selected
                    .checked_sub(self.device_start())
                    .and_then(|n| self.device_indices().get(n).copied())
                {
                    let address = self.devices[i].address.clone();
                    if self.forget_confirmation.as_ref() == Some(&address) {
                        self.ask("forget", Some(address));
                    } else {
                        self.forget_confirmation = Some(address);
                        self.status.clear();
                    }
                }
            }
            Action::GbaDown(Btn::A) => match self.page.clone() {
                Page::Home => {
                    if self.bluetooth_available && self.selected == 1 {
                        self.toggle_bluetooth();
                    } else if self.selected == self.local_row() {
                        self.ask(if self.bluetooth_available { "speaker" } else { "local" }, None);
                    } else if self.bluetooth_available && self.selected == self.pair_row() {
                        self.open(Page::Pair);
                        if !bluetooth_enabled(&self.root) { self.ask("on", None); }
                    } else if self.selected != self.disconnect_row() {
                        if let Some(n) = self.selected.checked_sub(self.device_start()) {
                            let devices = self.device_indices();
                            if let Some(&i) = devices.get(n) {
                                self.ask("connect", Some(self.devices[i].address.clone()));
                            } else if let Some(device) = self.usb_devices.get(n.saturating_sub(devices.len())) {
                                let route = device.route.clone();
                                if self.output != route { self.ask("usb", Some(route)); }
                            }
                        }
                    }
                },
                Page::Pair => {
                    if self.selected == 0 {
                        self.ask("scan", None);
                    } else if let Some(&i) = self.device_indices().get(self.selected - 1) {
                        self.ask("connect", Some(self.devices[i].address.clone()));
                    }
                }
            },
            _ => {}
        }
        false
    }

    fn toggle_bluetooth(&mut self) {
        self.ask(
            if bluetooth_enabled(&self.root) {
                "off"
            } else {
                "on"
            },
            None,
        );
    }

    fn toggle_pause_on_disconnect(&mut self) {
        let value = !self.pause_on_disconnect;
        match save_sound_value(&self.root, "pause_on_disconnect", if value {"1"} else {"0"}) {
            Ok(()) => { self.pause_on_disconnect = value; self.status.clear(); }
            Err(_) => self.status = "Error: Could not save pause setting".into(),
        }
    }

    fn change_disconnect_volume(&mut self, increase: bool) {
        let value = if increase {
            self.disconnect_volume.saturating_add(5).min(100)
        } else {
            self.disconnect_volume.saturating_sub(5)
        };
        match save_disconnect_volume(&self.root, value) {
            Ok(()) => {
                self.disconnect_volume = value;
                self.status.clear();
            }
            Err(_) => self.status = "Error: Could not save volume setting".into(),
        }
    }

    pub fn visible_sliders(&self)->Vec<slot_ui::MenuSlider> {
        let(_,_,first,visible)=self.scroll_info();
        if self.page!=Page::Home {return vec![];}
        [(self.disconnect_row(),self.disconnect_volume,4),(0,self.current_volume,3)].into_iter().filter(|(row,_,_)|(first..first+visible).contains(row)).map(|(row,value,id)|slot_ui::MenuSlider{width:112.0,span:200.0,id:self.page_key()*16+id,fraction:value as f32/100.0,y:94.0+(row-first)as f32*slot_ui::QUICK_PITCH}).collect()
    }

    pub fn visible_switches(&self) -> Vec<slot_ui::MenuSwitch> {
        let (_,_,first,visible)=self.scroll_info();
        self.rows().iter().enumerate().skip(first).take(visible).filter_map(|(i,row)| {
            let (id,value)=if let Some(v)=row.strip_prefix("BLUETOOTH ") {(1,v)}
                else if let Some(v)=row.strip_prefix("PAUSE_DISCONNECT ") {(2,v)} else {return None;};
            Some(slot_ui::MenuSwitch{id:self.page_key()*16+id,on:value=="On",y:94.0+(i-first) as f32*slot_ui::QUICK_PITCH})
        }).collect()
    }
    pub fn scroll_info(&self) -> (usize,usize,usize,usize) {
        let total=self.rows().len();
        let visible=total.min(slot_ui::QUICK_VISIBLE);
        (self.selected,total,self.selected.saturating_sub(slot_ui::QUICK_VISIBLE-1),visible)
    }
    pub fn page_key(&self) -> u64 { 1000 + self.page.clone() as u64 }
    pub fn lines(&self) -> Vec<String> {
        let title = "";
        let message = if self.forget_confirmation.is_some() {
            "CONFIRM_FORGET".to_owned()
        } else if self.recovering.load(Ordering::Relaxed) {
            "Error".into()
        } else if self.busy() {
            String::new()
        } else if self.status.starts_with("Error:") {
            "Error".into()
        } else if self.status.is_empty() && self.disconnect_notice {
            "Device disconnected — choose output".into()
        } else {
            self.status.clone()
        };
        let compact = matches!(self.page, Page::Home | Page::Pair);
        let mut lines = vec![
            title.to_owned(),
            message,
            if compact {
                "COMPACT".into()
            } else if self.status.starts_with("Error:") {
                "Log: Bluetooth/latest-error.txt".into()
            } else {
                String::new()
            },
        ];
        let visible = if compact { slot_ui::QUICK_VISIBLE } else { 3 };
        let first = self.selected.saturating_sub(visible - 1);
        for (i, row) in self.rows().iter().enumerate().skip(first).take(visible) {
            lines.push(format!(
                "{}{}",
                if i == self.selected {
                    "SELECTED "
                } else {
                    "ROW "
                },
                row
            ));
        }
        lines.push(
            if self.page == Page::Home
                && (self.selected == self.disconnect_row() || self.selected == self.pause_row() || self.volume_selected()
                    || (self.bluetooth_available && self.selected == 1))
            {
                "B Back · Change"
            } else if self.disconnect_notice && self.page == Page::Home {
                "A Select"
            } else if self.page == Page::Home
                && self.selected >= self.device_start()
                && self.selected < self.device_start() + self.device_indices().len()
            {
                "B Back · A Connect · Y Forget"
            } else if self.page == Page::Pair && self.selected > 0 {
                "B Back · A Connect"
            } else {
                "B Back · A Select"
            }
            .into(),
        );
        lines
    }
}

impl Drop for Manager {
    fn drop(&mut self) { self.cancel_pending(); }
}

// Radio power failures get one restart and retry; a missing/offline device does not.
// Background reconnect keeps its existing backoff instead of cycling the radio.
fn run_with_recovery(verb: &str, address: Option<String>, on_recover: impl FnOnce(),
    mut run: impl FnMut(&str, Option<String>) -> ResultMessage) -> ResultMessage {
    let first = run(verb, address.clone());
    if !first.status.starts_with("Error:") || matches!(verb, "diagnose" | "connect") { return first; }
    on_recover();
    let restarted = run("restart", None);
    if restarted.status.starts_with("Error:") { return restarted; }
    run(verb, address)
}

fn run_usb(root: PathBuf, route: Option<String>, bluetooth_available: bool) -> ResultMessage {
    let error = |s: &str| ResultMessage {
        devices: None,
        output: None,
        status: format!("Error: {s}"),
    };
    if let Some(route) = &route {
        if !crate::usb_audio::devices()
            .iter()
            .any(|d| d.route == *route)
        {
            return error("USB audio device is no longer connected");
        }
    }
    #[cfg(feature = "device")]
    if let Some(route) = &route {
        // One second of digital silence verifies playback/resampling without
        // changing mixer gain or selecting this output before it can open.
        let ready = std::process::Command::new("timeout")
            .args([
                "3",
                "aplay",
                "-q",
                "-D",
                route,
                "-t",
                "raw",
                "-f",
                "S16_LE",
                "-r",
                "48000",
                "-c",
                "2",
                "-d",
                "1",
                "/dev/zero",
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok_and(|s| s.success());
        if !ready {
            return error("Could not open USB audio playback");
        }
    }
    let mut result = if bluetooth_available || root.join("Config/bluetooth-headphones").exists() {
        if route.is_some() {
            run(root, "usb", route.clone())
        } else {
            run(root, "speaker", None)
        }
    } else {
        ResultMessage {
            devices: None,
            output: None,
            status: String::new(),
        }
    };
    if result.status.starts_with("Error:") {
        return result;
    }
    if let Some(route) = &route {
        if !crate::usb_audio::devices()
            .iter()
            .any(|d| d.route == *route)
        {
            return error("USB audio device was unplugged");
        }
    }
    if crate::usb_audio::write_route(route.as_deref().unwrap_or("")).is_err() {
        return error("Could not select audio output");
    }
    result.output = Some(route.unwrap_or_else(|| "speaker".into()));
    result.status.clear();
    result
}

pub fn fallback_to_local(root: PathBuf, address: String) {
    std::thread::spawn(move || {
        let result = run(root, "fallback", Some(address));
        if result.status.starts_with("Error:") {
            eprintln!("slot: {}", result.status);
        }
    });
}

pub fn disconnect_volume(root: &std::path::Path) -> u8 {
    std::fs::read_to_string(root.join("Config/sound.conf"))
        .ok()
        .and_then(|s| {
            s.lines()
                .find_map(|l| l.strip_prefix("disconnect_volume=")?.parse::<u8>().ok())
        })
        .filter(|v| *v <= 100)
        .unwrap_or(0)
}

pub fn pause_on_disconnect(root: &std::path::Path) -> bool {
    std::fs::read_to_string(root.join("Config/sound.conf")).ok()
        .is_some_and(|text| text.lines().any(|line| line == "pause_on_disconnect=1"))
}

fn save_sound_value(root: &std::path::Path, key: &str, value: &str) -> std::io::Result<()> {
    let path = root.join("Config/sound.conf");
    let previous = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e),
    };
    let mut text: String = previous.lines().filter(|line| line.split_once('=').is_none_or(|(k, _)| k != key))
        .map(|line| format!("{line}\n")).collect();
    text.push_str(&format!("{key}={value}\n"));
    std::fs::create_dir_all(root.join("Config"))?;
    slot_store::atomic_write(&path, text.as_bytes())
}

fn save_disconnect_volume(root: &std::path::Path, value: u8) -> std::io::Result<()> {
    save_sound_value(root, "disconnect_volume", &value.min(100).to_string())
}

fn run(root: PathBuf, verb: &str, address: Option<String>) -> ResultMessage {
    run_cancel(root, verb, address, None)
}

#[cfg(feature = "device")]
fn run_cancel(root: PathBuf, verb: &str, address: Option<String>, cancel: Option<&std::path::Path>) -> ResultMessage {
    let mut command = std::process::Command::new("timeout");
    command
        .args(["-k", "5", if verb == "connect" {"60"} else {"90"}])
        .arg("/bin/sh")
        .arg(root.join("System/bluetooth.sh"))
        .arg(verb)
        .env("SLOT_ROOT", &root)
        .env("SLOT_BT_LOCK_TRIES", "12");
    if let Some(cancel) = cancel { command.env("SLOT_BT_CANCEL_FILE", cancel); }
    if let Some(address) = address {
        command.arg(address);
    }
    let result = command.output();
    let fallback = match &result {
        Ok(o)
            if !matches!(verb, "restore" | "reconnect")
                && !o.status.success()
                && !String::from_utf8_lossy(&o.stderr)
                    .contains("REPORT Bluetooth/latest-error.txt") =>
        {
            Some(format!(
                "Operation: {verb}\nExit: {}\nSTDOUT:\n{}\nSTDERR:\n{}",
                o.status,
                String::from_utf8_lossy(&o.stdout),
                String::from_utf8_lossy(&o.stderr)
            ))
        }
        Err(e) if !matches!(verb, "restore" | "reconnect") => Some(format!(
            "Operation: {verb}\nCould not start Bluetooth helper: {e}"
        )),
        _ => None,
    };
    if let Some(mut report) = fallback {
        if let Ok(mut file) = std::fs::File::open(root.join("Logs/slot.log")) {
            use std::io::{Read, Seek, SeekFrom};
            let size = file.metadata().map_or(0, |m| m.len());
            let _ = file.seek(SeekFrom::Start(size.saturating_sub(65536)));
            let mut bytes = Vec::new();
            let _ = file.take(65536).read_to_end(&mut bytes);
            report.push_str("\n--- Application log ---\n");
            report.push_str(&String::from_utf8_lossy(&bytes));
        }
        let _ = std::fs::create_dir_all(root.join("Bluetooth"));
        let _ = std::fs::write(
            root.join("Bluetooth/latest-error.txt"),
            &report.as_bytes()[..report.len().min(131072)],
        );
    }
    match result {
        Ok(output) if output.status.success() => ResultMessage {
            devices: String::from_utf8_lossy(&output.stdout)
                .lines()
                .any(|l| l == "DEVICES")
                .then(|| parse_devices(&String::from_utf8_lossy(&output.stdout))),
            output: String::from_utf8_lossy(&output.stdout)
                .lines()
                .find_map(|line| line.strip_prefix("OUTPUT ").map(str::to_owned)),
            status: match verb {
                "connect" => "Device connected",
                "speaker" => "Speaker selected",
                "off" => "Bluetooth off",
                "on" => "Bluetooth on",
                "forget" => "Device forgotten",
                "diagnose" => "Saved Bluetooth/latest-report.txt",
                "scan" => "Scan complete.",
                _ => "Select a menu to continue",
            }
            .into(),
        },
        Ok(output) => ResultMessage {
            devices: None,
            output: None,
            status: format!(
                "Error: {}",
                String::from_utf8_lossy(&output.stderr)
                    .lines()
                    .rfind(|s| !s.starts_with("REPORT ") && !s.is_empty())
                    .unwrap_or("Bluetooth failed or timed out")
            ),
        },
        Err(error) => ResultMessage {
            devices: None,
            output: None,
            status: format!("Error: {error}"),
        },
    }
}

#[cfg(not(feature = "device"))]
fn run_cancel(_root: PathBuf, _verb: &str, _address: Option<String>, _cancel: Option<&std::path::Path>) -> ResultMessage {
    ResultMessage {
        devices: None,
        output: None,
        status: "Error: Bluetooth requires the handheld".into(),
    }
}

pub fn screen_face(lines: &[String]) -> UndoFace {
    screen_face_with_ink(lines, slot_ui::footer_ink(false))
}

pub fn screen_face_with_ink(lines: &[String], footer_ink: [u8; 3]) -> UndoFace {
    screen_face_on(lines,footer_ink,slot_ui::Canvas::FRAMED)
}

pub fn screen_face_on(lines: &[String], footer_ink: [u8; 3], canvas:slot_ui::Canvas) -> UndoFace {render_face(lines,footer_ink,canvas,true)}
pub fn screen_face_without_switches(lines: &[String], footer_ink: [u8;3], canvas:slot_ui::Canvas) -> UndoFace {
    let mut lines=lines.to_vec();
    if let Some(message)=lines.get_mut(1) {
        if message != "CONFIRM_FORGET" { message.clear(); }
    }
    render_face(&lines,footer_ink,canvas,false)
}
fn render_face(lines: &[String], footer_ink: [u8;3], canvas:slot_ui::Canvas, switches: bool) -> UndoFace {
    let screen_w=canvas.w.round() as u32;
    let rgba = vec![0u8; (screen_w * OUT_H * 4) as usize];
    let Some(font) = slot_ui::text::label_font() else {
        return UndoFace {
            rgba,
            w: screen_w,
            h: OUT_H,
        };
    };
    let label_w = screen_w - 2 * slot_ui::QUICK_LABEL_X as u32;
    let compact = lines.get(2).is_some_and(|line| line == "COMPACT");
    let clean = lines.iter().take(3).all(String::is_empty);
    let row_count = lines.len().saturating_sub(4) as u32;
    // Reserve six row positions so status text and new scan results cannot
    // recenter the menu or change the visible window.
    let row_top = if compact {
        88
    } else if clean {
        (OUT_H - slot_ui::MOUTH_H as u32 - slot_ui::MENU_HINT_H - 12 - row_count * slot_ui::QUICK_PITCH as u32) / 2 + 4
    } else {
        184
    };
    let selected = lines.iter().skip(3).position(|line|line.starts_with("SELECTED "));
    let mut panel = UndoFace { rgba, w: screen_w, h: OUT_H };
    slot_ui::expressive::raster_menu_rows(&mut panel, row_top as f32, slot_ui::QUICK_PITCH, row_count as usize, selected);
    let mut rgba = panel.rgba;
    if compact { let title=slot_ui::quick_heading_face("Sound"); blit(&mut rgba,&title,slot_ui::QUICK_LABEL_X as u32 - slot_ui::MENU_PAD,40); }
    for (i, line) in lines.iter().enumerate() {
        if compact && (i == 0 || i == 2) {
            continue;
        }
        let (label, top, h, px, max_lines, highlight) = match i {
            1 if compact => (if line.starts_with("Error:") { "Error" } else { line.as_str() }, 346, 26, 16.0, 1, false),
            0 => (line.as_str(), 16, 46, 30.0, 1, false),
            1 => (line.as_str(), 66, 82, 22.0, 3, false),
            2 => (line.as_str(), 146, 30, 20.0, 1, false),
            _ if i == lines.len() - 1 => (line.as_str(), 427, 38, 22.0, 1, false),
            _ => (
                line.strip_prefix("SELECTED ")
                    .or_else(|| line.strip_prefix("ROW "))
                    .unwrap_or(line),
                row_top + (i as u32 - 3) * slot_ui::QUICK_PITCH as u32 + 1,
                40u32,
                26.0,
                1,
                line.starts_with("SELECTED "),
            ),
        };
        let row = i >= 3 && i != lines.len() - 1;
        let ink = if row && !highlight {
            [0x9a, 0x9a, 0xa4]
        } else {
            [246, 244, 239]
        };
        if i == lines.len() - 1 {
            let faces = slot_ui::sound_button_legends(footer_ink);
            let left: Vec<_> = [line.contains("B Back").then_some(0)]
                .into_iter()
                .flatten()
                .collect();
            let right: Vec<_> = [
                line.contains("Y Forget").then_some(2),
                Some(if line.contains("Change") {
                    4
                } else if line.contains("A Connect") {
                    1
                } else {
                    3
                }),
            ]
            .into_iter()
            .flatten()
            .collect();
            for (indices, right) in [(&left, false), (&right, true)] {
                let seen = |i: usize| faces[i].w.saturating_sub(slot_ui::HINT_EDGE);
                let gap = 16;
                let total = indices.iter().map(|&i| seen(i)).sum::<u32>()
                    + gap * indices.len().saturating_sub(1) as u32;
                let mut x = if right {
                    screen_w.saturating_sub(16 + total)
                } else {
                    16
                };
                for &i in indices {
                    blit(&mut rgba, &faces[i], x, OUT_H - slot_ui::MOUTH_H as u32 - slot_ui::MENU_HINT_H - 12);
                    x += seen(i) + gap;
                }
            }
            continue;
        }
        if label == "CONFIRM_FORGET" {
            let hint = slot_ui::sound_forget_confirmation();
            blit(&mut rgba, &hint, (screen_w - hint.w) / 2, 12);
            continue;
        }
        if let Some((name, value)) = label
            .strip_prefix("DISCONNECT ")
            .map(|v| ("Volume on disconnect", v))
            .or_else(|| label.strip_prefix("VOLUME ").map(|v| ("Volume", v)))
            .or_else(|| label.strip_prefix("PAUSE_DISCONNECT ").map(|v| ("Pause on disconnect", v)))
            .or_else(|| label.strip_prefix("BLUETOOTH ").map(|v| ("Bluetooth", v)))
        {
            let name = slot_ui::quick_value_face(name, highlight);
            blit(&mut rgba, &name, slot_ui::QUICK_LABEL_X as u32 - slot_ui::MENU_PAD, top);
            if label.starts_with("BLUETOOTH ") || label.starts_with("PAUSE_DISCONNECT ") {
                if !switches {continue;}
                let switch = slot_ui::toggle_face(value == "On", highlight);
                let x = screen_w - slot_ui::QUICK_EDGE as u32 + slot_ui::MENU_PAD - switch.w;
                blit(&mut rgba, &switch, x, top);
                continue;
            }
            if label.starts_with("VOLUME ") || label.starts_with("DISCONNECT ") {
                if switches {let slider=slot_ui::slider_face_with_width(value.trim_end_matches('%').parse::<f32>().unwrap_or(0.0)/100.0,112.0);blit(&mut rgba,&slider,screen_w-slot_ui::QUICK_EDGE as u32-200-slot_ui::MENU_PAD,top);}
                let value=slot_ui::quick_value_face(value,highlight);blit(&mut rgba,&value,screen_w-slot_ui::QUICK_EDGE as u32+slot_ui::MENU_PAD-value.w,top);continue;
            }
            let value = slot_ui::quick_value_face(value, highlight);
            if highlight {
                let left = slot_ui::quick_caret_face(false);
                let right = slot_ui::quick_caret_face(true);
                let rx = screen_w - slot_ui::QUICK_EDGE as u32 - right.w;
                let vx = rx - 14 + 18 - value.w;
                blit(&mut rgba, &right, rx, top);
                blit(&mut rgba, &value, vx, top);
                blit(&mut rgba, &left, vx + 18 - 14 - left.w, top);
            } else {
                // Align visible ink, not the font's trailing side bearing.
                let ink_right = value
                    .rgba
                    .chunks_exact(4)
                    .enumerate()
                    .filter(|(_, p)| p[3] > 127)
                    .map(|(i, _)| i as u32 % value.w)
                    .max()
                    .unwrap_or(0);
                blit(&mut rgba, &value, screen_w - slot_ui::QUICK_EDGE as u32 - 1 - ink_right, top);
            }
            continue;
        }
        let active = label.starts_with("ACTIVE ");
        let label = label.strip_prefix("ACTIVE ").unwrap_or(label);
        let usb = label.starts_with("USB_DEVICE ");
        let wired = label.starts_with("WIRED_DEVICE ");
        let speaker = label.starts_with("SPEAKER_DEVICE ");
        let device = usb || wired || speaker || label.starts_with("DEVICE ");
        let label = label
            .strip_prefix("DEVICE ")
            .or_else(|| label.strip_prefix("USB_DEVICE "))
            .or_else(|| label.strip_prefix("WIRED_DEVICE "))
            .or_else(|| label.strip_prefix("SPEAKER_DEVICE "))
            .unwrap_or(label);
        let inset = if !row {
            48
        } else if device {
            slot_ui::QUICK_LABEL_X as u32 + 36
        } else {
            slot_ui::QUICK_LABEL_X as u32
        };
        let label_w = label_w - if device { 36 } else { 0 };
        if device {
            let icon = slot_ui::sound_device_icon(
                if speaker {
                    slot_ui::Icon::Volume
                } else if usb || wired {
                    slot_ui::Icon::Headphones
                } else {
                    slot_ui::Icon::Bluetooth
                },
                ink,
            );
            let center = slot_ui::quick_text_centre();
            blit(
                &mut rgba,
                &icon,
                slot_ui::QUICK_LABEL_X as u32 + 28u32.saturating_sub(icon.w) / 2,
                top + (center - icon.h as f32 / 2.).round().max(0.) as u32,
            );
        }
        let role = if row {
            if highlight { slot_ui::text::Role::Focus } else { slot_ui::text::Role::Menu }
        } else { slot_ui::text::Role::Detail };
        let font = slot_ui::text::font(role).unwrap_or(font);
        let layout = slot_ui::text::fit_preserving_case(
            font,
            label,
            (label_w - if active { 44 } else { 0 }) as f32,
            max_lines,
            px,
            18.0,
        );
        let row = i >= 3 && i != lines.len() - 1;
        let face_w = if row {
            layout
                .lines
                .first()
                .map(|l| {
                    (slot_ui::text::line_width(font, l, layout.px, layout.tracking).ceil() as u32
                        + 2)
                    .min(label_w)
                })
                .unwrap_or(label_w)
        } else {
            label_w
        };
        if active {
            draw_check(&mut rgba, screen_w - slot_ui::QUICK_EDGE as u32 - 28, top + h / 2, ink);
        }
        let mut face = vec![0u8; (face_w * h * 4) as usize];
        slot_ui::text::draw_centred(&mut face, face_w, h, &layout, ink);
        blit(
            &mut rgba,
            &UndoFace {
                rgba: face,
                w: face_w,
                h,
            },
            inset,
            top,
        );
    }
    UndoFace {
        rgba,
        w: screen_w,
        h: OUT_H,
    }
}

fn blit(dst: &mut [u8], face: &UndoFace, left: u32, top: u32) {
    let screen_w=dst.len() as u32/(OUT_H*4);
    for y in 0..face.h.min(OUT_H.saturating_sub(top)) {
        for x in 0..face.w.min(screen_w.saturating_sub(left)) {
            let src = ((y * face.w + x) * 4) as usize;
            let dest = (((top + y) * screen_w + left + x) * 4) as usize;
            let a = u32::from(face.rgba[src + 3]);
            let behind = u32::from(dst[dest + 3]) * (255 - a) / 255;
            let alpha = a + behind;
            if alpha == 0 {
                continue;
            }
            for c in 0..3 {
                dst[dest + c] = ((u32::from(face.rgba[src + c]) * a
                    + u32::from(dst[dest + c]) * behind)
                    / alpha) as u8;
            }
            dst[dest + 3] = alpha as u8;
        }
    }
}

fn draw_check(dst: &mut [u8], x: u32, y: u32, ink: [u8; 3]) {
    let screen_w=dst.len() as u32/(OUT_H*4);
    // Draw the mark directly: the original slot label font has no check glyph.
    for i in 0..10 {
        for thickness in 0..3 {
            let offset = (((y - 3 + i + thickness) * screen_w + x + i) * 4) as usize;
            dst[offset..offset + 4].copy_from_slice(&[ink[0], ink[1], ink[2], 255]);
        }
    }
    for i in 0..19 {
        for thickness in 0..3 {
            let offset = (((y + 6 - i + thickness) * screen_w + x + 9 + i) * 4) as usize;
            dst[offset..offset + 4].copy_from_slice(&[ink[0], ink[1], ink[2], 255]);
        }
    }
}

#[cfg(test)]
mod menu_tests {
    use super::*;
    fn menu() -> Manager {
        Manager {
            root: PathBuf::new(),
            bluetooth_available: true,
            disconnect_volume: 0,
            current_volume:50,
            pause_on_disconnect: false,
            page: Page::Home,
            stack: Vec::new(),
            usb_devices: Vec::new(),
            devices: vec![Headphones {
                address: "AA:BB:CC:DD:EE:FF".into(),
                name: "WF-C510".into(),
                paired: true,
                connected: false,
            }],
            selected: 0,
            status: String::new(),
            output: "speaker".into(),
            pending: None,
            recovering: Arc::default(),
            cancel_path: None,
            face: None,
            progress_face: None,
            wired: false,
            disconnect_notice: false,
            route_choice: None,
            choosing_route: false,
            forget_confirmation: None,
        }
    }
    #[test]
    fn backing_out_cancels_work_and_keeps_the_sound_menu_responsive() {
        let dir=tempfile::tempdir().unwrap();
        let path=dir.path().join("cancel");
        let mut m=menu();
        let (_tx,rx)=mpsc::channel();
        m.pending=Some(rx); m.cancel_path=Some(path.clone());
        m.page=Page::Pair; m.stack.push(Page::Home);
        m.input(Action::GbaDown(Btn::B));
        assert!(!m.busy()); assert!(path.exists()); assert_eq!(m.page,Page::Home);
        let selected=m.selected;
        m.input(Action::GbaDown(Btn::Down));
        assert_ne!(m.selected,selected);
    }

    #[test]
    fn offline_connect_does_not_restart_the_stack_but_radio_errors_can_recover() {
        let mut calls = Vec::new();
        let mut announced = false;
        let result = run_with_recovery("connect", Some("AA:BB:CC:DD:EE:FF".into()),
            || announced = true, |verb, address| {
                calls.push((verb.to_owned(), address));
                ResultMessage { devices: None, output: None,
                    status: if calls.len() == 1 { "Error: failed" } else { "Connected" }.into() }
            });
        assert!(!announced);
        assert_eq!(calls, vec![("connect".into(), Some("AA:BB:CC:DD:EE:FF".into()))]);
        assert!(result.status.starts_with("Error:"));
        let mut calls = Vec::new();
        let result = run_with_recovery("on", None, || {}, |verb, _| {
            calls.push(verb.to_owned());
            ResultMessage { devices: None, output: None, status: "Error: unavailable".into() }
        });
        assert_eq!(calls, ["on", "restart"]);
        assert!(result.status.starts_with("Error:"));
        calls.clear();
        run_with_recovery("connect", None, || {}, |verb, _| {
            calls.push(verb.to_owned());
            ResultMessage { devices: None, output: None,
                status: if verb == "restart" { "Ready" } else { "Error: unavailable" }.into() }
        });
        assert_eq!(calls, ["connect"]);
    }

    #[test]
    fn recovery_shows_only_error_and_never_exposes_helper_details() {
        let mut m = menu();
        m.status = "Error: org.bluez.Error.Failed, secret debug detail".into();
        assert_eq!(m.lines()[1], "Error");
        assert!(!m.lines().iter().any(|l| l.contains("org.bluez") || l.contains("debug detail")));
        m.recovering.store(true, Ordering::Relaxed);
        m.status.clear();
        assert_eq!(m.lines()[1], "Error");
    }

    #[test]
    fn busy_work_uses_progress_without_hiding_failures_or_moving_rows() {
        let mut m = menu();
        let idle = m.lines();
        let (tx, rx) = mpsc::channel();
        m.pending = Some(rx);
        for status in [
            "Scanning…",
            "Connecting...",
            "Turning Bluetooth on...",
            "Saving report...",
        ] {
            m.status = status.into();
            let busy = m.lines();
            assert!(busy[1].is_empty());
            assert_eq!(&busy[3..], &idle[3..]);
        }
        tx.send(ResultMessage {
            devices: None,
            output: None,
            status: "Error: Bluetooth could not connect".into(),
        })
        .unwrap();
        m.poll();
        assert!(!m.busy());
        assert_eq!(m.lines()[1], "Error");
        assert_eq!(&m.lines()[3..], &idle[3..]);
    }

    #[test]
    fn opening_sound_adopts_an_automatic_usb_switch() {
        let mut worker = menu();
        let (tx, rx) = mpsc::channel();
        worker.pending = Some(rx);
        worker.choosing_route = true;
        worker.status = "Switching output...".into();
        let mut visible = menu();
        visible.adopt_usb_switch(&mut worker);
        assert!(visible.busy());
        assert!(!worker.busy());
        tx.send(ResultMessage {
            devices: None,
            output: Some("plughw:CARD=Audio,DEV=0".into()),
            status: String::new(),
        })
        .unwrap();
        visible.poll();
        assert_eq!(
            visible.route_choice.as_deref(),
            Some("plughw:CARD=Audio,DEV=0")
        );
        assert!(!visible.busy());
        assert_eq!(
            visible.devices.len(),
            1,
            "an output-only reply must preserve paired devices"
        );
    }

    #[test]
    fn an_authoritative_empty_list_removes_forgotten_devices() {
        let mut m = menu();
        let (tx, rx) = mpsc::channel();
        m.pending = Some(rx);
        tx.send(ResultMessage {
            devices: Some(vec![]),
            output: None,
            status: "Device forgotten".into(),
        })
        .unwrap();
        m.poll();
        assert!(m.devices.is_empty());
    }

    #[test]
    fn forget_confirmation_cancels_on_navigation_and_x_remains_hidden() {
        let mut m = menu();
        m.selected = m.device_start();
        assert!(m.lines().last().unwrap().contains("A Connect · Y Forget"));
        assert!(!m.lines().last().unwrap().contains("X"));
        m.input(Action::GbaDown(Btn::Y));
        m.input(Action::GbaDown(Btn::Down));
        assert!(m.forget_confirmation.is_none());
        m.input(Action::GbaDown(Btn::Up));
        m.input(Action::GbaDown(Btn::Y));
        assert!(m.pending.is_none(), "moving away disarms the first press");
        m.input(Action::GbaDown(Btn::A));
        assert_eq!(m.status, "Connecting...");
        assert!(m.forget_confirmation.is_none());
        let mut m = menu();
        m.input(Action::GbaDown(Btn::X));
        assert_eq!(m.status, "Saving report...");
    }

    #[test]
    fn usb_outputs_appear_without_bluetooth_and_show_selected_output() {
        let mut m = menu();
        m.bluetooth_available = false;
        m.usb_devices = vec![crate::usb_audio::Device {
            route: "plughw:CARD=Audio,DEV=0".into(),
            name: "AB13X Audio".into(),
        }];
        assert_eq!(
            m.rows(),
            vec![
                "VOLUME 50%",
                "ACTIVE SPEAKER_DEVICE Internal speaker",
                "USB_DEVICE AB13X Audio",
                "DISCONNECT 0%",
                "PAUSE_DISCONNECT Off"
            ]
        );
        m.observe_output(Some("plughw:CARD=Audio,DEV=0"), true);
        assert_eq!(m.rows()[1], "SPEAKER_DEVICE Internal speaker");
        assert!(m.rows()[2].starts_with("ACTIVE USB"));
        assert_eq!(m.disconnect_row(), 3);
    }

    #[test]
    fn bluetooth_defaults_off_and_remembers_explicit_enable_or_existing_headphones() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("Config")).unwrap();
        assert_eq!(startup_verb(root.path()), "off");
        std::fs::write(root.path().join("Config/bluetooth-enabled"), "").unwrap();
        assert_eq!(startup_verb(root.path()), "on");
        std::fs::remove_file(root.path().join("Config/bluetooth-enabled")).unwrap();
        std::fs::write(
            root.path().join("Config/bluetooth-headphones"),
            "AA:BB:CC:DD:EE:FF",
        )
        .unwrap();
        assert_eq!(startup_verb(root.path()), "restore");
        std::fs::write(root.path().join("Config/bluetooth-off"), "").unwrap();
        assert_eq!(startup_verb(root.path()), "off");
    }

    #[test]
    fn pair_entry_enables_bluetooth_and_toggle_has_change_hint() {
        let root = tempfile::tempdir().unwrap();
        let mut m = menu();
        m.root = root.path().to_owned();
        m.selected = 1;
        assert!(m.lines().last().unwrap().contains("Change"));
        m.selected = m.disconnect_row() - 1;
        m.input(Action::GbaDown(Btn::A));
        assert_eq!(m.page, Page::Pair);
        assert!(m.busy());
        assert_eq!(m.status, "Turning Bluetooth on...");
    }

    #[test]
    fn off_is_available_only_with_bluetooth_and_reflects_the_saved_state() {
        let root = tempfile::tempdir().unwrap();
        let mut m = menu();
        m.root = root.path().to_owned();
        assert_eq!(m.rows().get(1).unwrap(), "BLUETOOTH Off");
        std::fs::create_dir(root.path().join("Config")).unwrap();
        std::fs::write(root.path().join("Config/bluetooth-off"), "").unwrap();
        assert_eq!(m.rows().get(1).unwrap(), "BLUETOOTH Off");
        m.bluetooth_available = false;
        assert!(!m.rows().iter().any(|r| r.starts_with("BLUETOOTH ")));
    }

    #[test]
    fn reconnect_is_slow_bounded_and_respects_off_and_manual_output() {
        assert_eq!(
            [0, 1, 2, 3, 100].map(|n| retry_delay(n).as_secs()),
            [5, 15, 30, 60, 60]
        );
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("System")).unwrap();
        std::fs::write(root.path().join("System/bluetooth.sh"), "").unwrap();
        std::fs::create_dir(root.path().join("Config")).unwrap();
        let mut retry = Reconnect::default();
        retry.checked = Some(std::time::Instant::now() - std::time::Duration::from_secs(100));
        retry.poll(root.path(), false);
        assert!(retry.pending.is_none(), "no selection means no reconnect");
        std::fs::write(
            root.path().join("Config/bluetooth-headphones"),
            "AA:BB:CC:DD:EE:FF",
        )
        .unwrap();
        std::fs::write(root.path().join("Config/bluetooth-off"), "").unwrap();
        retry.checked = Some(std::time::Instant::now() - std::time::Duration::from_secs(100));
        retry.poll(root.path(), false);
        assert!(retry.pending.is_none(), "off overrides a stale selection");
        retry.failures = 10;
        retry.poll(root.path(), true);
        assert_eq!(retry.failures, 0);
    }

    #[test]
    fn pairing_is_separate_and_back_restores_the_parent() {
        let mut m = menu();
        m.devices[0].paired = false;
        m.selected = m.pair_row();
        m.input(Action::GbaDown(Btn::A));
        assert_eq!(m.page, Page::Pair);
        m.input(Action::GbaDown(Btn::B));
        assert_eq!(m.page, Page::Home);
        assert!(m.input(Action::GbaDown(Btn::B)));
    }
    #[test]
    fn saved_headphones_are_on_home_and_forget_requires_two_y_presses() {
        let mut m = menu();
        assert_eq!(
            m.rows(),
            vec![
                "VOLUME 50%",
                "BLUETOOTH Off",
                "ACTIVE SPEAKER_DEVICE Internal speaker",
                "DEVICE WF-C510",
                "Pair audio device",
                "DISCONNECT 0%",
                "PAUSE_DISCONNECT Off"
            ]
        );
        assert!(m.lines()[0].is_empty());
        assert!(m.lines()[1].is_empty());
        let idle = screen_face(&m.lines());
        for status in ["Connecting...", "Scanning…", "Error: Connection failed"] {
            m.status = status.into();
            let busy = screen_face(&m.lines());
            assert_eq!(
                &idle.rgba[(88 * OUT_W * 4) as usize..(340 * OUT_W * 4) as usize],
                &busy.rgba[(88 * OUT_W * 4) as usize..(340 * OUT_W * 4) as usize],
                "status must not move or redraw the menu rows or hints"
            );
        }
        m.status.clear();
        m.selected = m.device_start();
        m.input(Action::GbaDown(Btn::Y));
        assert_eq!(m.page, Page::Home);
        assert_eq!(m.selected, m.device_start());
        assert!(m.forget_confirmation.is_some());
        assert!(m.lines()[1].contains("CONFIRM_FORGET"));
        assert!(m.pending.is_none());
        m.input(Action::GbaDown(Btn::B));
        assert_eq!(m.page, Page::Home);
        assert!(m.forget_confirmation.is_none());
        m.input(Action::GbaDown(Btn::Y));
        m.input(Action::GbaDown(Btn::Y));
        assert_eq!(m.page, Page::Home);
        assert!(m.pending.is_some());
        assert_eq!(m.status, "Forgetting device...");
    }
    #[test]
    fn active_route_not_connection_alone_controls_checkmark() {
        let mut m = menu();
        m.devices[0].connected = true;
        assert_eq!(m.rows()[m.local_row()], "ACTIVE SPEAKER_DEVICE Internal speaker");
        m.output = m.devices[0].address.clone();
        assert_eq!(m.rows()[m.local_row()], "SPEAKER_DEVICE Internal speaker");
        assert_eq!(m.rows()[m.device_start()], "ACTIVE DEVICE WF-C510");
        m.devices[0].connected = false;
        assert_eq!(m.rows()[m.local_row()], "SPEAKER_DEVICE Internal speaker");
    }
    #[test]
    #[cfg(feature = "device")]
    fn saved_headphones_in_scan_results_connect_directly() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("System")).unwrap();
        std::fs::write(
            root.path().join("System/bluetooth.sh"),
            "printf '%s %s' \"$1\" \"$2\" > \"$SLOT_ROOT/called\"\n",
        )
        .unwrap();
        let mut m = menu();
        m.root = root.path().into();
        m.open(Page::Pair);
        assert_eq!(m.rows(), vec!["Scan for devices", "DEVICE WF-C510"]);
        m.selected = 1;
        m.input(Action::GbaDown(Btn::A));
        assert_eq!(m.page, Page::Pair);
        assert_eq!(m.status, "Connecting...");
        m.pending
            .take()
            .unwrap()
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();
        assert_eq!(
            std::fs::read_to_string(root.path().join("called")).unwrap(),
            "connect AA:BB:CC:DD:EE:FF"
        );
    }

    #[test]
    fn connected_device_returns_to_sound_and_is_selected_and_checked() {
        let mut m = menu();
        m.open(Page::Pair);
        let (tx, rx) = mpsc::channel();
        m.pending = Some(rx);
        m.choosing_route = true;
        let mut devices = m.devices.clone();
        devices[0].connected = true;
        tx.send(ResultMessage {
            output: Some(devices[0].address.clone()),
            devices: Some(devices),
            status: "Device connected".into(),
        })
        .unwrap();
        m.poll();
        assert_eq!(m.page, Page::Home);
        assert!(m.stack.is_empty());
        assert_eq!(m.selected, m.device_start());
        assert_eq!(m.rows()[m.device_start()], "ACTIVE DEVICE WF-C510");
        if let Ok(path) = std::env::var("SLOT_SOUND_PREVIEW") {
            let face = screen_face(&m.lines());
            let file = std::fs::File::create(path).unwrap();
            let mut encoder = png::Encoder::new(file, face.w, face.h);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .unwrap()
                .write_image_data(&face.rgba)
                .unwrap();
        }
        assert!(m.lines()[0].is_empty());
        assert!(m.lines()[1].is_empty());
        assert!(m.input(Action::GbaDown(Btn::B)));
    }

    #[test]
    fn failed_connection_stays_on_scan_and_preserves_output() {
        let mut m = menu();
        m.open(Page::Pair);
        let (tx, rx) = mpsc::channel();
        m.pending = Some(rx);
        m.choosing_route = true;
        tx.send(ResultMessage {
            output: None,
            devices: None,
            status: "Error: unavailable".into(),
        })
        .unwrap();
        m.poll();
        assert_eq!(m.page, Page::Pair);
        assert_eq!(m.output, "speaker");
        assert!(m.route_choice.is_none());
    }

    #[test]
    fn local_output_follows_jack_and_does_not_claim_bluetooth_output() {
        let mut m = menu();
        m.wired = true;
        assert_eq!(m.rows()[m.local_row()], "ACTIVE WIRED_DEVICE Wired headphones");
        m.observe_output(Some("AA:BB:CC:DD:EE:FF"), true);
        assert_eq!(m.rows()[m.local_row()], "WIRED_DEVICE Wired headphones");
        assert_eq!(m.rows()[m.device_start()], "ACTIVE DEVICE WF-C510");
        m.wired = false;
        assert_eq!(m.rows()[m.local_row()], "SPEAKER_DEVICE Internal speaker");
        m.observe_output(None, false);
        assert_eq!(m.rows()[m.local_row()], "ACTIVE SPEAKER_DEVICE Internal speaker");
        m.open(Page::Pair);
        assert!(m.lines()[0].is_empty());
        assert!(m.lines()[1].is_empty());
        assert_eq!(m.rows()[0], "Scan for devices");
    }

    #[test]
    fn local_sound_settings_do_not_expose_bluetooth_when_unavailable() {
        let root = tempfile::tempdir().unwrap();
        let mut m = Manager::new(root.path().into(), false);
        assert_eq!(
            m.rows(),
            vec!["VOLUME 50%","ACTIVE SPEAKER_DEVICE Internal speaker", "DISCONNECT 0%", "PAUSE_DISCONNECT Off"]
        );
        assert!(m.pending.is_none());
        m.selected = m.disconnect_row();
        assert!(m.slider_selected());
        assert_eq!(m.visible_sliders().len(),2);
        assert!(m.visible_sliders().iter().any(|s|s.fraction==0.0));
        for _ in 0..25 {
            m.input(Action::GbaDown(Btn::Right));
        }
        assert_eq!(m.disconnect_volume, 100);
        assert_eq!(disconnect_volume(root.path()), 100);
        for _ in 0..25 {
            m.input(Action::GbaDown(Btn::Left));
        }
        assert_eq!(m.disconnect_volume, 0);
        assert_eq!(disconnect_volume(root.path()), 0);
        m.input(Action::GbaDown(Btn::Right));
        assert_eq!(Manager::new(root.path().into(), false).disconnect_volume, 5);
        std::fs::write(
            root.path().join("Config/sound.conf"),
            "disconnect_volume=999",
        )
        .unwrap();
        assert_eq!(disconnect_volume(root.path()), 0);
    }

    #[test]
    fn diagnostic_results_preserve_devices_and_route() {
        let mut m = menu();
        let (tx, rx) = mpsc::channel();
        m.pending = Some(rx);
        m.output = m.devices[0].address.clone();
        tx.send(ResultMessage {
            devices: None,
            output: None,
            status: "Saved Bluetooth/latest-report.txt".into(),
        })
        .unwrap();
        m.poll();
        assert_eq!(m.devices.len(), 1);
        assert_eq!(m.output, m.devices[0].address);
    }
}

/// A passive BlueZ signal listener. It never powers the adapter or starts discovery.
/// Signals only invalidate cached state; the helper still verifies the saved device.
#[derive(Default)]
pub struct Events {
    child: Option<std::process::Child>,
    incoming: Option<Receiver<()>>,
    attempted: Option<std::time::Instant>,
    selection: String,
    sampled: Option<std::time::Instant>,
    asleep: bool,
}

impl Events {
    pub fn poll(&mut self, root: &std::path::Path, awake: bool) -> (bool, bool) {
        if !awake {
            if !self.asleep {
                self.stop();
                self.selection.clear();
                self.sampled = None;
                self.attempted = None;
                // Cancel only our automatic helper, never a manual pairing.
                let lock = std::env::var_os("SLOT_BT_LOCK")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("/run/slot-bluetooth.lock"));
                if std::fs::read_to_string(lock.join("verb"))
                    .is_ok_and(|v| matches!(v.trim(), "restore" | "reconnect"))
                {
                    let _ = std::fs::write(lock.join("cancel"), "");
                }
            }
            self.asleep = true;
            return (false, false);
        }
        self.asleep = false;
        if self.sampled.is_none_or(|t| t.elapsed().as_secs() >= 1) {
            self.sampled = Some(std::time::Instant::now());
            let selected = if !root.join("Config/bluetooth-off").exists() {
                std::fs::read_to_string(root.join("Config/bluetooth-headphones"))
                    .unwrap_or_default()
                    .trim()
                    .to_owned()
            } else {
                String::new()
            };
            if self.selection != selected {
                self.stop();
                self.selection = selected;
                self.attempted = None;
            }
        }
        if !valid_address(&self.selection) {
            return (false, false);
        }
        if self
            .child
            .as_mut()
            .is_some_and(|c| !matches!(c.try_wait(), Ok(None)))
        {
            self.stop();
        }
        if self.child.is_none() && self.attempted.is_none_or(|t| t.elapsed().as_secs() >= 60) {
            self.attempted = Some(std::time::Instant::now());
            self.start();
        }
        let changed = self
            .incoming
            .as_ref()
            .is_some_and(|rx| rx.try_recv().is_ok());
        (changed, self.child.is_some())
    }

    #[cfg(feature = "device")]
    fn start(&mut self) {
        use std::io::{BufRead, BufReader};
        use std::process::{Command, Stdio};
        // Accept any adapter, but only the selected device's connection events.
        let rule = "type='signal',sender='org.bluez',interface='org.freedesktop.DBus.Properties',member='PropertiesChanged',arg0='org.bluez.Device1'";
        let Ok(mut child) = Command::new("dbus-monitor")
            .args(["--system", rule])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
        else {
            return;
        };
        let stdout = child.stdout.take().unwrap();
        let address = self.selection.clone();
        let (tx, rx) = mpsc::sync_channel(1);
        std::thread::spawn(move || {
            let mut selected = false;
            for line in BufReader::new(stdout).lines() {
                let Ok(line) = line else { break };
                if connection_event(&line, &address, &mut selected) {
                    // Coalesce events; the authoritative property read follows.
                    let _ = tx.try_send(());
                }
            }
        });
        self.child = Some(child);
        self.incoming = Some(rx);
    }

    #[cfg(not(feature = "device"))]
    fn start(&mut self) {}

    fn stop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.incoming = None;
    }
}
impl Drop for Events {
    fn drop(&mut self) {
        self.stop();
    }
}

fn connection_event(line: &str, address: &str, selected: &mut bool) -> bool {
    if line.starts_with("signal ") {
        let component = format!("/dev_{};", address.replace(':', "_"));
        *selected = line
            .to_ascii_uppercase()
            .contains(&component.to_ascii_uppercase());
        false
    } else {
        *selected && line.trim() == "string \"Connected\""
    }
}

/// Retry only the saved choice. No discovery, new pairing, or UI-thread commands.
#[derive(Default)]
pub struct Reconnect {
    checked: Option<std::time::Instant>,
    pending: Option<Receiver<()>>,
    failures: u32,
}

impl Reconnect {
    pub fn changed(&mut self) {
        self.checked = Some(std::time::Instant::now() - std::time::Duration::from_secs(60));
    }
    pub fn poll(&mut self, root: &std::path::Path, connected: bool) {
        if let Some(rx) = &self.pending {
            if !matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)) {
                self.pending = None;
            }
        }
        let now = std::time::Instant::now();
        if connected {
            self.failures = 0;
            self.checked = Some(now);
            return;
        }
        let delay = retry_delay(self.failures);
        let Some(checked) = self.checked else {
            self.checked = Some(now);
            return;
        };
        if self.pending.is_some() || now.duration_since(checked) < delay {
            return;
        }
        self.checked = Some(now);
        if root.join("Config/bluetooth-off").exists()
            || !root.join("Config/bluetooth-headphones").is_file()
            || !root.join("System/bluetooth.sh").is_file()
        {
            self.failures = 0;
            return;
        }
        self.failures = self.failures.saturating_add(1);
        let root = root.to_owned();
        let (tx, rx) = mpsc::channel();
        self.pending = Some(rx);
        std::thread::spawn(move || {
            let _ = run(root, "reconnect", None);
            let _ = tx.send(());
        });
    }
}

fn retry_delay(failures: u32) -> std::time::Duration {
    std::time::Duration::from_secs(match failures {
        0 => 5,
        1 => 15,
        2 => 30,
        _ => 60,
    })
}

/// Read-only connection checks; never starts the radio or changes audio routing.
#[derive(Default)]
pub struct Indicator {
    checked: Option<std::time::Instant>,
    pending: Option<Receiver<(String, Option<bool>)>>,
    output: String,
    active: bool,
}

impl Indicator {
    pub fn changed(&mut self) {
        self.checked = None;
        self.pending = None;
    }

    pub fn select(&mut self, address: Option<String>) {
        self.active = address.is_some();
        self.output = address.unwrap_or_default();
        self.pending = None;
        self.checked = Some(std::time::Instant::now());
    }

    pub fn address(&self) -> Option<String> {
        (!self.output.is_empty()).then(|| self.output.clone())
    }
    pub fn poll(&mut self, _watching: bool) -> bool {
        if let Some(rx) = &self.pending {
            match rx.try_recv() {
                Ok((address, connected)) => {
                    if self.output == address {
                        if let Some(connected) = connected {
                            self.active = connected;
                        }
                    }
                    self.pending = None;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.pending = None;
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
        let path = std::env::var_os("SLOT_AUDIO_ROUTE")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/run/slot-audio-device"));
        let route = std::fs::read_to_string(path).unwrap_or_default();
        let address = route_address(&route).unwrap_or_default().to_owned();
        if address != self.output {
            self.active = false;
            self.output = address.clone();
            self.checked = None;
        }
        if address.is_empty() {
            self.active = false;
            return false;
        }
        // Keep a short read-only fallback even with a listener: a missed signal
        // must not leave the game on a disconnected output for 30 seconds.
        if self
            .checked
            .is_some_and(|t| t.elapsed() < std::time::Duration::from_secs(2))
        {
            return self.active;
        }
        self.checked = Some(std::time::Instant::now());
        if self.pending.is_none() {
            let (tx, rx) = mpsc::channel();
            self.pending = Some(rx);
            std::thread::spawn(move || {
                #[cfg(feature = "device")]
                let connected = std::process::Command::new("timeout")
                    .args(["2", "bluetoothctl", "info", &address])
                    .output()
                    .ok()
                    .and_then(|output| {
                        if !output.status.success() {
                            return None;
                        }
                        let info = String::from_utf8_lossy(&output.stdout);
                        if connection_present(&info) {
                            Some(true)
                        } else if info.lines().any(|l| l.trim() == "Connected: no") {
                            Some(false)
                        } else {
                            None
                        }
                    });
                #[cfg(not(feature = "device"))]
                let connected = Some(false);
                let _ = tx.send((address, connected));
            });
        }
        self.active
    }
}

fn route_address(route: &str) -> Option<&str> {
    let address = route
        .trim()
        .strip_prefix("bluealsa:DEV=")?
        .strip_suffix(",PROFILE=a2dp")?;
    valid_address(address).then_some(address)
}

#[cfg(any(feature = "device", test))]
fn connection_present(info: &str) -> bool {
    info.lines().any(|line| line.trim() == "Connected: yes")
}

pub fn indicator_face() -> UndoFace {
    indicator_face_with_ink([246, 244, 239])
}

pub fn indicator_face_with_ink(ink: [u8; 3]) -> UndoFace {
    let source=slot_ui::icon_face(slot_ui::Icon::Bluetooth,18.0,ink);
    let mut face=UndoFace{rgba:vec![0;20*20*4],w:20,h:20};
    let points:Vec<_>=source.rgba.chunks_exact(4).enumerate().filter(|(_,p)|p[3]>0).map(|(i,_)|(i as u32%source.w,i as u32/source.w)).collect();
    let left=points.iter().map(|p|p.0).min().unwrap_or(0);
    let top=points.iter().map(|p|p.1).min().unwrap_or(0);
    let w=(points.iter().map(|p|p.0).max().unwrap_or(0)-left+1).min(20);
    let h=(points.iter().map(|p|p.1).max().unwrap_or(0)-top+1).min(20);
    let (x,y)=((20-w)/2,(20-h)/2);
    for yy in 0..h {for xx in 0..w {
        let src=(((yy+top)*source.w+xx+left)*4)as usize;
        let dst=(((yy+y)*20+xx+x)*4)as usize;
        face.rgba[dst..dst+4].copy_from_slice(&source.rgba[src..src+4]);
    }}
    face
}

#[cfg(test)]
mod indicator_tests {
    use super::*;
    #[test]
    fn events_ignore_other_devices_and_unrelated_properties() {
        let address = "AA:BB:CC:DD:EE:FF";
        let mut selected = false;
        assert!(!connection_event(
            "signal path=/org/bluez/hci1/dev_AA_BB_CC_DD_EE_FF; interface=x",
            address,
            &mut selected
        ));
        assert!(!connection_event(
            "   string \"Alias\"",
            address,
            &mut selected
        ));
        assert!(connection_event(
            "   string \"Connected\"",
            address,
            &mut selected
        ));
        assert!(!connection_event(
            "signal path=/org/bluez/hci0/dev_00_11_22_33_44_55; interface=x",
            address,
            &mut selected
        ));
        assert!(!connection_event(
            "   string \"Connected\"",
            address,
            &mut selected
        ));
    }

    #[test]
    fn no_selection_off_and_sleep_do_not_start_a_listener() {
        let root = tempfile::tempdir().unwrap();
        let mut events = Events::default();
        assert_eq!(events.poll(root.path(), true), (false, false));
        std::fs::create_dir(root.path().join("Config")).unwrap();
        std::fs::write(
            root.path().join("Config/bluetooth-headphones"),
            "AA:BB:CC:DD:EE:FF",
        )
        .unwrap();
        std::fs::write(root.path().join("Config/bluetooth-off"), "").unwrap();
        events.sampled = None;
        assert_eq!(events.poll(root.path(), true), (false, false));
        assert_eq!(events.poll(root.path(), false), (false, false));
        assert!(events.child.is_none());
    }

    #[test]
    fn only_a_valid_selected_headphone_route_can_show_an_indicator() {
        assert_eq!(
            route_address("bluealsa:DEV=AA:BB:CC:DD:EE:FF,PROFILE=a2dp\n"),
            Some("AA:BB:CC:DD:EE:FF")
        );
        for route in [
            "",
            "speaker",
            "bluealsa:DEV=bad,PROFILE=a2dp",
            "bluealsa:DEV=AA:BB:CC:DD:EE:FF,PROFILE=sco",
        ] {
            assert_eq!(route_address(route), None);
        }
        assert!(connection_present("Name: WF-C510\n\tConnected: yes\n"));
        assert!(!connection_present("Connected: no\nAlias: Connected: yes"));
    }
}

/// Check the device asynchronously: unsupported hosts never expose the menu.
/// The helper owns BaseOS initialization; the UI has no model-name assumptions.
pub fn probe(root: &std::path::Path) -> Option<Receiver<bool>> {
    if !cfg!(feature = "device") || !root.join("System/bluetooth.sh").is_file() {
        return None;
    }
    let root = root.to_owned();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let supported = std::process::Command::new("timeout")
            .args(["25", "/bin/sh"])
            .arg(root.join("System/bluetooth.sh"))
            .arg("supported")
            .env("SLOT_ROOT", &root)
            .output()
            .is_ok_and(|o| o.status.success());
        let _ = tx.send(supported);
        if supported {
            let verb = startup_verb(&root);
            let result = run(root, verb, None);
            if result.status.starts_with("Error:") {
                eprintln!("slot: headphone reconnect unavailable");
            }
        }
    });
    Some(rx)
}

/// Small antialiased progress ring. Frames are cached once by the frontend;
/// animation never rerasterizes the full menu or delays its worker.
pub fn progress_face(frame: usize) -> UndoFace { slot_ui::loading_face(frame) }
