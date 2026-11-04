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
    Stopped {
        legacy: Option<Import>,
        catalog: Option<Vec<slot_store::Cart>>,
    },
    Failed(&'static str),
}
#[derive(Default)]
pub struct Import {
    pub added: Vec<slot_store::Cart>,
    pub wallpapers: usize,
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
                            let _ = tx.send((current_gen, Event::Stopped {
                                legacy: None, catalog: scan_library(&root),
                            }));
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
                        let was_running = running.is_some();
                        stop(&mut running);
                        let legacy = import_uploads.then(|| import(&root).unwrap_or_else(|_| Import {
                            skipped: 1,
                            ..Import::default()
                        }));
                        let catalog = if was_running || import_uploads { scan_library(&root) } else { None };
                        (gen, Event::Stopped { legacy, catalog })
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
            if matches!(&event, Event::Stopped { .. }) {
                if gen == self.generation {
                    self.view = View::default();
                }
                events.push(event);
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
    pub(crate) fn requested_connection(&self) -> Option<&Connection> {
        self.wanted.as_ref()
    }
}
fn scan_library(root: &Path) -> Option<Vec<slot_store::Cart>> {
    let mut carts = slot_store::scan(root).ok()?;
    for cart in &mut carts {
        cart.read_header();
    }
    Some(carts)
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
/// Legacy Inbox migration accepts flat GB/GBC/GBA games and valid PNG wallpapers.
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
            if fs::symlink_metadata(&dest).is_ok() {
                result.skipped += 1;
                continue;
            }
            let mut reader = match fs::File::open(&path) {
                Ok(reader) => reader,
                Err(_) => {
                    result.skipped += 1;
                    continue;
                }
            };
            let (temp, mut writer) = match staging_file(&target) {
                Ok(staged) => staged,
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
            if fs::symlink_metadata(&dest).is_ok() {
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
    import_wallpapers(root, &mut result);
    Ok(result)
}

const MAX_WALLPAPER_FILE: u64 = 16 * 1024 * 1024;
const MAX_WALLPAPER_PIXELS: u64 = 8 * 1024 * 1024;

fn import_wallpapers(root: &Path, result: &mut Import) {
    let inbox = root.join(INBOX);
    let src = inbox.join("Wallpapers");
    let target = root.join("Wallpapers");
    if !real_dir(&inbox) || !real_dir(&src) {
        return;
    }
    // create_dir, rather than create_dir_all, must never follow an existing target link.
    if !real_dir(&target) && (target.symlink_metadata().is_ok() || fs::create_dir(&target).is_err())
    {
        result.skipped += 1;
        return;
    }
    let entries = match fs::read_dir(&src) {
        Ok(entries) => entries,
        Err(_) => {
            result.skipped += 1;
            return;
        }
    };
    for entry in entries {
        let Some(entry) = entry.ok() else {
            result.skipped += 1;
            continue;
        };
        let path = entry.path();
        let name = entry.file_name();
        // Wallpaper choices persist as UTF-8 names; refuse names that cannot round-trip.
        let valid_name = name
            .to_str()
            .is_some_and(|name| !name.starts_with('.') && !name.contains(['\\', '\n', '\r']))
            && path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("png"));
        let valid_file = path
            .symlink_metadata()
            .is_ok_and(|m| m.is_file() && m.len() > 0 && m.len() <= MAX_WALLPAPER_FILE);
        let dest = target.join(&name);
        if !valid_name || !valid_file || dest.symlink_metadata().is_ok() {
            result.skipped += 1;
            continue;
        }
        // Validate the private copy, so no unvalidated or partial image reaches the selector.
        // SMB has already stopped; the source remains intact until publishing succeeds.
        let staged = (|| -> io::Result<()> {
            let mut reader = fs::File::open(&path)?;
            let expected = reader.metadata()?.len();
            let (temp, mut writer) = staging_file(&target)?;
            let copy = (|| -> io::Result<()> {
                let copied = io::copy(
                    &mut reader.by_ref().take(MAX_WALLPAPER_FILE + 1),
                    &mut writer,
                )?;
                if copied != expected || copied > MAX_WALLPAPER_FILE {
                    return Err(io::Error::other("upload changed"));
                }
                writer.sync_all()?;
                drop(writer);
                validate_wallpaper(&temp).ok_or_else(|| io::Error::other("invalid wallpaper"))?;
                // symlink_metadata also sees dangling links; never replace those either.
                if dest.symlink_metadata().is_ok() {
                    return Err(io::ErrorKind::AlreadyExists.into());
                }
                fs::rename(&temp, &dest)
            })();
            if copy.is_err() {
                let _ = fs::remove_file(&temp);
            }
            copy
        })();
        if staged.is_err() {
            result.skipped += 1;
            continue;
        }
        result.wallpapers += 1;
        if fs::remove_file(&path).is_err() {
            result.skipped += 1;
        }
    }
}

fn validate_wallpaper(path: &Path) -> Option<()> {
    let mut decoder = png::Decoder::new(fs::File::open(path).ok()?);
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    decoder.set_limits(png::Limits {
        bytes: 32 * 1024 * 1024,
    });
    let mut reader = decoder.read_info().ok()?;
    let info = reader.info();
    if info.width > 4096
        || info.height > 4096
        || u64::from(info.width) * u64::from(info.height) > MAX_WALLPAPER_PIXELS
        || info.animation_control.is_some()
        || reader.output_buffer_size() > 32 * 1024 * 1024
    {
        return None;
    }
    let mut pixels = vec![0; reader.output_buffer_size()];
    reader.next_frame(&mut pixels).ok()?;
    reader.finish().ok()?;
    Some(())
}

// A crash can leave its private staging file behind. Never reuse a PID-only name:
// handheld startup often reuses PIDs, and a previous partial write must not block new imports.
fn staging_file(target: &Path) -> io::Result<(PathBuf, fs::File)> {
    use ring::rand::SecureRandom;
    let mut random = [0; 8];
    ring::rand::SystemRandom::new()
        .fill(&mut random)
        .map_err(|_| io::Error::other("could not create upload staging name"))?;
    let path = target.join(format!(
        ".slot-upload-{}-{:016x}",
        std::process::id(),
        u64::from_ne_bytes(random)
    ));
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?;
    Ok((path, file))
}

fn real_dir(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_dir())
}
