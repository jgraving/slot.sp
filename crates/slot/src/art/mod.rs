//! Cartridge artwork adapted from upstream Slot. All network and image work stays off the UI thread.
pub mod index;
pub mod make;
pub mod rom;

use std::{io::Read, path::Path};
use index::{ArtIndex, Look};
use slot_store::Cart;
use crate::update::{self, UpdateFail};

const ART_BASE: &str = "https://art.slot-cfw.fyi/";
const LIMIT: u64 = 16 * 1024 * 1024;

#[derive(Clone, Default)]
pub struct Artwork {
    pub real: Option<Vec<u8>>,
    pub logo: Option<Vec<u8>>,
    pub wheel: Option<Vec<u8>>,
    pub look: Option<Look>,
}

pub fn safe_asset(path: &str) -> bool {
    !path.is_empty() && !path.starts_with('/') && !path.contains(['\\', '?', '#', ':', '%', '\r', '\n'])
        && path.split('/').all(|p| !p.is_empty() && p != "." && p != "..")
}
fn get(url: &str) -> Result<Vec<u8>, UpdateFail> {
    let response = update::agent(update::API_TIMEOUT).get(url).call().map_err(|e| update::request_fail(&e))?;
    let mut bytes = Vec::new();
    response.into_body().into_with_config().limit(LIMIT).reader().take(LIMIT + 1)
        .read_to_end(&mut bytes).map_err(|_| UpdateFail::Download)?;
    if bytes.len() as u64 > LIMIT {return Err(UpdateFail::Bad);}
    Ok(bytes)
}
fn image(path: &str) -> Result<Vec<u8>, UpdateFail> {
    if !safe_asset(path) {return Err(UpdateFail::Bad);}
    if let Some(stem) = path.strip_suffix(".png") {
        if let Ok(bytes) = get(&format!("{ART_BASE}{stem}.webp")) {return Ok(bytes);}
    }
    get(&format!("{ART_BASE}{path}"))
}
pub fn fetch(root: &Path, cart: &Cart) -> Result<Artwork, UpdateFail> {
    if update::clock_is_wrong(update::now_unix()) {return Err(UpdateFail::Clock);}
    let cache = root.join("Cache/art/index.json");
    let parse = |bytes: &[u8]| std::str::from_utf8(bytes).ok().and_then(ArtIndex::parse);
    let index = match get(&format!("{ART_BASE}index.json")) {
        Ok(bytes) => {
            let index = parse(&bytes).ok_or(UpdateFail::Bad)?;
            if let Some(parent) = cache.parent() {let _ = std::fs::create_dir_all(parent);}
            let _ = slot_store::atomic_write(&cache, &bytes);
            index
        }
        Err(error) => std::fs::read(&cache).ok().and_then(|b| parse(&b)).ok_or(error)?,
    };
    let crc = rom::rom_crc(&cart.rom).map_err(|_| UpdateFail::Bad)?;
    let Some(entry) = index.get(crc) else {return Ok(Artwork::default());};
    let mut art = Artwork {look: entry.look.clone(), ..Artwork::default()};
    if let Some(path) = &entry.real {art.real = image(path).ok().and_then(|b| make::real_label(&b, cart.platform));}
    if let Some(path) = &entry.logo {
        if let Ok(bytes) = image(path) {
            art.logo = make::logo_label(&bytes, entry.look.as_ref(), &cart.stem, cart.platform);
            art.wheel = make::as_png(&bytes).map(|b| b.into_owned());
        }
    }
    if art.real.is_none() && art.logo.is_none() {return Err(UpdateFail::Download);}
    Ok(art)
}
