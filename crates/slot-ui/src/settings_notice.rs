//! Transient settings feedback, cached as text and animated without re-rasterising.
use crate::{text, Canvas, Draw, TexId, UndoFace, MOUTH_H};

#[derive(Default)]
pub struct SettingsNotice {
    source: Option<(u64, String)>,
    shown: u64,
}
impl SettingsNotice {
    /// A stable status expires once; another input attempt can supply a new key.
    pub fn observe(&mut self, source: Option<(u64, String)>, now: u64) {
        if self.source != source {
            self.source = source;
            self.shown = now;
        }
    }
    pub fn text(&self) -> Option<&str> {
        self.source.as_ref().map(|(_, text)| text.as_str())
    }
    pub fn alpha(&self, now: u64) -> f32 {
        if self.source.is_none() {
            return 0.0;
        }
        let age = now.saturating_sub(self.shown) as f32;
        let arrival = (age / 140.0).clamp(0.0, 1.0);
        let departure = ((2800.0 - age) / 220.0).clamp(0.0, 1.0);
        arrival * arrival * (3.0 - 2.0 * arrival) * departure
    }
    pub fn draw(
        &self,
        canvas: Canvas,
        face: Option<(TexId, u32, u32)>,
        now: u64,
        out: &mut Vec<Draw>,
    ) {
        let alpha = self.alpha(now);
        let Some((tex, w, h)) = face.filter(|_| alpha > 0.0) else {
            return;
        };
        let (w, h) = (w as f32, h as f32);
        let x = canvas.centred(w);
        let y = canvas.h - MOUTH_H - 8.0 - h;
        crate::expressive::rounded(out, x, y, w, h, h / 2.0, [0.10, 0.12, 0.14, 0.97 * alpha]);
        out.push(Draw::Tex {
            x,
            y,
            w,
            h,
            tex,
            alpha,
        });
    }
}

pub fn settings_notice_face(message: &str, canvas: Canvas) -> UndoFace {
    let Some(font) = text::font(text::Role::Detail) else {
        return UndoFace {
            rgba: vec![],
            w: 0,
            h: 0,
        };
    };
    let layout =
        text::fit_preserving_case(font, message, (canvas.w - 112.0).min(520.0), 2, 18.0, 16.0);
    let width = layout
        .lines
        .iter()
        .map(|line| text::line_width(font, line, layout.px, layout.tracking))
        .fold(0.0_f32, f32::max);
    let w = (width.ceil() as u32 + 32).min(canvas.w as u32 - 64).max(64);
    let h = 24 * layout.lines.len().max(1) as u32 + 20;
    let mut face = UndoFace {
        rgba: vec![0; (w * h * 4) as usize],
        w,
        h,
    };
    text::draw_centred(&mut face.rgba, w, h, &layout, [246, 244, 239]);
    face
}
