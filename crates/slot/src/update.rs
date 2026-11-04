//! Adapted from BrandonKowalski/slot OTA (87788eaf), GPL-3.0-or-later.
//! Fork release selection and archive checks deliberately use slot.sp packaging.
use std::io::{Cursor, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

struct Sha256(ring::digest::Context);
impl Sha256 {
    fn new()->Self {Self(ring::digest::Context::new(&ring::digest::SHA256))}
    fn update(&mut self,bytes:&[u8]) {self.0.update(bytes);}
    fn finalize(self)->[u8;32] {self.0.finish().as_ref().try_into().unwrap()}
    fn digest(bytes:&[u8])->[u8;32] {ring::digest::digest(&ring::digest::SHA256,bytes).as_ref().try_into().unwrap()}
}

pub const STAGING: &str = "System.new";
pub const MANIFEST: &str = ".manifest";
const REQUIRED: [&str; 4] = [
    "slot",
    "launch_frontend.sh",
    "mgba_libretro.so",
    "gpsp_libretro.so",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Slot,
    BaseOs,
}

impl Kind {
    pub fn index(self) -> usize {
        match self {
            Kind::Slot => 0,
            Kind::BaseOs => 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Release {
    pub kind: Kind,
    pub running: String,
    pub version: String,
    pub url: String,
    pub sha256: [u8; 32],
    pub size: u64,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateFail {
    NoWifi,
    Offline,
    Clock,
    Download,
    Checksum,
    Bad,
    Space,
    Rejected(String),
}

pub const SLOT_REPO: &str = "jgraving/slot.sp";
const BASEOS_REPO: &str = "pvaibhav/BaseOS";
pub const LATEST: &str = "https://api.github.com/repos/jgraving/slot.sp/releases/latest";
const BASEOS_LATEST: &str = "https://api.github.com/repos/pvaibhav/BaseOS/releases/latest";
const BASEOS_RELEASES: &str = "https://api.github.com/repos/pvaibhav/BaseOS/releases?per_page=8";

fn release_json(json:&str)->Result<serde_json::Value,UpdateFail> {
    let v:serde_json::Value=serde_json::from_str(json).map_err(|_|UpdateFail::Bad)?;
    if v["draft"]!=false || v["prerelease"]!=false {return Err(UpdateFail::Bad);}
    Ok(v)
}
fn asset(v:&serde_json::Value,repo:&str,name:&str,kind:Kind)->Result<Release,UpdateFail> {
    let tag=v["tag_name"].as_str().filter(|t|!t.is_empty()&&t.bytes().all(|c|c.is_ascii_alphanumeric()||b"._-".contains(&c))).ok_or(UpdateFail::Bad)?;
    let a=v["assets"].as_array().and_then(|a|a.iter().find(|a|a["name"]==name && a["state"]=="uploaded")).ok_or(UpdateFail::Bad)?;
    let url=format!("https://github.com/{repo}/releases/download/{tag}/{name}");
    if a["browser_download_url"]!=url {return Err(UpdateFail::Bad);}
    let digest=a["digest"].as_str().and_then(|s|s.strip_prefix("sha256:")).and_then(hex32).ok_or(UpdateFail::Bad)?;
    let size=a["size"].as_u64().filter(|&s|s>0&&s<=512*1024*1024).ok_or(UpdateFail::Bad)?;
    Ok(Release {kind,running:String::new(),version:tag.into(),url,sha256:digest,size,source:String::new()})
}
pub fn parse_release(json:&str)->Result<Release,UpdateFail> {
    let v=release_json(json)?;
    let source=v["target_commitish"].as_str().filter(|s|s.len()==40&&s.bytes().all(|c|c.is_ascii_hexdigit())).ok_or(UpdateFail::Bad)?;
    let mut r=asset(&v,SLOT_REPO,"slot.sp-System.zip",Kind::Slot)?;
    r.source=source.into();Ok(r)
}
pub fn parse_baseos(json:&str,target:&str,running:&str)->Option<Release> {
    let v = release_json(json).ok()?;
    baseos_asset(&v, target, running)
}

fn baseos_asset(v: &serde_json::Value, target: &str, running: &str) -> Option<Release> {
    if target.is_empty() || !target.bytes().all(|c| c.is_ascii_alphanumeric()) {
        return None;
    }
    let version = v["tag_name"].as_str()?.strip_prefix('v')?;
    if !plain(version) {
        return None;
    }
    let mut release = asset(v, BASEOS_REPO, &format!("baseos-{target}-{version}.bosupd"), Kind::BaseOs).ok()?;
    release.version = version.into();
    release.running = running.strip_prefix('v').unwrap_or(running).into();
    if v["prerelease"] == true {
        release.version.push_str(" (preview)");
    }
    Some(release)
}

fn baseos_version(release: &Release) -> &str {
    release.version.strip_suffix(" (preview)").unwrap_or(&release.version)
}

fn parse_baseos_releases(json: &str, target: &str, running: &str) -> Result<Option<Release>, UpdateFail> {
    let releases: Vec<serde_json::Value> = serde_json::from_str(json).map_err(|_| UpdateFail::Bad)?;
    Ok(releases.iter()
        .filter(|v| v["draft"] == false && v["prerelease"].is_boolean())
        .filter_map(|v| baseos_asset(v, target, running))
        .max_by_key(|r| (parts(baseos_version(r)), !r.version.ends_with(" (preview)"))))
}

pub fn plain(v: &str) -> bool {
    parts(v).is_some_and(|p| p.len() == 3)
}

fn hex32(s: &str) -> Option<[u8; 32]> {
    if s.len() != 64 {
        return None;
    }
    let mut out = [0u8; 32];
    for (i, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(s.get(i * 2..i * 2 + 2)?, 16).ok()?;
    }
    Some(out)
}

fn parts(v: &str) -> Option<Vec<u64>> {
    v.split('.').map(|p| p.parse().ok()).collect()
}

pub fn newer(release: &str, running: &str) -> bool {
    match (parts(release), parts(running)) {
        (Some(r), Some(c)) => r > c,
        _ => false,
    }
}

pub fn verify(bytes: &[u8], release: &Release) -> Result<(), UpdateFail> {
    let got: [u8; 32] = Sha256::digest(bytes).into();
    match got == release.sha256 {
        true => Ok(()),
        false => Err(UpdateFail::Checksum),
    }
}

pub fn stage(root: &Path, zip: &[u8], release:&Release) -> Result<(), UpdateFail> {
    let dir = root.join(STAGING);
    if dir.join(MANIFEST).is_file() {return Err(UpdateFail::Rejected("Restart to finish the staged update".into()));}
    match std::fs::symlink_metadata(&dir) {
        Ok(meta) if meta.is_dir() => std::fs::remove_dir_all(&dir).map_err(|_|UpdateFail::Bad)?,
        Ok(_) => return Err(UpdateFail::Bad),
        Err(e) if e.kind()==std::io::ErrorKind::NotFound => {},
        Err(_) => return Err(UpdateFail::Bad),
    }
    // Never extract into a leftover tree after failed cleanup (or follow a
    // replacement symlink at the staging root).
    std::fs::create_dir(&dir).map_err(|_|UpdateFail::Bad)?;
    let result = unpack(&dir, zip, release);
    if result.is_err() {
        let _ = std::fs::remove_dir_all(&dir);
    }
    result
}

fn unpack(dir: &Path, zip: &[u8], release:&Release) -> Result<(), UpdateFail> {
    let mut archive = zip::ZipArchive::new(Cursor::new(zip)).map_err(|_| UpdateFail::Bad)?;
    let mut listed = Vec::new();
    let mut expanded=0u64;
    for i in 0..archive.len() {
        let entry = archive.by_index(i).map_err(|_| UpdateFail::Bad)?;
        let rel = match system_path(entry.name())? {
            Some(rel) if !entry.is_dir() => rel,
            _ => continue,
        };
        if entry.unix_mode().is_some_and(|m|m & 0o170000 == 0o120000) {return Err(UpdateFail::Bad);}
        expanded=expanded.checked_add(entry.size()).ok_or(UpdateFail::Bad)?;
        if entry.size()>64*1024*1024 || expanded>512*1024*1024 {return Err(UpdateFail::Bad);}
        let name=rel.to_string_lossy().replace('\\',"/");
        if listed.contains(&name) || name==MANIFEST {return Err(UpdateFail::Bad);}
        if name.bytes().any(|c|c<32||c==127) {return Err(UpdateFail::Bad);}
        let expected_size=entry.size();
        let mode=entry.unix_mode().unwrap_or(0o644)&0o777;
        let out = dir.join(&rel);
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent).map_err(|_| UpdateFail::Bad)?;
        }
        let mut bytes = Vec::new();
        entry.take(64*1024*1024+1).read_to_end(&mut bytes).map_err(|_| UpdateFail::Bad)?;
        if bytes.len()>64*1024*1024 || bytes.len() as u64 != expected_size {return Err(UpdateFail::Bad);}
        let mut file=std::fs::File::create(&out).map_err(|_|UpdateFail::Bad)?;
        file.write_all(&bytes).map_err(|_|UpdateFail::Bad)?;
        file.sync_all().map_err(|_|UpdateFail::Bad)?;
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&out,std::fs::Permissions::from_mode(mode)).map_err(|_|UpdateFail::Bad)?;
        listed.push(name);
    }
    if REQUIRED.iter().any(|r| !listed.iter().any(|l| l == r)) {
        return Err(UpdateFail::Bad);
    }
    let meta:serde_json::Value=serde_json::from_slice(&std::fs::read(dir.join("build-info.json")).map_err(|_|UpdateFail::Bad)?).map_err(|_|UpdateFail::Bad)?;
    if meta["product"]!="slot.sp" || meta["repository"]!=SLOT_REPO || meta["architecture"]!="aarch64" || meta["source_commit"]!=release.source {return Err(UpdateFail::Bad);}
    let mut manifest = listed.join("\n");
    manifest.push('\n');
    slot_store::atomic_write(&dir.join(MANIFEST), manifest.as_bytes()).map_err(|_| UpdateFail::Bad)
}

fn system_path(name: &str) -> Result<Option<PathBuf>, UpdateFail> {
    let path = Path::new(name);
    if path
        .components()
        .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(UpdateFail::Bad);
    }
    let mut parts = path.components();
    if parts.next().map(|c|c.as_os_str())!=Some("System".as_ref()) {return Err(UpdateFail::Bad);}
    let rel:PathBuf=parts.collect();
    Ok((!rel.as_os_str().is_empty()).then_some(rel))
}

pub fn clock_is_wrong(now_unix: u64) -> bool {
    build_day().is_some_and(|day| now_unix < day * 86_400)
}

fn build_day() -> Option<u64> {
    let mut it = env!("SLOT_BUILD_DATE")
        .split('-')
        .map(|p| p.parse::<i64>().ok());
    let (y, m, d) = (it.next()??, it.next()??, it.next()??);
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    u64::try_from(era * 146_097 + doe - 719_468).ok()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateJob {
    Check,
    CheckWithBaseOsPreviews,
    Install(Release),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateEvent {
    Available(Vec<Release>),
    UpToDate,
    Progress(u64),
    Staged,
    Cancelled,
    Failed(UpdateFail),
}

const RUNNING: u8 = 0;
const CANCELLED: u8 = 1;
const COMMITTED: u8 = 2;

#[derive(Debug, Clone, Default)]
pub struct Ticket(Arc<AtomicU8>);

impl Ticket {
    pub fn cancel(&self) -> bool {
        self.swap(CANCELLED)
    }

    pub fn commit(&self) -> bool {
        self.swap(COMMITTED)
    }

    pub fn cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst) == CANCELLED
    }

    pub fn settle(&self, event: UpdateEvent) -> UpdateEvent {
        match self.cancelled() {
            true => UpdateEvent::Cancelled,
            false => event,
        }
    }

    fn swap(&self, to: u8) -> bool {
        self.0
            .compare_exchange(RUNNING, to, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }
}

pub struct Stoppable<R> {
    inner: R,
    ticket: Ticket,
}

impl<R> Stoppable<R> {
    pub fn new(inner: R, ticket: Ticket) -> Self {
        Self { inner, ticket }
    }
}

impl<R: Read> Read for Stoppable<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.ticket.cancelled() {
            return Err(std::io::Error::other("cancelled"));
        }
        self.inner.read(buf)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Running<'a> {
    pub slot: &'a str,
    pub baseos: Option<(&'a str, &'a str)>,
}

pub fn check_with(
    now_unix: u64,
    fetch: &mut dyn FnMut(&str) -> Result<Vec<u8>, UpdateFail>,
    running: Running,
) -> UpdateEvent {
    check_with_options(now_unix, fetch, running, false)
}

pub fn check_with_options(
    now_unix: u64,
    fetch: &mut dyn FnMut(&str) -> Result<Vec<u8>, UpdateFail>,
    running: Running,
    previews: bool,
) -> UpdateEvent {
    if clock_is_wrong(now_unix) {
        return UpdateEvent::Failed(UpdateFail::Clock);
    }
    let mut get = |url: &str| fetch(url).and_then(|b| String::from_utf8(b).map_err(|_| UpdateFail::Bad));
    let slot = (|| {
        let mut slot = parse_release(&get(LATEST)?)?;
        slot.running = running.slot.into();
        if slot.running.is_empty() || !slot.running.bytes().all(|c| c.is_ascii_hexdigit()) {
            return Err(UpdateFail::Bad);
        }
        if slot.source.starts_with(&slot.running) {
            return Ok(None);
        }
        // Only offer a descendant of the installed build, never a downgrade.
        let url = format!("https://api.github.com/repos/{SLOT_REPO}/compare/{}...{}", slot.running, slot.source);
        let ancestry: serde_json::Value = serde_json::from_str(&get(&url)?).map_err(|_| UpdateFail::Bad)?;
        match ancestry["status"].as_str() {
            Some("ahead") => Ok(Some(slot)),
            Some("behind" | "diverged" | "identical") => Ok(None),
            _ => Err(UpdateFail::Bad),
        }
    })();
    let baseos = (|| {
        let Some((target, version)) = running.baseos else { return Ok(None); };
        let release = if previews {
            parse_baseos_releases(&get(BASEOS_RELEASES)?, target, version)?
        } else {
            parse_baseos(&get(BASEOS_LATEST)?, target, version)
        };
        Ok(release.filter(|r| newer(baseos_version(r), &r.running)))
    })();
    let mut offers = Vec::new();
    let mut failure = None;
    for result in [slot, baseos] {
        match result {
            Ok(Some(release)) => offers.push(release),
            Ok(None) => {},
            Err(error) => { failure.get_or_insert(error); }
        }
    }
    if !offers.is_empty() {
        UpdateEvent::Available(offers)
    } else if let Some(error) = failure {
        UpdateEvent::Failed(error)
    } else {
        UpdateEvent::UpToDate
    }
}

pub fn install_with(
    root: &Path,
    fetch: &mut dyn FnMut(&str) -> Result<Vec<u8>, UpdateFail>,
    release: &Release,
) -> UpdateEvent {
    let staged = fetch(&release.url)
        .and_then(|bytes| match bytes.len() as u64 == release.size {
            true => Ok(bytes),
            false => Err(UpdateFail::Download),
        })
        .and_then(|bytes| verify(&bytes, release).map(|()| bytes))
        .and_then(|bytes| stage(root, &bytes, release));
    match staged {
        Ok(()) => UpdateEvent::Staged,
        Err(e) => UpdateEvent::Failed(e),
    }
}

pub fn read_counted(
    reader: &mut dyn Read,
    limit: u64,
    report: &mut dyn FnMut(u64),
) -> Result<Vec<u8>, UpdateFail> {
    let mut body = Vec::new();
    let mut chunk = vec![0u8; 64 * 1024];
    loop {
        let n = reader.read(&mut chunk).map_err(|_| UpdateFail::Download)?;
        if n == 0 {
            return Ok(body);
        }
        body.extend_from_slice(&chunk[..n]);
        if body.len() as u64 > limit {
            return Err(UpdateFail::Download);
        }
        report(body.len() as u64);
    }
}

pub fn copy_counted(
    reader: &mut dyn Read,
    out: &mut dyn Write,
    limit: u64,
    report: &mut dyn FnMut(u64),
) -> Result<[u8; 32], UpdateFail> {
    let mut hash = Sha256::new();
    copy_into(reader, out, &mut hash, &mut 0, limit, report)?;
    Ok(hash.finalize().into())
}

pub fn fetch_ranged(
    open: &mut dyn FnMut(u64, u64) -> Result<Box<dyn Read>, UpdateFail>,
    size: u64,
    chunk: u64,
    out: &mut dyn Write,
    report: &mut dyn FnMut(u64),
) -> Result<[u8; 32], UpdateFail> {
    let mut hash = Sha256::new();
    let mut got = 0u64;
    while got < size {
        let to = (got + chunk).min(size) - 1;
        let mut body = open(got, to)?;
        copy_into(&mut body, out, &mut hash, &mut got, to + 1, report)?;
        if got != to + 1 {
            return Err(UpdateFail::Download);
        }
    }
    Ok(hash.finalize().into())
}

fn copy_into(
    reader: &mut dyn Read,
    out: &mut dyn Write,
    hash: &mut Sha256,
    got: &mut u64,
    limit: u64,
    report: &mut dyn FnMut(u64),
) -> Result<(), UpdateFail> {
    let mut chunk = vec![0u8; 64 * 1024];
    loop {
        let n = reader.read(&mut chunk).map_err(|_| UpdateFail::Download)?;
        if n == 0 {
            return Ok(());
        }
        *got += n as u64;
        if *got > limit {
            return Err(UpdateFail::Download);
        }
        out.write_all(&chunk[..n])
            .map_err(|e| match e.raw_os_error() {
                Some(libc::ENOSPC) => UpdateFail::Space,
                _ => UpdateFail::Download,
            })?;
        hash.update(&chunk[..n]);
        report(*got);
    }
}

pub fn bosupd_name(release: &Release) -> String {
    release
        .url
        .rsplit('/')
        .next()
        .unwrap_or_default()
        .to_string()
}

pub fn will_apply(status: &str, name: &str) -> Result<(), String> {
    let ours = format!("/{name}");
    let mut lines = status.lines().map(str::trim);
    lines.find(|l| {
        l.strip_prefix("payload:")
            .is_some_and(|p| p.trim().ends_with(&ours))
    });
    for line in lines.take_while(|l| !l.starts_with("payload:")) {
        if line.starts_with("WILL APPLY") {
            return Ok(());
        }
        for verdict in ["skipped:", "rejected:", "cannot apply:"] {
            if let Some(why) = line.strip_prefix(verdict) {
                return Err(why.trim().to_string());
            }
        }
    }
    Err("BaseOS did not see the update".to_string())
}

pub type FetchInto<'a> = dyn FnMut(&str, &mut dyn Write) -> Result<[u8; 32], UpdateFail> + 'a;

pub fn install_baseos_with(
    root: &Path,
    free: u64,
    fetch_into: &mut FetchInto<'_>,
    status: &mut dyn FnMut() -> Option<String>,
    release: &Release,
) -> UpdateEvent {
    if free < release.size {
        return UpdateEvent::Failed(UpdateFail::Space);
    }
    let name = bosupd_name(release);
    let done = root.join(&name);
    let part = root.join(format!("{name}.part"));
    // A ready payload belongs to the next boot, not this download attempt.
    match std::fs::symlink_metadata(&done) {
        Ok(_) => return UpdateEvent::Failed(UpdateFail::Rejected("Restart to finish the staged BaseOS update".into())),
        Err(e) if e.kind()==std::io::ErrorKind::NotFound => {},
        Err(_) => return UpdateEvent::Failed(UpdateFail::Download),
    }
    match std::fs::symlink_metadata(&part) {
        Ok(meta) if meta.is_file() => {
            if std::fs::remove_file(&part).is_err() {return UpdateEvent::Failed(UpdateFail::Download);}
        },
        Ok(_) => return UpdateEvent::Failed(UpdateFail::Bad),
        Err(e) if e.kind()==std::io::ErrorKind::NotFound => {},
        Err(_) => return UpdateEvent::Failed(UpdateFail::Download),
    }
    let mut published=false;
    let result = download_bosupd(&part, fetch_into, release)
        .and_then(|()| {
            std::fs::rename(&part, &done).map_err(|_| UpdateFail::Download)?;
            published=true;Ok(())
        })
        .and_then(|()| match status() {
            Some(text) => will_apply(&text, &name).map_err(UpdateFail::Rejected),
            None => Err(UpdateFail::Rejected("BaseOS did not see the update".into())),
        });
    match result {
        Ok(()) => UpdateEvent::Staged,
        Err(e) => {
            let _ = std::fs::remove_file(&part);
            if published {let _ = std::fs::remove_file(&done);}
            UpdateEvent::Failed(e)
        }
    }
}

fn download_bosupd(
    part: &Path,
    fetch_into: &mut FetchInto<'_>,
    release: &Release,
) -> Result<(), UpdateFail> {
    let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(part).map_err(|_| UpdateFail::Download)?;
    let digest = fetch_into(&release.url, &mut file)?;
    file.sync_all().map_err(|_| UpdateFail::Download)?;
    let size = file.metadata().map_err(|_| UpdateFail::Download)?.len();
    if size != release.size {
        return Err(UpdateFail::Download);
    }
    match digest == release.sha256 {
        true => Ok(()),
        false => Err(UpdateFail::Checksum),
    }
}

pub fn clean_baseos(root: &Path, target: &str, running: &str) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    let prefix = format!("baseos-{target}-");
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(rest) = name.strip_prefix(&prefix) else {
            continue;
        };
        let stale = match (
            rest.strip_suffix(".bosupd.part"),
            rest.strip_suffix(".bosupd"),
        ) {
            (Some(_), _) => true,
            (None, Some(version)) => plain(version) && !newer(version, running),
            _ => false,
        };
        if stale {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

pub trait UpdateJobs: Send {
    fn ask(&mut self, job: UpdateJob);

    fn take(&mut self) -> Option<UpdateEvent>;

    fn cancel(&self) -> bool {
        false
    }
}

struct NoUpdates;

impl UpdateJobs for NoUpdates {
    fn ask(&mut self, _job: UpdateJob) {}

    fn take(&mut self) -> Option<UpdateEvent> {
        None
    }
}

pub fn no_update_jobs() -> Box<dyn UpdateJobs> {
    Box::new(NoUpdates)
}

struct UpdateQueue {
    jobs: Sender<(UpdateJob, Ticket)>,
    events: Receiver<UpdateEvent>,
    ticket: Ticket,
}

impl UpdateJobs for UpdateQueue {
    fn ask(&mut self, job: UpdateJob) {
        self.ticket = Ticket::default();
        let _ = self.jobs.send((job, self.ticket.clone()));
    }

    fn take(&mut self) -> Option<UpdateEvent> {
        self.events.try_recv().ok()
    }

    fn cancel(&self) -> bool {
        self.ticket.cancel()
    }
}

pub fn update_jobs(root: PathBuf) -> Box<dyn UpdateJobs> {
    let (jobs, rx) = channel::<(UpdateJob, Ticket)>();
    let (tx, events) = channel::<UpdateEvent>();
    std::thread::spawn(move || {
        for (job, ticket) in rx {
            if tx
                .send(ticket.settle(run(&root, &job, &ticket, &tx)))
                .is_err()
            {
                return;
            }
        }
    });
    Box::new(UpdateQueue {
        jobs,
        events,
        ticket: Ticket::default(),
    })
}

pub(crate) fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

fn run(root: &Path, job: &UpdateJob, ticket: &Ticket, tx: &Sender<UpdateEvent>) -> UpdateEvent {
    if clock_is_wrong(now_unix()) {
        return UpdateEvent::Failed(UpdateFail::Clock);
    }
    if ticket.cancelled() {return UpdateEvent::Cancelled;}
    let event = match job {
        UpdateJob::Check | UpdateJob::CheckWithBaseOsPreviews => {
            let target = crate::baseos::var("BASEOS_TARGET");
            let version = crate::baseos::var("BASEOS_VERSION");
            check_with_options(
                now_unix(),
                &mut |url| {
                    if ticket.cancelled() { return Err(UpdateFail::Offline); }
                    fetch(url, API_TIMEOUT, &mut |_| {})
                },
                Running {
                    slot: crate::build_info::Build::current().hash,
                    baseos: if baseos_status().is_some() {target.as_deref().zip(version.as_deref())}else{None},
                },
                matches!(job, UpdateJob::CheckWithBaseOsPreviews),
            )
        }
        UpdateJob::Install(release) => {
            let mut report = |got| {
                let _ = tx.send(UpdateEvent::Progress(got));
            };
            match release.kind {
                Kind::Slot => install_with(
                    root,
                    &mut |url| {
                        fetch_until(url, DOWNLOAD_TIMEOUT, ticket, &mut report)
                            .and_then(|bytes| committed(ticket, bytes))
                    },
                    release,
                ),
                Kind::BaseOs => install_baseos_with(
                    root,
                    free_space(root),
                    &mut |url, out| {
                        fetch_ranged(
                            &mut |from, to| {
                                open_range(url, from, to).map(|r| {
                                    Box::new(Stoppable::new(r, ticket.clone())) as Box<dyn Read>
                                })
                            },
                            release.size,
                            RANGE_BYTES,
                            out,
                            &mut report,
                        )
                        .and_then(|digest| committed(ticket, digest))
                    },
                    &mut baseos_status,
                    release,
                ),
            }
        }
    };
    event
}

#[cfg(feature = "device")]
fn baseos_status() -> Option<String> {
    let out = std::process::Command::new("baseos-update")
        .arg("status")
        .output()
        .ok()?;
    Some(String::from_utf8_lossy(&out.stdout).into_owned())
}

#[cfg(not(feature = "device"))]
fn baseos_status() -> Option<String> {
    None
}

fn free_space(root: &Path) -> u64 {
    use std::os::unix::ffi::OsStrExt;
    let Ok(path) = std::ffi::CString::new(root.as_os_str().as_bytes()) else {
        return 0;
    };
    let mut st: libc::statvfs = unsafe { std::mem::zeroed() };
    match unsafe { libc::statvfs(path.as_ptr(), &mut st) } {
        0 => (st.f_bavail as u64).saturating_mul(st.f_frsize as u64),
        _ => 0,
    }
}

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
pub(crate) const API_TIMEOUT: Duration = Duration::from_secs(15);
pub(crate) const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(120);
pub(crate) const DOWNLOAD_LIMIT: u64 = 64 * 1024 * 1024;
const RANGE_TIMEOUT: Duration = Duration::from_secs(90);
const RANGE_BYTES: u64 = 4 * 1024 * 1024;

fn committed<T>(ticket: &Ticket, value: T) -> Result<T, UpdateFail> {
    match ticket.commit() {
        true => Ok(value),
        false => Err(UpdateFail::Download),
    }
}

fn fetch_until(
    url: &str,
    timeout: Duration,
    ticket: &Ticket,
    report: &mut dyn FnMut(u64),
) -> Result<Vec<u8>, UpdateFail> {
    let body = open(url, timeout, DOWNLOAD_LIMIT)?;
    read_counted(
        &mut Stoppable::new(body, ticket.clone()),
        DOWNLOAD_LIMIT,
        report,
    )
}

pub(crate) fn fetch(url: &str, timeout: Duration, report: &mut dyn FnMut(u64)) -> Result<Vec<u8>, UpdateFail> {
    let mut body = open(url, timeout, DOWNLOAD_LIMIT)?;
    read_counted(&mut body, DOWNLOAD_LIMIT, report)
}

fn open_range(url: &str, from: u64, to: u64) -> Result<impl Read, UpdateFail> {
    let response = agent(RANGE_TIMEOUT)
        .get(url)
        .header("Range", format!("bytes={from}-{to}"))
        .call()
        .map_err(|e| request_fail(&e))?;
    if response.status() != 206 {
        return Err(UpdateFail::Download);
    }
    Ok(response
        .into_body()
        .into_with_config()
        .limit(to - from + 2)
        .reader())
}

fn open(url: &str, timeout: Duration, limit: u64) -> Result<impl Read, UpdateFail> {
    let response = agent(timeout)
        .get(url)
        .header("Accept","application/vnd.github+json")
        .call()
        .map_err(|e| request_fail(&e))?;
    Ok(response
        .into_body()
        .into_with_config()
        .limit(limit)
        .reader())
}

pub(crate) fn agent(timeout: Duration) -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_connect(Some(CONNECT_TIMEOUT))
        .timeout_global(Some(timeout))
        .user_agent(format!(
            "slot/{}",
            crate::build_info::Build::current().version
        ))
        .build()
        .into()
}

pub(crate) fn request_fail(e: &ureq::Error) -> UpdateFail {
    let text = format!("{e:?}");
    match text.contains("Expired") || text.contains("NotValidYet") {
        true => UpdateFail::Clock,
        false => UpdateFail::Offline,
    }
}

