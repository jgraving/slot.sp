use crate::{MENU_PAD, UndoFace};

/// Shared on/off control. Colour and thumb position both show the state.
pub fn toggle_face(on: bool, lit: bool) -> UndoFace {
    let (w, h) = (80 + 2 * MENU_PAD, 40);
    let mut face = UndoFace { rgba: vec![0; (w * h * 4) as usize], w, h };
    let track = if on { [84u8, 194, 184] } else { [128u8, 128, 128] };
    let thumb_x = MENU_PAD as f32 + if on { 62.0 } else { 18.0 };
    for y in 0..h {
        for x in MENU_PAD..w - MENU_PAD {
            let mut rgb = [0u32; 3];
            let mut covered = 0;
            for sy in 0..4 {
                for sx in 0..4 {
                    let px = x as f32 + (sx as f32 + 0.5) / 4.0;
                    let py = y as f32 + (sy as f32 + 0.5) / 4.0;
                    let centre = px.clamp(MENU_PAD as f32 + 16.0, MENU_PAD as f32 + 64.0);
                    if (px - centre).powi(2) + (py - 20.0).powi(2) > 16.0 * 16.0 { continue; }
                    let thumb = (px - thumb_x).powi(2) + (py - 20.0).powi(2) <= 12.5 * 12.5;
                    let colour = if thumb { [250, 249, 246] } else { track };
                    for c in 0..3 { rgb[c] += u32::from(colour[c]); }
                    covered += 1;
                }
            }
            if covered == 0 { continue; }
            let at = ((y * w + x) * 4) as usize;
            for c in 0..3 { face.rgba[at + c] = (rgb[c] / covered) as u8; }
            face.rgba[at + 3] = (covered * if lit { 255 } else { 220 } / 16) as u8;
        }
    }
    face
}
