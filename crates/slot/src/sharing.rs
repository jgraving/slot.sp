//! Password-only, encrypted SMB sharing of the content SD card.
use std::{
    net::Ipv4Addr,
    path::{Path, PathBuf},
    sync::mpsc::{channel, Receiver, Sender},
};
use zeroize::Zeroizing;

pub const PASSWORD_MIN: usize = 8;
pub const PASSWORD_MAX: usize = 32;

pub fn valid_password(password: &str) -> bool {
    (PASSWORD_MIN..=PASSWORD_MAX).contains(&password.len())
        && password.bytes().all(|byte| (b' '..=b'~').contains(&byte))
}
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
    Stopped(Option<Vec<slot_store::Cart>>),
    Failed(&'static str),
}
enum Command {
    Start(u64, Start),
    Stop(u64),
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
                            let _ = tx.send((current_gen, Event::Stopped(scan_library(&root))));
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
                    Command::Stop(gen) => {
                        let was_running=running.is_some();
                        stop(&mut running);
                        let catalog = if was_running { scan_library(&root) } else { None };
                        (gen, Event::Stopped(catalog))
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
            None => Command::Stop(self.generation),
        };
        if self.jobs.send(command).is_err() {
            self.view.busy = false;
            self.view.error = Some("Sharing unavailable");
        }
    }
    pub fn poll(&mut self) -> Vec<Event> {
        let mut events = Vec::new();
        while let Ok((gen, event)) = self.events.try_recv() {
            // A catalog refresh still matters if the share has already restarted.
            if matches!(&event, Event::Stopped(_)) {
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
        let status=setup.wait().map_err(|_| "Sharing unavailable")?;
        if !status.success() {
            return Err(match status.code() {
                Some(10)=>"File server missing · update System",
                Some(12)=>"Sharing account setup failed",
                Some(13)=>"Sharing configuration failed",
                _=>"Sharing setup failed · save report",
            });
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
        for _ in 0..100 {
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
