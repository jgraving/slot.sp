//! Transient settings feedback, cached as text and animated without re-rasterising.
use crate::{text, Canvas, Draw, TexId, UndoFace, MOUTH_H};

#[derive(Default)]
pub struct SettingsNotice {
    source: Option<(u64, String)>,
    shown: u64,
    observed: Option<(u64,String)>,
    motion: crate::expressive::Spring,
    tick: u64,
}
impl SettingsNotice {
    /// A stable status expires once; another input attempt can supply a new key.
    pub fn observe(&mut self, source: Option<(u64, String)>, now: u64) {
        self.motion=self.motion_at(now); self.tick=now;
        if self.observed != source {
            if source.is_some() {self.source=source.clone();self.shown=now;}
            self.observed=source;
        }
        if self.observed.is_none() && self.motion.value==0.0 {self.source=None;}
    }
    fn motion_at(&self, now:u64)->crate::expressive::Spring {
        let mut spring=self.motion;
        let deadline=self.shown.saturating_add(2400);
        if self.observed.is_some() && self.tick<deadline && now>deadline {
            spring.step(1.0,(deadline-self.tick) as f32/1000.0,800.0,1.0);
            spring.step(0.0,(now-deadline) as f32/1000.0,800.0,1.0);
        } else {
            let target=if self.observed.is_some() && now<deadline {1.0}else{0.0};
            spring.step(target,now.saturating_sub(self.tick) as f32/1000.0,800.0,1.0);
        }
        spring
    }
    pub fn text(&self) -> Option<&str> {
        self.source.as_ref().map(|(_, text)| text.as_str())
    }
    /// Legacy name: this is spatial presence, never texture transparency.
    pub fn alpha(&self, now: u64) -> f32 {
        if self.source.is_none() {0.0}else{self.motion_at(now).value.clamp(0.0,1.0)}
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
        let base = canvas.h - MOUTH_H - 8.0 - h;
        let y=base+(1.0-alpha)*(canvas.h-base+h+16.0);
        let begin=out.len();
        crate::expressive::rounded(out, x, y, w, h, h / 2.0, [0.10, 0.12, 0.14, 0.97]);
        out.push(Draw::Tex {
            x,
            y,
            w,
            h,
            tex,
            alpha:1.0,
        });
        crate::quick_menu::clip_quads(out,begin,[16.0,40.0,canvas.w-16.0,canvas.h-MOUTH_H-8.0]);
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
