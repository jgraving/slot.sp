use crate::{Canvas, Draw, MenuFrame, UndoFace, MENU_PAD};

pub const SWITCH_W: f32 = 60.0;
pub const SWITCH_H: f32 = 30.0;
pub const SWITCH_OFF: [f32; 4] = [0.33, 0.36, 0.40, 1.0];
pub const SWITCH_THUMB: [f32; 4] = [0.95, 0.97, 0.96, 1.0];

/// Geometry and palette are shared by live controls and cached static previews.
pub fn switch_style(t: f32) -> ([f32; 4], f32, f32) {
    let t = t.clamp(0.0, 1.0);
    (
        std::array::from_fn(|i| SWITCH_OFF[i] + (crate::expressive::ACCENT[i] - SWITCH_OFF[i]) * t),
        5.0 + 28.0 * t,
        20.0 + 2.0 * t,
    )
}
pub fn draw_switch(out: &mut Vec<Draw>, x: f32, y: f32, t: f32, alpha: f32) {
    let (mut track, thumb_x, size) = switch_style(t);
    track[3] *= alpha;
    let mut thumb = SWITCH_THUMB;
    thumb[3] *= alpha;
    crate::expressive::rounded(out, x, y, SWITCH_W, SWITCH_H, SWITCH_H / 2.0, track);
    crate::expressive::rounded(
        out,
        x + thumb_x,
        y + (SWITCH_H - size) / 2.0,
        size,
        size,
        size / 2.0,
        thumb,
    );
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MenuSwitch {
    pub id: u64,
    pub on: bool,
    pub y: f32,
}

/// Fallback raster for static exports. Live menus draw `draw_switch` instead.
pub fn toggle_face(on: bool, _lit: bool) -> UndoFace {
    let (w, h) = (SWITCH_W as u32 + 2 * MENU_PAD, 40);
    let mut face = UndoFace {
        rgba: vec![0; (w * h * 4) as usize],
        w,
        h,
    };
    let (track, thumb_x, size) = switch_style(if on { 1.0 } else { 0.0 });
    for y in 0..h {
        for x in MENU_PAD..w - MENU_PAD {
            let mut rgb = [0.0; 3];
            let mut covered = 0;
            for sy in 0..4 {
                for sx in 0..4 {
                    let px = x as f32 - MENU_PAD as f32 + (sx as f32 + 0.5) / 4.0;
                    let py = y as f32 - 5.0 + (sy as f32 + 0.5) / 4.0;
                    let cx = px.clamp(SWITCH_H / 2.0, SWITCH_W - SWITCH_H / 2.0);
                    if (px - cx).powi(2) + (py - SWITCH_H / 2.0).powi(2) > (SWITCH_H / 2.0).powi(2)
                    {
                        continue;
                    }
                    let thumb = (px - thumb_x - size / 2.0).powi(2) + (py - SWITCH_H / 2.0).powi(2)
                        <= (size / 2.0).powi(2);
                    let colour = if thumb { SWITCH_THUMB } else { track };
                    for c in 0..3 {
                        rgb[c] += colour[c];
                    }
                    covered += 1;
                }
            }
            if covered == 0 {
                continue;
            }
            let at = ((y * w + x) * 4) as usize;
            for c in 0..3 {
                face.rgba[at + c] = (rgb[c] / covered as f32 * 255.0).round() as u8;
            }
            face.rgba[at + 3] = (covered * 255 / 16) as u8;
        }
    }
    face
}

pub fn draw_menu_switches(
    canvas: Canvas,
    switches: &[MenuSwitch],
    motion: &crate::expressive::SwitchMotion,
    frame: MenuFrame,
    out: &mut Vec<Draw>,
) {
    let begin = out.len();
    let dx = frame.page_shift * (canvas.w - 2.0 * crate::expressive::MENU_INSET);
    for s in switches {
        draw_switch(
            out,
            canvas.w - crate::QUICK_EDGE - SWITCH_W + dx,
            s.y,
            motion.value(s.id, s.on),
            frame.opacity,
        );
    }
    crate::quick_menu::clip_quads(
        out,
        begin,
        [
            crate::expressive::MENU_INSET,
            crate::expressive::CARD_TOP,
            canvas.w - crate::expressive::MENU_INSET,
            crate::expressive::CARD_BOTTOM,
        ],
    );
}
