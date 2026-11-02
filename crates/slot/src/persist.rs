use std::path::{Path, PathBuf};

use slot_store::{atomic_write, write_slot_state, Core, Platform, SlotState, StateRing};

pub trait Snapshot {
    fn state(&self) -> Option<Vec<u8>>;
    fn save_ram(&self) -> Option<Vec<u8>>;
    fn thumb(&self) -> Option<Vec<u8>>;
    fn load(&self, state: Vec<u8>);

    fn resume_trusted(&self) -> bool {
        true
    }

    fn save_ram_trusted(&self) -> bool {
        true
    }
}

pub fn flush(
    root: &Path,
    platform: Platform,
    core: Core,
    stem: &str,
    state: Option<&[u8]>,
    sav: Option<&[u8]>,
) -> std::io::Result<()> {
    if let Some(state) = state {
        let ring = StateRing::new(root, platform, core, stem);
        if !matches!(ring.read_resume(), Ok(Some(old)) if old == state) {
            ring.write_resume(state)?;
        }
    }
    if let Some(sav) = sav {
        write_sav(root, platform, stem, sav)?;
    }
    Ok(())
}

pub fn eject(
    root: &Path,
    platform: Platform,
    core: Core,
    stem: &str,
    state: Option<&[u8]>,
    sav: Option<&[u8]>,
    slot: &SlotState,
) -> std::io::Result<()> {
    flush(root, platform, core, stem, state, sav)?;
    let mut slot = slot.clone();
    slot.cart = None;
    slot.cart_platform = None;
    write_slot_state(root, &slot)
}

pub fn write_sav(root: &Path, platform: Platform, stem: &str, sav: &[u8]) -> std::io::Result<bool> {
    let path = sav_path(root, platform, stem);
    if let Some(old) = read_sav(root, platform, stem)? {
        if old == sav {
            return Ok(false);
        }
    }
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    atomic_write(&path, sav)?;
    Ok(true)
}

pub fn read_sav(root: &Path, platform: Platform, stem: &str) -> std::io::Result<Option<Vec<u8>>> {
    let srm = crate::root::saves_dir(root)
        .join(platform.dir_name())
        .join(format!("{stem}.srm"));
    for path in [sav_path(root, platform, stem), srm] {
        match std::fs::read(&path) {
            Ok(bytes) if bytes.starts_with(b"#RZIP") => return unrzip(&bytes).map(Some),
            Ok(bytes) => return Ok(Some(bytes)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
    }
    Ok(None)
}

const RZIP_MAGIC: &[u8] = b"#RZIPv\x01#";
// Battery saves are small; refuse oversized containers before allocating decoded bytes.
const MAX_RZIP_SAVE: u64 = 16 << 20;

fn unrzip(bytes: &[u8]) -> std::io::Result<Vec<u8>> {
    let bad = || std::io::Error::new(std::io::ErrorKind::InvalidData, "bad rzip save");
    let header = bytes.get(..20).ok_or_else(bad)?;
    if &header[..8] != RZIP_MAGIC {
        return Err(bad());
    }
    let chunk_size = u32::from_le_bytes(header[8..12].try_into().map_err(|_| bad())?) as u64;
    let total = u64::from_le_bytes(header[12..20].try_into().map_err(|_| bad())?);
    if chunk_size == 0 || chunk_size > 64 << 20 || total == 0 || total > MAX_RZIP_SAVE {
        return Err(bad());
    }
    let mut rest = &bytes[20..];
    let mut out = Vec::new();
    while (out.len() as u64) < total {
        let (len, body) = rest.split_first_chunk::<4>().ok_or_else(bad)?;
        let len = u32::from_le_bytes(*len) as usize;
        if len == 0 {
            return Err(bad());
        }
        let chunk = body.get(..len).ok_or_else(bad)?;
        let expected = chunk_size.min(total - out.len() as u64) as usize;
        // The extra byte lets inflate finish its trailer, or detect an oversized chunk.
        let mut decoded = vec![0; expected + 1];
        let mut decoder = flate2::Decompress::new(true);
        let status = decoder
            .decompress(chunk, &mut decoded, flate2::FlushDecompress::Finish)
            .map_err(|_| bad())?;
        if status != flate2::Status::StreamEnd
            || decoder.total_in() != len as u64
            || decoder.total_out() != expected as u64
        {
            return Err(bad());
        }
        out.extend_from_slice(&decoded[..expected]);
        rest = &body[len..];
    }
    if rest.is_empty() {
        Ok(out)
    } else {
        Err(bad())
    }
}

pub fn read_resume(
    root: &Path,
    platform: Platform,
    core: Core,
    stem: &str,
) -> std::io::Result<Option<Vec<u8>>> {
    StateRing::new(root, platform, core, stem).read_resume()
}

fn sav_path(root: &Path, platform: Platform, stem: &str) -> PathBuf {
    crate::root::saves_dir(root)
        .join(platform.dir_name())
        .join(format!("{stem}.sav"))
}
