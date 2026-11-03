//! Badge files are downloaded/validated by HTTP's worker and decoded by the UI raster
//! worker. Neither operation touches the emulator or render thread.
use std::collections::{BTreeSet, VecDeque};
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::network::{Failure, Transport};
use crate::storage::{self, Game};

pub(crate) const MAX_BYTES: u64 = 128 * 1024;

pub struct BadgeImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

pub(crate) fn path(dir: &Path, name: &str) -> Option<PathBuf> {
    // API names are identifiers, never URLs or paths. Keep leading zeroes intact.
    (!name.is_empty()
        && name.len() <= 64
        && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_'))
    .then(|| dir.join("badges").join(format!("{name}.png")))
}

pub fn load_badge(path: &Path) -> Option<BadgeImage> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .ok()?
        .take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    decode(&bytes)
}

pub(crate) fn decode(bytes: &[u8]) -> Option<BadgeImage> {
    if bytes.len() as u64 > MAX_BYTES {
        return None;
    }
    let mut decoder = png::Decoder::new(Cursor::new(bytes));
    decoder.set_limits(png::Limits { bytes: 1024 * 1024 });
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().ok()?;
    let info = reader.info();
    if info.width == 0 || info.height == 0 || info.width > 256 || info.height > 256 {
        return None;
    }
    let mut data = vec![0; reader.output_buffer_size()];
    let frame = reader.next_frame(&mut data).ok()?;
    let mut rgba = Vec::with_capacity((frame.width * frame.height * 4) as usize);
    for pixel in data[..frame.buffer_size()].chunks_exact(frame.color_type.samples()) {
        match frame.color_type {
            png::ColorType::Rgba => rgba.extend_from_slice(pixel),
            png::ColorType::Rgb => rgba.extend_from_slice(&[pixel[0], pixel[1], pixel[2], 255]),
            png::ColorType::Grayscale => {
                rgba.extend_from_slice(&[pixel[0], pixel[0], pixel[0], 255])
            }
            png::ColorType::GrayscaleAlpha => {
                rgba.extend_from_slice(&[pixel[0], pixel[0], pixel[0], pixel[1]])
            }
            png::ColorType::Indexed => return None,
        }
    }
    Some(BadgeImage {
        width: frame.width,
        height: frame.height,
        rgba,
    })
}

pub(crate) struct Queue {
    dir: PathBuf,
    names: BTreeSet<String>,
    known: BTreeSet<String>,
    pending: VecDeque<String>,
    retry: Instant,
    pub waiting: bool,
    retry_delay:u64,
}

impl Queue {
    pub fn new(dir: &Path) -> Self {
        let mut queue = Self {
            dir: dir.into(),
            names: BTreeSet::new(),
            known: BTreeSet::new(),
            pending: VecDeque::new(),
            retry: Instant::now(),
            waiting: false,retry_delay:30,
        };
        queue.scan();
        queue
    }

    pub fn enqueue(&mut self, game: &Game) {
        for achievement in &game.achievements {
            if achievement.flags != 3 {
                continue;
            }
            let Some(path) = path(&self.dir, &achievement.badge) else {
                continue;
            };
            self.known.insert(achievement.badge.clone());
            if !self.names.contains(&achievement.badge) && load_badge(&path).is_none() {
                self.names.insert(achievement.badge.clone());
                self.pending.push_back(achievement.badge.clone());
            }
        }
    }

    pub fn scan(&mut self) {
        for entry in std::fs::read_dir(&self.dir).into_iter().flatten().flatten() {
            let path = entry.path();
            let hash = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            if hash.len() == 32
                && hash.bytes().all(|b| b.is_ascii_hexdigit())
                && path.extension().is_some_and(|e| e == "json")
            {
                if let Ok((game, _)) = storage::read::<(Game, BTreeSet<u32>)>(&path) {
                    self.enqueue(&game);
                }
            }
        }
    }

    pub fn counts(&self) -> (usize, usize) {
        (self.known.len().saturating_sub(self.pending.len()), self.known.len())
    }
    pub fn retry_seconds(&self)->u64 {if self.waiting {self.retry.saturating_duration_since(Instant::now()).as_secs()}else{0}}
    pub fn percent(&self) -> u8 {
        if self.known.is_empty() {
            return 100;
        }
        ((self.known.len().saturating_sub(self.pending.len())) as u64 * 100
            / self.known.len() as u64) as u8
    }

    pub fn pending(&self) -> bool {
        !self.pending.is_empty()
    }

    /// At most one bounded HTTP request per pass. Award uploads and game setup go first.
    /// Badge failures never invalidate authentication or delay achievement evaluation.
    pub fn step(&mut self, http: &mut impl Transport) {
        if Instant::now() < self.retry {
            return;
        }
        let Some(name) = self.pending.pop_front() else {
            return;
        };
        let target = path(&self.dir, &name).unwrap();
        let result = if load_badge(&target).is_some() {
            Ok(())
        } else {
            http.badge(&name).and_then(|bytes| {
                decode(&bytes).ok_or(Failure::Invalid)?;
                std::fs::create_dir_all(target.parent().unwrap()).map_err(|_| Failure::Invalid)?;
                slot_store::atomic_write(&target, &bytes).map_err(|_| Failure::Invalid)
            })
        };
        if result.is_ok() {
            self.waiting=false;self.retry_delay=30;self.retry=Instant::now()+Duration::from_secs(3);
            self.names.remove(&name);
            if self.pending.is_empty() {
                self.waiting = false;
            }
        } else {
            self.pending.push_back(name);
            self.waiting = true;
            self.retry = Instant::now() + Duration::from_secs(self.retry_delay);
            self.retry_delay=(self.retry_delay*2).min(300);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_artwork_backs_off_without_repeated_http_calls() {
        struct Failing(usize);
        impl Transport for Failing {
            fn call(&mut self,_:&[(&str,String)])->Result<serde_json::Value,Failure>{panic!("artwork must not reauthenticate")}
            fn badge(&mut self,_:&str)->Result<Vec<u8>,Failure>{self.0+=1;Err(Failure::Network)}
        }
        let dir=tempfile::tempdir().unwrap();let mut q=Queue::new(dir.path());q.pending.push_back("25000".into());q.known.insert("25000".into());
        let mut http=Failing(0);q.step(&mut http);assert_eq!(http.0,1);assert!(q.waiting);assert!(q.retry_seconds()>=29);
        for _ in 0..100 {q.step(&mut http);}assert_eq!(http.0,1);
        q.retry=Instant::now();q.step(&mut http);assert_eq!(http.0,2);assert!(q.retry_seconds()>=59);
    }
    #[test]
    #[ignore = "fetches one public RA badge; no account or award request"]
    fn public_ra_badge_endpoint() {
        let bytes = crate::network::Http::new().badge("25000").unwrap();
        let image = decode(&bytes).expect("RA badge must decode");
        assert!(image.width > 0 && image.height > 0);
        if let Some(path) = std::env::var_os("SLOT_RA_PREVIEW_BADGE") {
            std::fs::write(path, bytes).unwrap();
        }
    }
}
