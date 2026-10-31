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
        let charging=self.text().is_some_and(|s|s.starts_with("Charging: "));
        let base = canvas.h - MOUTH_H - 8.0 - h;
        let y=base+(1.0-alpha)*(canvas.h-base+h+16.0);
        let begin=out.len();
        crate::expressive::rounded(out, x, y, w, h, h / 2.0, if charging {[0.16,0.29,0.23,1.0]}else{[0.10, 0.12, 0.14, 1.0]});
        if charging {
            let level=self.text().and_then(|s|s.strip_prefix("Charging: ")).and_then(|s|s.trim_end_matches('%').parse::<f32>().ok()).unwrap_or(0.0).clamp(0.0,100.0)/100.0;
            let mut fill=vec![];
            crate::expressive::rounded(&mut fill,x,y,w,h,h/2.0,[0.69,0.94,0.80,1.0]);
            crate::quick_menu::clip_quads(&mut fill,0,[x,y,x+w*level,y+h]);
            out.extend(fill);
        }
        out.push(Draw::Tex {
            x,
            y,
            w,
            h,
            tex,
            alpha:1.0,
        });
        // Floating feedback crosses the hardware/footer while entering. Only
        // the physical display boundary clips it, never a menu's viewport.
        crate::quick_menu::clip_quads(out,begin,[0.0,0.0,canvas.w,crate::OUT_H as f32]);
    }
}

pub fn settings_notice_face(message: &str, canvas: Canvas) -> UndoFace {
    if let Some(percent)=message.strip_prefix("Charging: ") {return charging_notice_face(percent);}
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

/// Filled charging chip with a native vector bolt, never a font substitute.
fn charging_notice_face(percent:&str)->UndoFace {
    let label=format!("{percent} · Charging");
    let font=text::font(text::Role::Value).expect("value font");
    let layout=text::fit_preserving_case(font,&label,300.0,1,22.0,22.0);
    let tw=text::line_width(font,&label,layout.px,layout.tracking).ceil() as u32;
    let (w,h)=(tw+76,52);
    let mut face=UndoFace{rgba:vec![0;(w*h*4) as usize],w,h};
    let mut words=vec![0;(tw*h*4) as usize];
    text::draw_centred(&mut words,tw,h,&layout,[16,48,35]);
    crate::plate::blit(&mut face.rgba,w,&words,tw,h,52,0);
    let svg=r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path fill="#103023" d="M13 1 4 14h7l-1 9 10-14h-7z"/></svg>"##;
    let bolt=crate::art::render_svg(svg,26,30).expect("charging bolt");
    crate::plate::blit(&mut face.rgba,w,&bolt,26,30,18,11);
    let level=percent.trim_end_matches('%').parse::<f32>().unwrap_or(0.0).clamp(0.0,100.0)/100.0;
    for (i,pixel) in face.rgba.chunks_exact_mut(4).enumerate(){if (i as u32%w)as f32>=w as f32*level && pixel[3]>0 {pixel[..3].copy_from_slice(&[218,250,230]);}}
    face
}
