//! The About label belongs to the same physical housing as the cartridge mouth.
//! One reversible spring moves the housing and recedes its live parent page.
use crate::{Canvas, Draw, TexId};
use crate::expressive::Spring;

pub const ABOUT_STICKER_W: f32 = 364.0;
pub const ABOUT_STICKER_Y: f32 = 66.0;
const REST_TOP: f32 = crate::OUT_H as f32 - crate::MOUTH_H;
const OPEN_TOP: f32 = 68.0;

#[derive(Default, Debug)]
pub struct AboutMotion {
    reveal: Spring,
    open: bool,
}

impl AboutMotion {
    pub fn update(&mut self, open: bool, dt: f32) {
        self.open = open;
        // Critically damped: immediate reversal retains both position and velocity.
        self.reveal.step(if open { 1.0 } else { 0.0 }, dt, 150.0, 1.0);
    }

    pub fn value(&self) -> f32 { self.reveal.value.clamp(0.0, 1.0) }
    pub fn active(&self) -> bool { self.open || self.value() > 0.0 }

    pub fn transform(&self, canvas: Canvas, draws: &mut [Draw]) {
        let t = self.value();
        let enlarged = ((canvas.w - 80.0) / crate::ABOUT_BODY_W as f32).min(1.5);
        let scale = 1.0 + (enlarged - 1.0) * t;
        let top = REST_TOP + (OPEN_TOP - REST_TOP) * t;
        transform(draws, canvas.w / 2.0, REST_TOP, scale, top - REST_TOP, false);
    }

    pub fn transform_parent(&self, canvas: Canvas, draws: &mut [Draw]) {
        let t = self.value();
        let anchor = (crate::expressive::CARD_TOP + crate::expressive::CARD_BOTTOM) / 2.0;
        transform(draws, canvas.w / 2.0, anchor, 1.0 - 0.18 * t, canvas.h * 0.6 * t, true);
    }

    pub fn clip_hardware(&self, canvas: Canvas, out: &mut Vec<Draw>, begin: usize) {
        // Tall GB/GBC paks pass behind the fixed title/status band. The shallow
        // GBA cap remains complete; its seated top is below this boundary.
        crate::quick_menu::clip_quads(out, begin, [0.0, crate::expressive::CARD_TOP * self.value(), canvas.w, canvas.h]);
    }
}

fn transform(draws: &mut [Draw], cx: f32, cy: f32, scale: f32, dy: f32, preserve_chrome: bool) {
    for draw in draws {
        if preserve_chrome && crate::expressive::is_chrome(draw) { continue; }
        match draw {
            Draw::Rect { x, y, w, h, .. } | Draw::RoundRect { x, y, w, h, .. }
            | Draw::Tex { x, y, w, h, .. } | Draw::Part { x, y, w, h, .. }
            | Draw::Turned { x, y, w, h, .. } => {
                *x = cx + (*x - cx) * scale;
                *y = cy + (*y - cy) * scale + dy;
                *w *= scale;
                *h *= scale;
            }
            _ => {}
        }
        if let Draw::RoundRect { radius, .. } = draw { *radius *= scale; }
    }
}

/// Body under the slot; the overlapping sticker hides the paint join just as
/// the original moulded shell does. Logical dimensions never follow raster DPI.
pub fn draw_about_body(canvas: Canvas, body: Option<TexId>, out: &mut Vec<Draw>) {
    if let Some(tex) = body {
        out.push(Draw::Tex { x: canvas.centred(crate::ABOUT_BODY_W as f32),
            y: REST_TOP + crate::SLOT_H as f32, w: crate::ABOUT_BODY_W as f32,
            h: crate::ABOUT_BODY_H as f32, tex, alpha: 1.0 });
    }
}

pub fn draw_about_sticker(canvas: Canvas, sticker: Option<TexId>, out: &mut Vec<Draw>) {
    if let Some(tex) = sticker {
        out.push(Draw::Tex { x: canvas.centred(ABOUT_STICKER_W), y: REST_TOP + ABOUT_STICKER_Y,
            w: ABOUT_STICKER_W, h: ABOUT_STICKER_W * crate::STICKER_H as f32 / crate::STICKER_W as f32,
            tex, alpha: 1.0 });
    }
}
