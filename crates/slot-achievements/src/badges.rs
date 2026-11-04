//! Badge files are downloaded/validated by HTTP's worker and decoded by the UI raster
//! worker. Neither operation touches the emulator or render thread.
use std::collections::{BTreeMap,BTreeSet, VecDeque};
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

fn stamp(path:&Path)->Option<(u64,std::time::SystemTime)> {let m=path.metadata().ok()?;Some((m.len(),m.modified().ok()?))}

pub(crate) struct Queue {
    dir: PathBuf,
    names: BTreeSet<String>,
    known: BTreeSet<String>,
    pending: VecDeque<String>,
    checking:VecDeque<String>,
    discovery:VecDeque<PathBuf>,
    validated:BTreeMap<String,(u64,std::time::SystemTime)>,
    retry: Instant,
    pub waiting: bool,
    pub problem:&'static str,
    retry_delay:u64,
}

impl Queue {
    pub fn new(dir: &Path) -> Self {
        let mut queue = Self {
            dir: dir.into(),
            names: BTreeSet::new(),
            known: BTreeSet::new(),
            pending: VecDeque::new(),checking:VecDeque::new(),discovery:VecDeque::new(),validated:BTreeMap::new(),
            retry: Instant::now(),
            waiting: false,problem:"",retry_delay:30,
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
            // Known, unchanged files are never inflated for each referencing game.
            let fresh=self.known.insert(achievement.badge.clone());
            if !fresh && (self.names.contains(&achievement.badge) || self.validated.get(&achievement.badge).copied()==stamp(&path)) {continue;}
            self.names.insert(achievement.badge.clone());
            if path.is_file() {self.checking.push_back(achievement.badge.clone());}
            else {self.pending.push_back(achievement.badge.clone());}
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
                if !self.discovery.contains(&path) {self.discovery.push_back(path);}
            }
        }
    }

    /// Bounded cache work, independent of HTTP backoff. Never inflate an entire
    /// library's artwork in a single startup/rescan burst.
    pub fn maintain(&mut self) {
        let until=Instant::now()+Duration::from_millis(4);
        while Instant::now()<until {
            if let Some(path)=self.discovery.pop_front() {
                if let Ok((game,_))=storage::read::<(Game,BTreeSet<u32>)>(&path) {self.enqueue(&game);}
            } else if let Some(name)=self.checking.pop_front() {
                if path(&self.dir,&name).and_then(|p|load_badge(&p)).is_none() {
                    self.pending.push_back(name);
                } else {
                    if let Some(value)=path(&self.dir,&name).and_then(|p|stamp(&p)) {self.validated.insert(name.clone(),value);}
                    self.names.remove(&name);
                }
            } else {break;}
        }
    }
    pub fn discovery_pending(&self)->bool {!self.discovery.is_empty()}
    pub fn counts(&self) -> (usize, usize) {
        (self.known.len().saturating_sub(self.pending.len()+self.checking.len()), self.known.len())
    }
    pub fn network_available(&mut self) {self.retry=Instant::now();self.waiting=false;self.retry_delay=30;}
    pub fn retry_seconds(&self)->u64 {if self.waiting {self.retry.saturating_duration_since(Instant::now()).as_secs()}else{0}}
    pub fn percent(&self) -> u8 {
        if self.known.is_empty() {
            return 100;
        }
        ((self.known.len().saturating_sub(self.pending.len()+self.checking.len())) as u64 * 100
            / self.known.len() as u64) as u8
    }

    pub fn pending(&self) -> bool {
        !self.pending.is_empty() || !self.checking.is_empty() || !self.discovery.is_empty()
    }

    /// At most one bounded HTTP request per pass. Award uploads and game setup go first.
    /// Badge failures never invalidate authentication or delay achievement evaluation.
    pub fn step(&mut self, http: &mut impl Transport) {
        self.maintain();
        if Instant::now() < self.retry {
            return;
        }
        let Some(name) = self.pending.pop_front() else {
            return;
        };
        let target = path(&self.dir, &name).unwrap();
        let result = if load_badge(&target).is_some() {Ok(())}else {
            http.badge(&name).map_err(|e|if e==Failure::Network {http.network_problem()}else{"Artwork response invalid"}).and_then(|bytes| {
                decode(&bytes).ok_or("Artwork response invalid")?;
                std::fs::create_dir_all(target.parent().unwrap()).map_err(|_|"Cannot save artwork")?;
                slot_store::atomic_write(&target,&bytes).map_err(|_|"Cannot save artwork")
            })
        };
        if result.is_ok() {
            self.waiting=false;self.problem="";self.retry_delay=30;self.retry=Instant::now()+Duration::from_secs(3);
            if let Some(value)=stamp(&target) {self.validated.insert(name.clone(),value);}
            self.names.remove(&name);
            if self.pending.is_empty() {
                self.waiting = false;
            }
        } else {
            self.problem=result.unwrap_err();self.pending.push_back(name);
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
    fn duplicate_cached_artwork_is_discovered_without_eager_inflation() {
        let root=tempfile::tempdir().unwrap();std::fs::create_dir_all(root.path().join("badges")).unwrap();std::fs::write(root.path().join("badges/12345.png"),b"corrupt PNG").unwrap();
        let mut data:Game=serde_json::from_value(serde_json::json!({"ID":1,"Title":"Test","ConsoleID":5,"Achievements":[{"ID":1,"Title":"Test","Description":"Test","Points":5,"Flags":3,"BadgeName":"12345","MemAddr":"0xH000000=1"}]})).unwrap();
        data.achievements.extend(std::iter::repeat_n(data.achievements[0].clone(),1000));
        let mut q=Queue::new(root.path());q.enqueue(&data);q.enqueue(&data);
        assert_eq!(q.checking.len(),1);assert!(q.pending.is_empty(),"enqueue must not decode or download");
        q.maintain();assert_eq!(q.pending.len(),1,"full PNG validation must schedule repair");
    }
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
        // A restored connection must wake artwork as well as the main RA queue.
        q.network_available();assert!(!q.waiting);q.step(&mut http);assert_eq!(http.0,3);assert!(q.retry_seconds()<=30);
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
