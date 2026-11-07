use std::borrow::Cow;
use std::sync::{OnceLock, RwLock};

use slot_gfx::OUT_W;

use crate::art::decode_png_bytes;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Mould {
    Gba,
    Notched,
    Rounded,
}

pub const GBA_CART_W: u32 = 360;
pub const GBA_CART_H: u32 = 212;
pub const GB_CART_W: u32 = 312;
pub const GB_CART_H: u32 = 337;
pub const BOARD_W: u32 = GBA_CART_W;
pub const BOARD_H: u32 = GBA_CART_H;
pub const SOCKET_W: u32 = 64;
pub const SOCKET_H: u32 = 46;
pub const CHIP_W: u32 = 59;
pub const CHIP_H: u32 = 41;
pub const SLOT_W: u32 = OUT_W;
pub const SLOT_H: u32 = 72;
/// Raster dimensions are separate from logical geometry so About can enlarge
/// the physical shell without magnifying a one-pixel source edge.
pub const SLOT_TEXTURE_W: u32 = SLOT_W * 2;
pub const SLOT_TEXTURE_H: u32 = SLOT_H * 2;
pub const ABOUT_BODY_W: u32 = 400;
pub const ABOUT_BODY_H: u32 = 360;
pub const ABOUT_BODY_TEXTURE_W: u32 = ABOUT_BODY_W * 2;
pub const ABOUT_BODY_TEXTURE_H: u32 = ABOUT_BODY_H * 2;
pub const PORT_W: u32 = OUT_W;
pub const PORT_H: u32 = 58;
pub const PLUG_W: u32 = 92;
pub const PLUG_H: u32 = 244;
pub const ADAPTER_W: u32 = 290;
pub const ADAPTER_H: u32 = 180;
pub const STICKER_W: u32 = 660;
pub const STICKER_H: u32 = 228;

const GBA_CART_PNG: &[u8] = include_bytes!("../../../assets/gba_cart.png");
const GB_CART_PNG: &[u8] = include_bytes!("../../../assets/gb_cart.png");
const GBC_CART_PNG: &[u8] = include_bytes!("../../../assets/gbc_cart.png");
const GBA_INTERIOR_PNG: &[u8] = include_bytes!("../../../assets/gba_interior.png");
const GB_INTERIOR_PNG: &[u8] = include_bytes!("../../../assets/gb_interior.png");
const GBC_INTERIOR_PNG: &[u8] = include_bytes!("../../../assets/gbc_interior.png");
const GB_BOARD_PNG: &[u8] = include_bytes!("../../../assets/gb_board.png");
const BOARD_PNG: &[u8] = include_bytes!("../../../assets/board.png");
const SOCKET_PNG: &[u8] = include_bytes!("../../../assets/socket.png");
const CHIP_PNG: &[u8] = include_bytes!("../../../assets/chip.png");
const SLOT_PLUGGED_DARK_PNG: &[u8] = include_bytes!("../../../assets/slot_plugged_dark.png");
const SLOT_PLUGGED_LIGHT_PNG: &[u8] = include_bytes!("../../../assets/slot_plugged_light.png");
const PLUG_HOST_PNG: &[u8] = include_bytes!("../../../assets/link_plug_host.png");
const PLUG_JOIN_PNG: &[u8] = include_bytes!("../../../assets/link_plug_join.png");
const ADAPTER_PNG: &[u8] = include_bytes!("../../../assets/link_adapter.png");
const STICKER_PNG: &[u8] = include_bytes!("../../../assets/sticker.png");

fn load(bytes: &'static [u8], w: u32, h: u32, slot: &'static OnceLock<Vec<u8>>) -> &'static [u8] {
    slot.get_or_init(|| {
        let (mut rgba, sw, sh) = decode_png_bytes(bytes).expect("a template png decodes");
        if sh == h && sw > w {
            rgba = crop_centre(&rgba, sw, sh, w);
        } else if (sw, sh) != (w, h) {
            rgba = scale(&rgba, sw, sh, w, h);
        }
        // Transparent pixels are often stored as white. The cart is sampled with a linear
        // filter, which would pull that white in along every edge.
        bleed(&mut rgba, w, h);
        rgba
    })
}

/// Copy an opaque neighbour's colour into each transparent pixel, leaving its alpha at 0.
fn bleed(rgba: &mut [u8], w: u32, h: u32) {
    let src = rgba.to_vec();
    let (w, h) = (w as i32, h as i32);
    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            if src[i + 3] != 0 {
                continue;
            }
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, ny) = (x + dx, y + dy);
                if nx < 0 || ny < 0 || nx >= w || ny >= h {
                    continue;
                }
                let n = ((ny * w + nx) * 4) as usize;
                if src[n + 3] == 0 {
                    continue;
                }
                rgba[i..i + 3].copy_from_slice(&src[n..n + 3]);
                break;
            }
        }
    }
}

fn crop_centre(src: &[u8], sw: u32, sh: u32, w: u32) -> Vec<u8> {
    let left = ((sw - w) / 2) as usize;
    let mut out = Vec::with_capacity((w * sh * 4) as usize);
    for row in src.chunks_exact(sw as usize * 4) {
        out.extend_from_slice(&row[left * 4..(left + w as usize) * 4]);
    }
    out
}

fn scale(src: &[u8], sw: u32, sh: u32, w: u32, h: u32) -> Vec<u8> {
    let mut out = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        let sy = y * sh / h;
        for x in 0..w {
            let sx = x * sw / w;
            let from = ((sy * sw + sx) * 4) as usize;
            let to = ((y * w + x) * 4) as usize;
            out[to..to + 4].copy_from_slice(&src[from..from + 4]);
        }
    }
    out
}

pub fn gba_cart() -> &'static [u8] {
    static P: OnceLock<Vec<u8>> = OnceLock::new();
    load(GBA_CART_PNG, GBA_CART_W, GBA_CART_H, &P)
}

pub fn gb_cart() -> &'static [u8] {
    static P: OnceLock<Vec<u8>> = OnceLock::new();
    load(GB_CART_PNG, GB_CART_W, GB_CART_H, &P)
}

pub fn gbc_cart() -> &'static [u8] {
    static P: OnceLock<Vec<u8>> = OnceLock::new();
    load(GBC_CART_PNG, GB_CART_W, GB_CART_H, &P)
}

/// The inside of a GBA cart, from `assets/gba_interior.png`. Same size as the cart and
/// the board, so the carousel can stack the three 1:1. Its colour is set by the game, with the
/// same overlay blend the shells use.
pub fn gba_interior() -> &'static [u8] {
    static P: OnceLock<Vec<u8>> = OnceLock::new();
    load(GBA_INTERIOR_PNG, BOARD_W, BOARD_H, &P)
}

/// The back of a Game Boy pak, from `assets/gb_interior.png`.
pub fn gb_interior() -> &'static [u8] {
    static P: OnceLock<Vec<u8>> = OnceLock::new();
    load(GB_INTERIOR_PNG, GB_CART_W, GB_CART_H, &P)
}

/// The back of a Game Boy Color pak, from `assets/gbc_interior.png`.
pub fn gbc_interior() -> &'static [u8] {
    static P: OnceLock<Vec<u8>> = OnceLock::new();
    load(GBC_INTERIOR_PNG, GB_CART_W, GB_CART_H, &P)
}

/// The Game Boy PCB, from `assets/gb_board.png`, drawn between the pak's back and front plastic.
pub fn gb_board() -> &'static [u8] {
    static P: OnceLock<Vec<u8>> = OnceLock::new();
    load(GB_BOARD_PNG, GB_CART_W, GB_CART_H, &P)
}

pub fn board() -> &'static [u8] {
    static P: OnceLock<Vec<u8>> = OnceLock::new();
    load(BOARD_PNG, BOARD_W, BOARD_H, &P)
}

pub fn socket() -> &'static [u8] {
    static P: OnceLock<Vec<u8>> = OnceLock::new();
    load(SOCKET_PNG, SOCKET_W, SOCKET_H, &P)
}

pub fn chip() -> &'static [u8] {
    static P: OnceLock<Vec<u8>> = OnceLock::new();
    load(CHIP_PNG, CHIP_W, CHIP_H, &P)
}

pub fn slot_back() -> &'static [u8] {
    crate::material::shell_slot_faces(slot_store::ShellColour::Graphite)[0]
}

pub fn slot_back_light() -> &'static [u8] {
    crate::material::shell_slot_faces(slot_store::ShellColour::Platinum)[0]
}

pub fn slot_front() -> &'static [u8] {
    crate::material::shell_slot_faces(slot_store::ShellColour::Graphite)[1]
}

pub fn slot_front_light() -> &'static [u8] {
    crate::material::shell_slot_faces(slot_store::ShellColour::Platinum)[1]
}

pub fn slot_plugged() -> &'static [u8] {
    static P: OnceLock<Vec<u8>> = OnceLock::new();
    load(SLOT_PLUGGED_DARK_PNG, SLOT_W, SLOT_H, &P)
}

pub fn slot_plugged_light() -> &'static [u8] {
    static P: OnceLock<Vec<u8>> = OnceLock::new();
    load(SLOT_PLUGGED_LIGHT_PNG, SLOT_W, SLOT_H, &P)
}

pub fn link_port() -> &'static [u8] {
    crate::material::shell_link_port(slot_store::ShellColour::Graphite)
}

pub fn link_port_light() -> &'static [u8] {
    crate::material::shell_link_port(slot_store::ShellColour::Platinum)
}

pub fn link_plug_host() -> &'static [u8] {
    static P: OnceLock<Vec<u8>> = OnceLock::new();
    load(PLUG_HOST_PNG, PLUG_W, PLUG_H, &P)
}

pub fn link_plug_join() -> &'static [u8] {
    static P: OnceLock<Vec<u8>> = OnceLock::new();
    load(PLUG_JOIN_PNG, PLUG_W, PLUG_H, &P)
}

pub fn link_adapter() -> &'static [u8] {
    static P: OnceLock<Vec<u8>> = OnceLock::new();
    load(ADAPTER_PNG, ADAPTER_W, ADAPTER_H, &P)
}

pub fn sticker() -> &'static [u8] {
    static P: OnceLock<Vec<u8>> = OnceLock::new();
    load(STICKER_PNG, STICKER_W, STICKER_H, &P)
}

// Shells are hand drawn. There is no Game Boy clear file yet, so a clear Game Boy pak uses
// the neutral shell at 80%.
const GBA_SHELL_PNG: &[u8] = include_bytes!("../../../assets/gba_neutral.png");
const GB_SHELL_PNG: &[u8] = include_bytes!("../../../assets/gb_neutral.png");
const GBC_SHELL_PNG: &[u8] = include_bytes!("../../../assets/gbc_neutral.png");
const GBA_CLEAR_PNG: &[u8] = include_bytes!("../../../assets/gba_clear.png");
const GB_CLEAR_PNG: &[u8] = include_bytes!("../../../assets/gb_neutral.png");
const GBC_CLEAR_PNG: &[u8] = include_bytes!("../../../assets/gbc_clear.png");

/// The neutral shell asset for a mould, opaque or the separate clear one. The game's colour is
/// blended onto it by the caller.
pub fn shell_asset(mould: Mould, clear: bool) -> Cow<'static, [u8]> {
    static GBA: OnceLock<Vec<u8>> = OnceLock::new();
    static GB: OnceLock<Vec<u8>> = OnceLock::new();
    static GBC: OnceLock<Vec<u8>> = OnceLock::new();
    static GBA_CLEAR: OnceLock<Vec<u8>> = OnceLock::new();
    static GB_CLEAR: OnceLock<Vec<u8>> = OnceLock::new();
    static GBC_CLEAR: OnceLock<Vec<u8>> = OnceLock::new();
    let (bytes, w, h, slot) = match (mould, clear) {
        (Mould::Gba, false) => (GBA_SHELL_PNG, GBA_CART_W, GBA_CART_H, &GBA),
        (Mould::Notched, false) => (GB_SHELL_PNG, GB_CART_W, GB_CART_H, &GB),
        (Mould::Rounded, false) => (GBC_SHELL_PNG, GB_CART_W, GB_CART_H, &GBC),
        (Mould::Gba, true) => (GBA_CLEAR_PNG, GBA_CART_W, GBA_CART_H, &GBA_CLEAR),
        (Mould::Notched, true) => (GB_CLEAR_PNG, GB_CART_W, GB_CART_H, &GB_CLEAR),
        (Mould::Rounded, true) => (GBC_CLEAR_PNG, GB_CART_W, GB_CART_H, &GBC_CLEAR),
    };
    let fallback = load(bytes, w, h, slot);
    let key = shell_logo_key(mould, clear);
    let logos = SHELL_LOGOS.read().unwrap_or_else(|e| e.into_inner());
    let Some(logo) = logos[key].as_ref() else { return Cow::Borrowed(fallback); };
    Cow::Owned(apply_shell_logo(fallback, logo, mould))
}

static SHELL_LOGOS: RwLock<[Option<Vec<u8>>; 5]> = RwLock::new([None, None, None, None, None]);

fn shell_logo_key(mould: Mould, clear: bool) -> usize {
    match (mould, clear) {
        (Mould::Gba, false) => 0, (Mould::Gba, true) => 1,
        (Mould::Notched, _) => 2, (Mould::Rounded, false) => 3, (Mould::Rounded, true) => 4,
    }
}

pub fn shell_logo_band(mould: Mould) -> (u32, u32, u32, u32) {
    match mould {
        Mould::Gba => (76, 286, 23, 46),
        Mould::Notched => (47, 267, 41, 69),
        Mould::Rounded => (53, 263, 39, 70),
    }
}

/// Validate upstream shell artwork before retaining only its logo band.
/// No downloaded pixels outside that band enter the local shell geometry.
pub fn extract_shell_logo(bytes: &[u8], mould: Mould) -> Option<Vec<u8>> {
    let (w, h) = if mould == Mould::Gba { (GBA_CART_W, GBA_CART_H) } else { (GB_CART_W, GB_CART_H) };
    if bytes.len() < 24 || &bytes[..8] != b"\x89PNG\r\n\x1a\n" || &bytes[12..16] != b"IHDR" { return None; }
    let sw = u32::from_be_bytes(bytes[16..20].try_into().ok()?);
    let sh = u32::from_be_bytes(bytes[20..24].try_into().ok()?);
    if sh != h || sw < w || sw > w + 64 { return None; }
    let (rgba, dw, dh) = decode_png_bytes(bytes)?;
    if (dw, dh) != (sw, sh) { return None; }
    let offset = (sw-w)/2;
    let (left, right, top, bottom) = shell_logo_band(mould);
    let mut band = Vec::new();
    for y in top..bottom {
        let start = ((y*sw+offset+left)*4) as usize;
        band.extend_from_slice(&rgba[start..start+((right-left)*4) as usize]);
    }
    Some(band)
}

/// Install validated bands from the handheld's background downloader.
pub fn install_shell_logos(logos: [Option<Vec<u8>>; 5]) {
    *SHELL_LOGOS.write().unwrap_or_else(|e| e.into_inner()) = logos;
}

fn apply_shell_logo(fallback: &[u8], logo: &[u8], mould: Mould) -> Vec<u8> {
    let w = if mould == Mould::Gba { GBA_CART_W } else { GB_CART_W };
    let (left, right, top, bottom) = shell_logo_band(mould);
    let mut out = fallback.to_vec();
    if logo.len() != ((right-left)*(bottom-top)*4) as usize { return out; }
    for (row, y) in (top..bottom).enumerate() {
        for x in left..right {
            let i = ((y*w+x)*4) as usize;
            let j = (row*(right-left) as usize+(x-left) as usize)*4;
            out[i..i+3].copy_from_slice(&logo[j..j+3]);
        }
    }
    out
}

/// Source button artwork. Menu hints downsample uniformly to their own height.
#[derive(Copy, Clone)]
pub(crate) enum ButtonLegend {
    A,
    B,
    Left,
    Right,
    Select,
    X,
    Y,
    Up,
    Down,
    Start,
    Menu,
    L, R, L1, R1, L2, R2, VolumeDown, VolumeUp,
}

#[cfg(test)]
mod shell_logo_tests {
    use super::*;
    #[test]
    fn logos_change_only_the_band_and_preserve_all_alpha() {
        for mould in [Mould::Gba, Mould::Notched, Mould::Rounded] {
            let (w,h) = if mould == Mould::Gba { (GBA_CART_W,GBA_CART_H) } else { (GB_CART_W,GB_CART_H) };
            let fallback = vec![100; (w*h*4) as usize];
            let (l,r,t,b) = shell_logo_band(mould);
            let logo = vec![50; ((r-l)*(b-t)*4) as usize];
            let out = apply_shell_logo(&fallback, &logo, mould);
            for y in 0..h { for x in 0..w {
                let i=((y*w+x)*4) as usize;
                assert_eq!(out[i+3],100);
                assert_eq!(out[i],if (l..r).contains(&x) && (t..b).contains(&y) {50} else {100});
            }}
            assert_eq!(apply_shell_logo(&fallback, &[], mould), fallback);
        }
    }
    #[test]
    fn extraction_rejects_wrong_geometry_and_invalid_data() {
        assert!(extract_shell_logo(GBA_SHELL_PNG, Mould::Gba).is_some());
        assert!(extract_shell_logo(GB_SHELL_PNG, Mould::Notched).is_some());
        assert!(extract_shell_logo(GBA_SHELL_PNG, Mould::Notched).is_none());
        assert!(extract_shell_logo(b"not png", Mould::Gba).is_none());
        let mut huge=GBA_SHELL_PNG.to_vec();
        huge[16..20].copy_from_slice(&u32::MAX.to_be_bytes());
        assert!(extract_shell_logo(&huge, Mould::Gba).is_none());
    }
}
