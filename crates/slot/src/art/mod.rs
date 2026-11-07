//! Cartridge artwork adapted from upstream Slot. All network and image work stays off the UI thread.
pub mod index;
pub mod make;
pub mod rom;

use std::{io::Read, path::Path};
use index::Look;
use slot_store::Cart;
use crate::update::{self, UpdateFail};

const ART_BASE: &str = "https://art.slot-cfw.fyi/";
const LIMIT: u64 = 16 * 1024 * 1024;
const INDEX_AGE: std::time::Duration = std::time::Duration::from_secs(24 * 60 * 60);
const ART_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(45);

#[derive(Clone, Default)]
pub struct Artwork {
    pub real: Option<Vec<u8>>,
    pub logo: Option<Vec<u8>>,
    pub wheel: Option<Vec<u8>>,
    pub look: Option<Look>,
}

impl Artwork {
    /// Compare decoded pixels, so PNG metadata and WebP encoding do not offer the same sticker twice.
    /// Keep the wheel even when its generated sticker is already worn: it enables recoloring.
    pub fn without_duplicates(&mut self, original: Option<&[u8]>) -> bool {
        use slot_ui::house_label::{glance, looks_same};
        let mut seen: Vec<Vec<u8>> = original.and_then(glance).into_iter().collect();
        let mut duplicates = false;
        for bytes in [&mut self.real, &mut self.logo] {
            if let Some(look) = bytes.as_deref().and_then(glance) {
                if seen.iter().any(|old| looks_same(old, &look)) {
                    *bytes = None;
                    duplicates = true;
                } else { seen.push(look); }
            }
        }
        duplicates
    }
}

pub fn safe_asset(path: &str) -> bool {
    !path.is_empty() && !path.starts_with('/') && !path.contains(['\\', '?', '#', ':', '%', '\r', '\n'])
        && path.split('/').all(|p| !p.is_empty() && p != "." && p != "..")
}
fn get(agent: &ureq::Agent, url: &str) -> Result<Vec<u8>, UpdateFail> {
    let response = agent.get(url).call().map_err(|e| update::request_fail(&e))?;
    let mut bytes = Vec::new();
    response.into_body().into_with_config().limit(LIMIT).reader().take(LIMIT + 1)
        .read_to_end(&mut bytes).map_err(|_| UpdateFail::Download)?;
    if bytes.len() as u64 > LIMIT {return Err(UpdateFail::Bad);}
    Ok(bytes)
}
fn image(agent: &ureq::Agent, path: &str) -> Result<Vec<u8>, UpdateFail> {
    if !safe_asset(path) {return Err(UpdateFail::Bad);}
    if let Some(stem) = path.strip_suffix(".png") {
        if let Ok(bytes) = get(agent, &format!("{ART_BASE}{stem}.webp")) {return Ok(bytes);}
    }
    get(agent, &format!("{ART_BASE}{path}"))
}
pub fn fetch(root: &Path, cart: &Cart) -> Result<Artwork, UpdateFail> {
    // The catalog is shared by every game. Reuse a recent copy, but only retain
    // the current game's entry while parsing it on this worker.
    let crc = rom::rom_crc(&cart.rom).map_err(|_| UpdateFail::Bad)?;
    let cache = root.join("Cache/art/index.json");
    let recent = cache.metadata().ok().and_then(|m| m.modified().ok())
        .and_then(|at| at.elapsed().ok()).is_some_and(|age| age < INDEX_AGE);
    let cached = || std::fs::read(&cache).ok().filter(|b| b.len() as u64 <= LIMIT)
        .and_then(|b| index::find(&b, crc).ok());
    let entry = if recent { cached() } else { None };
    let entry = match entry {
        Some(entry) => entry,
        None => {
            if update::clock_is_wrong(update::now_unix()) {return Err(UpdateFail::Clock);}
            let index_agent = update::agent(ART_TIMEOUT);
            match get(&index_agent, &format!("{ART_BASE}index.json")) {
                Ok(bytes) => {
                    let entry = index::find(&bytes, crc).map_err(|_| UpdateFail::Bad)?;
                    if let Some(parent) = cache.parent() {let _ = std::fs::create_dir_all(parent);}
                    let _ = slot_store::atomic_write(&cache, &bytes);
                    entry
                }
                Err(error) => cached().ok_or(error)?,
            }
        }
    };
    let Some(entry) = entry else {return Ok(Artwork::default());};
    let agent = update::agent(ART_TIMEOUT);
    let mut art = Artwork {look: entry.look.clone(), ..Artwork::default()};
    if let Some(path) = &entry.real {art.real = image(&agent, path).ok().and_then(|b| make::real_label(&b, cart.platform));}
    if let Some(path) = &entry.logo {
        if let Ok(bytes) = image(&agent, path) {
            art.logo = make::logo_label(&bytes, entry.look.as_ref(), &cart.stem, cart.platform);
            art.wheel = make::as_png(&bytes).map(|b| b.into_owned());
        }
    }
    if art.real.is_none() && art.logo.is_none() {return Err(UpdateFail::Download);}
    Ok(art)
}
