//! Device-only shell logo retrieval. Bundled assets remain the offline fallback.
use std::{io::Read, path::Path, sync::mpsc::{self, Receiver, TryRecvError}};
use slot_input::Millis;
use slot_ui::{extract_shell_logo, Mould};

// Pin the upstream asset revision so changes are deliberate and reproducible.
const BASE: &str = "https://raw.githubusercontent.com/slot-cfw/slot/0fe8f5cfa63161bdec5dd4680775d5a7ad0b463c/assets/";
const ASSETS: [(&str, Mould); 5] = [
    ("gba_neutral.png", Mould::Gba), ("gba_clear.png", Mould::Gba),
    ("gb_neutral.png", Mould::Notched), ("gbc_neutral.png", Mould::Rounded),
    ("gbc_clear.png", Mould::Rounded),
];
const LIMIT: u64 = 1024 * 1024;
type Logos = [Option<Vec<u8>>; 5];
#[derive(Default)]
pub struct Loader {
    asking: Option<Receiver<(Logos, bool)>>,
    loaded_cache: bool,
    complete: bool,
    retry: Millis,
    was_online: bool,
    fetching_online: bool,
}
impl Loader {
    /// Disk and HTTPS work always run on a worker. A failed request keeps the fallback
    /// and retries at most every five minutes while Wi-Fi is available.
    pub fn step(&mut self, root: &Path, online: bool, now: Millis) -> bool {
        if online && !self.was_online { self.retry = 0; }
        self.was_online = online;
        if let Some(rx) = &self.asking {
            match rx.try_recv() {
                Ok((logos, complete)) => {
                    self.asking = None;
                    self.complete = complete;
                    if self.fetching_online { self.retry = now.saturating_add(300_000); } else { self.retry = 0; }
                    let changed = logos.iter().any(Option::is_some);
                    if changed { slot_ui::install_shell_logos(logos); }
                    return changed;
                }
                Err(TryRecvError::Empty) => return false,
                Err(TryRecvError::Disconnected) => {
                    self.asking = None;
                    self.retry = now.saturating_add(300_000);
                }
            }
        }
        let first = !self.loaded_cache;
        if !first && (self.complete || !online || now < self.retry) { return false; }
        self.loaded_cache = true;
        self.fetching_online = online;
        let root = root.to_owned();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || { let _ = tx.send(load(&root, online, download)); });
        self.asking = Some(rx);
        false
    }
}
fn limited(mut reader: impl Read) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();
    reader.by_ref().take(LIMIT+1).read_to_end(&mut bytes).ok()?;
    (bytes.len() as u64 <= LIMIT).then_some(bytes)
}
fn download(name: &str) -> Option<Vec<u8>> {
    if crate::update::clock_is_wrong(crate::update::now_unix()) { return None; }
    let response = crate::update::agent(crate::update::API_TIMEOUT)
        .get(format!("{BASE}{name}")).call().ok()?;
    limited(response.into_body().into_with_config().limit(LIMIT+1).reader())
}
fn load(root: &Path, online: bool, mut fetch: impl FnMut(&str) -> Option<Vec<u8>>) -> (Logos, bool) {
    let dir = root.join("Cache/shell-logos/0fe8f5c");
    let logos = ASSETS.map(|(name, mould)| {
        let path = dir.join(name);
        if let Some(logo) = std::fs::File::open(&path).ok().and_then(limited)
            .and_then(|bytes| extract_shell_logo(&bytes, mould)) { return Some(logo); }
        if !online { return None; }
        let bytes = fetch(name)?;
        let logo = extract_shell_logo(&bytes, mould)?;
        if std::fs::create_dir_all(&dir).is_ok() { let _ = slot_store::atomic_write(&path, &bytes); }
        Some(logo)
    });
    let complete = logos.iter().all(Option::is_some);
    (logos, complete)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn offline_uses_cache_and_invalid_cache_is_replaced_only_after_validation() {
        let root = tempfile::tempdir().unwrap();
        let fixture = include_bytes!("../../../assets/gba_neutral.png").to_vec();
        let (logos, complete) = load(root.path(), false, |_| panic!("offline fetch"));
        assert!(!complete); assert!(logos.iter().all(Option::is_none));
        let (logos, _) = load(root.path(), true, |name| name.starts_with("gba").then(|| fixture.clone()));
        assert!(logos[0].is_some()); assert!(logos[1].is_some()); assert!(logos[2].is_none());
        let (logos, _) = load(root.path(), false, |_| panic!("cached fetch"));
        assert!(logos[0].is_some());
        let path = root.path().join("Cache/shell-logos/0fe8f5c/gb_neutral.png");
        std::fs::write(&path, b"broken").unwrap();
        let _ = load(root.path(), true, |_| Some(b"invalid download".to_vec()));
        assert_eq!(std::fs::read(path).unwrap(), b"broken");
    }
    #[test]
    fn download_and_cache_reads_are_bounded() {
        assert!(limited(std::io::repeat(0).take(LIMIT+1)).is_none());
        assert_eq!(limited(&b"small"[..]).unwrap(), b"small");
    }
}
