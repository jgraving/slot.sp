//! Password-only, encrypted SMB sharing. The SD root is never exported.
use std::{
    fs,
    io::{self, Read},
    net::Ipv4Addr,
    path::{Path, PathBuf},
    sync::mpsc::{channel, Receiver, Sender},
};
use zeroize::Zeroizing;

pub const INBOX: &str = "Inbox";
#[derive(Clone, PartialEq, Eq)]
pub struct Connection {
    pub ssid: String,
    pub ip: Ipv4Addr,
}
pub struct Start {
    pub connection: Connection,
    pub password: Zeroizing<String>,
}
#[derive(Clone, Default, PartialEq, Eq)]
pub struct View {
    pub active: bool,
    pub busy: bool,
    pub error: Option<&'static str>,
}
pub enum Event {
    Running,
    Stopped(Import),
    Failed(&'static str),
}
#[derive(Default)]
pub struct Import {
    pub added: Vec<slot_store::Cart>,
    pub skipped: usize,
}
enum Command {
    Start(u64, Start),
    Stop(u64, bool),
}
pub struct Sharing {
    jobs: Sender<Command>,
    events: Receiver<(u64, Event)>,
    pub view: View,
    wanted: Option<Connection>,
    generation: u64,
}
impl Sharing {
    pub fn new(root: PathBuf) -> Self {
        let (jobs, rx) = channel();
        let (tx, events) = channel();
        std::thread::spawn(move || {
            let mut running: Option<Running> = None;
            let mut current_gen = 0;
            loop {
                let command = match rx.recv_timeout(std::time::Duration::from_millis(250)) {
                    Ok(command) => command,
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        let died = running
                            .as_mut()
                            .and_then(|r| r.child.as_mut())
                            .is_some_and(|c| c.try_wait().ok().flatten().is_some());
                        if died {
                            stop(&mut running);
                            let _ = tx.send((
                                current_gen,
                                Event::Failed("Sharing stopped · connection closed"),
                            ));
                        }
                        continue;
                    }
                };
                let (gen, result) = match command {
                    Command::Start(gen, start) => {
                        stop(&mut running);
                        let result = launch(&root, start)
                            .map(|child| {
                                running = Some(child);
                                Event::Running
                            })
                            .unwrap_or_else(Event::Failed);
                        (gen, result)
                    }
                    Command::Stop(gen, import_uploads) => {
                        stop(&mut running);
                        (
                            gen,
                            Event::Stopped(if import_uploads {
                                import(&root).unwrap_or_else(|_| Import {
                                    skipped: 1,
                                    ..Import::default()
                                })
                            } else {
                                Import::default()
                            }),
                        )
                    }
                };
                current_gen = gen;
                if tx.send((gen, result)).is_err() {
                    break;
                }
            }
            stop(&mut running);
        });
        Self {
            jobs,
            events,
            view: View::default(),
            wanted: None,
            generation: 0,
        }
    }
    pub fn request(&mut self, start: Option<Start>) {
        let connection = start.as_ref().map(|s| s.connection.clone());
        if connection == self.wanted {
            return;
        }
        self.wanted = connection;
        self.generation = self.generation.wrapping_add(1);
        self.view.active = false;
        self.view.busy = true;
        self.view.error = None;
        let command = match start {
            Some(start) => Command::Start(self.generation, start),
            None => Command::Stop(self.generation, false),
        };
        if self.jobs.send(command).is_err() {
            self.view.busy = false;
            self.view.error = Some("Sharing unavailable");
        }
    }
    pub fn import_uploads(&mut self) {
        self.wanted = None;
        self.generation = self.generation.wrapping_add(1);
        self.view.active = false;
        self.view.busy = true;
        let _ = self.jobs.send(Command::Stop(self.generation, true));
    }
    pub fn poll(&mut self) -> Vec<Event> {
        let mut events = Vec::new();
        while let Ok((gen, event)) = self.events.try_recv() {
            // An import must never be lost when a network transition overtakes its UI result.
            if let Event::Stopped(import) = event {
                if gen == self.generation {
                    self.view = View::default();
                }
                events.push(Event::Stopped(import));
                continue;
            }
            if gen != self.generation {
                continue;
            }
            self.view.busy = false;
            match &event {
                Event::Running => self.view.active = true,
                Event::Failed(error) => {
                    self.view.active = false;
                    self.view.error = Some(error);
                    self.wanted = None;
                }
                _ => {}
            }
            events.push(event);
        }
        events
    }
    pub fn ready(&self) -> bool {
        !self.view.active && !self.view.busy
    }
}
pub fn password() -> Result<Zeroizing<String>, &'static str> {
    use ring::rand::SecureRandom;
    let mut bytes = [0u8; 16];
    ring::rand::SystemRandom::new()
        .fill(&mut bytes)
        .map_err(|_| "Could not create password")?;
    const LETTERS: &[u8; 32] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    Ok(Zeroizing::new(
        bytes
            .iter()
            .map(|b| LETTERS[(b & 31) as usize] as char)
            .collect(),
    ))
}

// A separate process group contains Samba, discovery and the crash watchdog.
struct Running {
    child: Option<std::process::Child>,
    mdns: Option<mdns_sd::ServiceDaemon>,
}
fn stop(running: &mut Option<Running>) {
    if let Some(mut run) = running.take() {
        if let Some(mdns) = run.mdns.take() {
            if let Ok(done) = mdns.shutdown() {
                let _ = done.recv_timeout(std::time::Duration::from_secs(1));
            }
        }
        if let Some(mut child) = run.child.take() {
            #[cfg(unix)]
            unsafe {
                libc::kill(-(child.id() as i32), libc::SIGTERM);
            }
            for _ in 0..20 {
                if child.try_wait().ok().flatten().is_some() {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(25));
            }
            #[cfg(unix)]
            unsafe {
                libc::kill(-(child.id() as i32), libc::SIGKILL);
            }
            let _ = child.wait();
        }
    }
}
fn launch(root: &Path, start: Start) -> Result<Running, &'static str> {
    #[cfg(not(all(feature = "device", target_os = "linux")))]
    {
        let _ = (root, start);
        return Ok(Running {
            child: None,
            mdns: None,
        });
    }
    #[cfg(all(feature = "device", target_os = "linux"))]
    {
        use std::{
            io::Write,
            os::unix::process::CommandExt,
            process::{Command, Stdio},
            time::Duration,
        };
        let script = root.join("System/slotsmb.sh");
        let mut setup = Command::new("sh")
            .arg(&script)
            .arg("prepare")
            .arg(root)
            .arg(start.connection.ip.to_string())
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| "Sharing unavailable")?;
        let mut input = setup.stdin.take().ok_or("Sharing unavailable")?;
        writeln!(input, "{}", start.password.as_str())
            .map_err(|_| "Could not set sharing password")?;
        drop(input);
        if !setup.wait().map_err(|_| "Sharing unavailable")?.success() {
            return Err("Sharing unavailable · update System");
        }
        let child = Command::new("sh")
            .arg(script)
            .arg("serve")
            .arg(root)
            .arg(std::process::id().to_string())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .process_group(0)
            .spawn()
            .map_err(|_| "Could not start sharing")?;
        let mut run = Some(Running {
            child: Some(child),
            mdns: None,
        });
        let mut ready = false;
        for _ in 0..30 {
            std::thread::sleep(Duration::from_millis(100));
            if run
                .as_mut()
                .unwrap()
                .child
                .as_mut()
                .unwrap()
                .try_wait()
                .ok()
                .flatten()
                .is_some()
            {
                break;
            }
            if std::net::TcpStream::connect_timeout(
                &(start.connection.ip, 445).into(),
                Duration::from_millis(50),
            )
            .is_ok()
            {
                ready = true;
                break;
            }
        }
        if !ready {
            stop(&mut run);
            return Err("Could not start sharing");
        }
        let mdns = mdns_sd::ServiceDaemon::new().map_err(|_| "Discovery unavailable");
        if let Ok(mdns) = mdns {
            let ip = start.connection.ip;
            let _ = mdns.disable_interface(mdns_sd::IfKind::All);
            let _ = mdns.enable_interface(mdns_sd::IfKind::Addr(ip.into()));
            let host = format!("slot-sp-{:08x}.local.", u32::from(ip));
            let name = format!("slot.sp · {ip}");
            if let Ok(info) = mdns_sd::ServiceInfo::new(
                "_smb._tcp.local.",
                &name,
                &host,
                ip.to_string(),
                445,
                None,
            ) {
                let _ = mdns.register(info);
            }
            run.as_mut().unwrap().mdns = Some(mdns);
        }
        Ok(run.take().unwrap())
    }
}

/// Imports are performed only after all SMB writers have stopped. Existing games are never overwritten.
/// Only flat GB/GBC/GBA game files are accepted; links, hidden files, scripts and folders stay in Inbox.
pub fn import(root: &Path) -> io::Result<Import> {
    let mut result = Import::default();
    for platform in slot_store::Platform::ALL {
        let src = root.join(INBOX).join(platform.dir_name());
        let target = root.join("Games").join(platform.dir_name());
        if !real_dir(&root.join(INBOX)) || !real_dir(&src) || !real_dir(&root.join("Games")) {
            continue;
        }
        if fs::create_dir_all(&target).is_err() || !real_dir(&target) {
            result.skipped += 1;
            continue;
        }
        let entries = match fs::read_dir(&src) {
            Ok(entries) => entries,
            Err(_) => {
                result.skipped += 1;
                continue;
            }
        };
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => {
                    result.skipped += 1;
                    continue;
                }
            };
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            let expected = match platform {
                slot_store::Platform::Gb => "gb",
                slot_store::Platform::Gbc => "gbc",
                slot_store::Platform::Gba => "gba",
            };
            let meta = match fs::symlink_metadata(&path) {
                Ok(meta) => meta,
                Err(_) => {
                    result.skipped += 1;
                    continue;
                }
            };
            if !meta.file_type().is_file()
                || meta.len() < 192
                || meta.len() > 64 * 1024 * 1024
                || name.starts_with('.')
                || name.contains(['\\', '\n', '\r'])
                || !ext.eq_ignore_ascii_case(expected)
            {
                result.skipped += 1;
                continue;
            }
            // Read the full file to a private staging file before publishing; FAT hard links are unavailable.
            let dest = target.join(name.as_ref());
            if dest.exists() {
                result.skipped += 1;
                continue;
            }
            let temp = target.join(format!(".slot-upload-{}", std::process::id()));
            let mut reader = match fs::File::open(&path) {
                Ok(reader) => reader,
                Err(_) => {
                    result.skipped += 1;
                    continue;
                }
            };
            let mut writer = match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temp)
            {
                Ok(writer) => writer,
                Err(_) => {
                    result.skipped += 1;
                    continue;
                }
            };
            let copied = io::copy(
                &mut std::io::Read::by_ref(&mut reader).take(64 * 1024 * 1024 + 1),
                &mut writer,
            )
            .and_then(|n| {
                if n != meta.len() {
                    return Err(io::Error::other("upload changed"));
                }
                writer.sync_all()
            });
            drop(writer);
            if copied.is_err() {
                let _ = fs::remove_file(&temp);
                result.skipped += 1;
                continue;
            }
            if dest.exists() {
                let _ = fs::remove_file(&temp);
                result.skipped += 1;
                continue;
            }
            if fs::rename(&temp, &dest).is_err() {
                let _ = fs::remove_file(&temp);
                result.skipped += 1;
                continue;
            }
            if fs::remove_file(&path).is_err() {
                result.skipped += 1;
            }
            let stem = dest
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            result.added.push(slot_store::Cart {
                platform,
                stem,
                rom: dest,
                label: None,
                title: String::new(),
                code: String::new(),
                shell: None,
            });
        }
    }
    Ok(result)
}
fn real_dir(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_dir())
}
