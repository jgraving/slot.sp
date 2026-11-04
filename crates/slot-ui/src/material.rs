//! Satin painted plastic shared by the carousel housing and power switch.
use std::sync::OnceLock;
use slot_store::ShellColour;

pub fn shell_rgb(colour: ShellColour) -> [u8; 3] {
    match colour {
        ShellColour::Theme | ShellColour::Graphite => [40, 43, 49],
        ShellColour::Platinum => [154, 160, 168],
        ShellColour::PearlBlue => [119, 150, 170],
        ShellColour::PearlPink => [198, 155, 170],
    }
}

pub(crate) fn paint_grain(x: usize, y: usize) -> f32 {
    static GRAIN: OnceLock<Vec<u8>> = OnceLock::new();
    let grain = GRAIN.get_or_init(|| {
        let src = crate::art::decode_png_bytes(include_bytes!("../../../assets/rgsp_paint_grain.png"))
            .expect("embedded RG SP paint sample").0;
        // Three source pixels per display pixel: fine paint, not oversized pebbles.
        crate::art::cover_rgba(&src, 128, 128, 43, 43).expect("paint grain dimensions")
    });
    (grain[((y % 43) * 43 + x % 43) * 4] as f32 - 128.0) * 0.7
}

/// Apply paint to red-tagged SVG surfaces while leaving hardware neutral.
/// Origin and density keep grain registered across the slot/body seam.
fn painted_svg(bytes: &str, width: u32, height: u32, origin: (usize, usize), density: usize, base: [u8; 3]) -> Vec<u8> {
    let mut rgba = crate::art::render_svg(bytes, width, height).expect("embedded SVG shell geometry");
    for (at, pixel) in rgba.chunks_exact_mut(4).enumerate() {
        let painted = pixel[0] as f32 > pixel[1] as f32 * 1.5;
        let relief = (pixel[0] as f32 - 174.0) * 0.68;
        // Antialiasing mixes the red paint marker with neutral metal at edges.
        let neutral_edge = !painted && pixel[0] > pixel[1];
        let neutral = pixel[1].max(pixel[2]);
        let x = at % width as usize / density + origin.0;
        let y = at / width as usize / density + origin.1;
        let grain = paint_grain(x, y);
        for c in 0..3 {
            pixel[c] = if painted {
                (base[c] as f32 + relief + grain).clamp(0.0, 255.0) as u8
            } else if neutral_edge { neutral } else { pixel[c] };
        }
    }
    rgba
}

pub fn shell_slot_faces(colour: ShellColour) -> [&'static [u8]; 2] {
    static FACES: [OnceLock<[Vec<u8>; 2]>; 4] = [const { OnceLock::new() }; 4];
    let index = colour.finish_index();
    let faces = FACES[index].get_or_init(|| {
        let base = shell_rgb(ShellColour::FINISHES[index]);
        [
            include_str!("../assets/slot_back.svg"),
            include_str!("../assets/slot_front.svg"),
        ].map(|bytes| painted_svg(bytes, crate::bitmaps::SLOT_TEXTURE_W,
            crate::bitmaps::SLOT_TEXTURE_H, (0, 0), 2, base))
    });
    [&faces[0], &faces[1]]
}

/// Lower SP shell, starting directly after the slot's 72 logical pixels.
/// This is cached once per finish, never rasterized during animation.
pub fn shell_about_body(colour: ShellColour) -> &'static [u8] {
    static BODIES: [OnceLock<Vec<u8>>; 4] = [const { OnceLock::new() }; 4];
    let index = colour.finish_index();
    BODIES[index].get_or_init(|| painted_svg(
        include_str!("../assets/about_body.svg"),
        crate::bitmaps::ABOUT_BODY_TEXTURE_W, crate::bitmaps::ABOUT_BODY_TEXTURE_H,
        (160, crate::bitmaps::SLOT_H as usize), 2, shell_rgb(ShellColour::FINISHES[index]),
    ))
}

/// The close-up EXT.1 face shares the cartridge bay and switch paint, not a
/// theme-specific photographic texture. The connector occlusion strip stays 58px.
pub fn shell_link_port(colour: ShellColour) -> &'static [u8] {
    static PORTS: [OnceLock<Vec<u8>>; 4] = [const { OnceLock::new() }; 4];
    let index=colour.finish_index();
    PORTS[index].get_or_init(|| {
        let base=shell_rgb(ShellColour::FINISHES[index]);
        let mut rgba=crate::art::render_svg(include_str!("../assets/link_port.svg"),720,58)
            .expect("embedded EXT.1 port geometry");
        for (at,pixel) in rgba.chunks_exact_mut(4).enumerate() {
            let relief=(pixel[0] as f32-174.0)*0.68;
            let grain=paint_grain(at%720,at/720);
            for c in 0..3 {pixel[c]=(base[c] as f32+relief+grain).clamp(0.0,255.0) as u8;}
        }
        rgba
    })
}
