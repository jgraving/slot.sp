//! Launcher-only play history. The ROM filename and platform remain the game identity.
use std::{collections::BTreeMap, io, path::Path};
use crate::{atomic_write, Cart, Platform};

const FILE: &str = "Config/play-history.txt";
const HEADER: &str = "slot-play-history-v1";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FavoriteSort {
    #[default]
    Alphabetical,
    MostUsed,
    RecentlyUsed,
}
impl FavoriteSort {
    pub fn key(self) -> &'static str {
        match self { Self::Alphabetical => "alphabetical", Self::MostUsed => "most-used", Self::RecentlyUsed => "recently-used" }
    }
    pub fn parse(key: &str) -> Option<Self> {
        match key { "alphabetical" => Some(Self::Alphabetical), "most-used" => Some(Self::MostUsed), "recently-used" => Some(Self::RecentlyUsed), _ => None }
    }
    pub fn step(self, right: bool) -> Self {
        let values = [Self::Alphabetical, Self::MostUsed, Self::RecentlyUsed];
        values[(self as usize + if right { 1 } else { 2 }) % 3]
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PlayRecord {
    pub milliseconds: u64,
    pub last_utc: Option<i64>,
    pub order: u64,
}

#[derive(Clone, Default, Debug)]
pub struct PlayHistory {
    entries: BTreeMap<(u8, String), PlayRecord>,
    order: u64,
}
impl PlayHistory {
    pub fn read(root: &Path) -> io::Result<Self> {
        let path = root.join(FILE);
        let size = match std::fs::metadata(&path) {
            Ok(m) => m.len(),
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(e) => return Err(e),
        };
        if size > 16 * 1024 * 1024 { return Err(invalid()); }
        let text = std::fs::read_to_string(path)?;
        let mut lines = text.lines();
        if lines.next() != Some(HEADER) { return Err(invalid()); }
        let mut history = Self::default();
        for line in lines {
            let fields: Vec<_> = line.split('\t').collect();
            if fields.len() != 5 { return Err(invalid()); }
            let platform = Platform::ALL.into_iter().find(|p| p.dir_name() == fields[0]).ok_or_else(invalid)?;
            let stem = decode(fields[1]).ok_or_else(invalid)?;
            let milliseconds = fields[2].parse().map_err(|_| invalid())?;
            let utc: i64 = fields[3].parse().map_err(|_| invalid())?;
            let order = fields[4].parse().map_err(|_| invalid())?;
            let record = PlayRecord { milliseconds, last_utc: (utc >= 1_577_836_800).then_some(utc), order };
            if history.entries.insert((platform as u8, stem), record).is_some() { return Err(invalid()); }
            history.order = history.order.max(order);
        }
        Ok(history)
    }

    pub fn save(&self, root: &Path) -> io::Result<()> {
        let mut text = format!("{HEADER}\n");
        for ((platform, stem), record) in &self.entries {
            let platform = Platform::ALL.into_iter().find(|p| *p as u8 == *platform).unwrap();
            text.push_str(&format!("{}\t{}\t{}\t{}\t{}\n", platform.dir_name(), encode(stem), record.milliseconds, record.last_utc.unwrap_or(0), record.order));
        }
        std::fs::create_dir_all(root.join("Config"))?;
        atomic_write(&root.join(FILE), text.as_bytes())
    }

    pub fn get(&self, platform: Platform, stem: &str) -> PlayRecord {
        self.entries.get(&(platform as u8, stem.to_owned())).copied().unwrap_or_default()
    }

    /// Called once when a game actually begins a session, never when merely highlighted.
    pub fn started(&mut self, platform: Platform, stem: &str, utc: Option<i64>) {
        self.order = self.order.saturating_add(1);
        let record = self.entries.entry((platform as u8, stem.to_owned())).or_default();
        record.order = self.order;
        record.last_utc = utc.filter(|v| *v >= 1_577_836_800);
    }

    pub fn add_time(&mut self, platform: Platform, stem: &str, milliseconds: u64) {
        let record = self.entries.entry((platform as u8, stem.to_owned())).or_default();
        record.milliseconds = record.milliseconds.saturating_add(milliseconds);
    }

    pub fn sort(&self, carts: &mut [Cart], mode: FavoriteSort) {
        carts.sort_by(|a, b| {
            let a_record = self.get(a.platform, &a.stem);
            let b_record = self.get(b.platform, &b.stem);
            let rank = match mode {
                FavoriteSort::Alphabetical => std::cmp::Ordering::Equal,
                FavoriteSort::MostUsed => b_record.milliseconds.cmp(&a_record.milliseconds),
                FavoriteSort::RecentlyUsed => b_record.order.cmp(&a_record.order),
            };
            rank.then_with(|| (crate::sort_key(&a.stem), a.platform as u8, &a.stem)
                .cmp(&(crate::sort_key(&b.stem), b.platform as u8, &b.stem)))
        });
    }
}

fn invalid() -> io::Error { io::Error::new(io::ErrorKind::InvalidData, "Invalid play history; original file kept") }
fn encode(stem: &str) -> String { stem.as_bytes().iter().map(|b| format!("{b:02x}")).collect() }
fn decode(hex: &str) -> Option<String> {
    if hex.is_empty() || hex.len() % 2 != 0 || !hex.is_ascii() { return None; }
    let bytes: Option<Vec<u8>> = hex.as_bytes().chunks_exact(2).map(|pair| {
        let a = (pair[0] as char).to_digit(16)?; let b = (pair[1] as char).to_digit(16)?;
        Some((a * 16 + b) as u8)
    }).collect();
    String::from_utf8(bytes?).ok()
}
