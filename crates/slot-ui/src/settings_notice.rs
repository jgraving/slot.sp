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
            if let Some((key,text))=&source {
                let same_connection=self.observed.as_ref().is_some_and(|(old_key,old_text)|
                    old_key==key && old_text.starts_with("Charging: ") && text.starts_with("Charging: "));
                self.source=source.clone();
                if !same_connection {self.shown=now;}
            }
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
        self.draw_on(canvas,face,now,false,out);
    }
    pub fn draw_on(&self,canvas:Canvas,face:Option<(TexId,u32,u32)>,now:u64,light:bool,out:&mut Vec<Draw>) {
        let alpha = self.alpha(now);
        let Some((tex, w, h)) = face.filter(|_| alpha > 0.0) else {
            return;
        };
        let charging=self.text().is_some_and(|s|s.starts_with("Charging: "));
        let (original_w, original_h) = (w as f32, h as f32);
        // Two smooth breaths after arrival, then a steady pill. Transform the
        // complete chip together so its measured fill and ink remain aligned.
        let age=now.saturating_sub(self.shown)as f32;
        let pulse=if charging && (140.0..1040.0).contains(&age) {
            let phase=(age-140.0)/450.0;
            (std::f32::consts::PI*phase.fract()).sin().powi(2)*(1.0-0.25*phase.floor())
        }else{0.0};
        let scale=1.0+0.035*pulse*alpha;
        let (w,h)=(original_w*scale,original_h*scale);
        let x = canvas.centred(w);
        let base = canvas.h - MOUTH_H - 8.0 - original_h;
        let y=base+(1.0-alpha)*(canvas.h-base+original_h+16.0)-(h-original_h)/2.0;
        let begin=out.len();
        let colours=crate::battery::charge_colours(light);
        crate::expressive::rounded(out, x, y, w, h, h / 2.0, if charging {colours.track}else{[0.10, 0.12, 0.14, 1.0]});
        if charging {
            let level=self.text().and_then(|s|s.strip_prefix("Charging: ")).and_then(|s|s.trim_end_matches('%').parse::<f32>().ok()).unwrap_or(0.0).clamp(0.0,100.0)/100.0;
            let mut fill=vec![];
            crate::expressive::rounded(&mut fill,x,y,w,h,h/2.0,colours.fill);
            crate::quick_menu::clip_quads(&mut fill,0,[x,y,x+w*level,y+h]);
            out.extend(fill);
            if pulse>0.0 {
                // Two soft packets of light flow into the bolt. Concentric
                // quads soften the glow without new textures or render passes;
                // the measured charge boundary and overlaid text stay intact.
                let phase=((age-140.0)/450.0).fract();
                let travel=phase*phase*(3.0-2.0*phase);
                let centre=x+w-31.0*scale-(w-62.0*scale)*travel;
                for ring in 0..8 {
                    let radius=(22.0-ring as f32*2.0)*scale;
                    crate::expressive::rounded(out,centre-radius,y+h/2.0-radius,2.0*radius,2.0*radius,radius,[1.0,1.0,1.0,0.012*pulse*alpha]);
                }
            }
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

pub fn settings_notice_face(message: &str, canvas: Canvas) -> UndoFace {settings_notice_face_on(message,canvas,false)}
pub fn settings_notice_face_on(message:&str,canvas:Canvas,light:bool)->UndoFace {
    if let Some(percent)=message.strip_prefix("Charging: ") {return charging_notice_face(percent,light);}
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
    text::draw_centred(&mut face.rgba, w, h, &layout, if light {slot_gfx::ui_palette::INK}else{[246,244,239]});
    face
}

/// Filled charging chip with a native vector bolt, never a font substitute.
fn charging_notice_face(percent:&str,light:bool)->UndoFace {
    let colours=crate::battery::charge_colours(light);
    let label=format!("{percent} · Charging");
    let font=text::font(text::Role::Value).expect("value font");
    let layout=text::fit_preserving_case(font,&label,300.0,1,22.0,22.0);
    let tw=text::line_width(font,&label,layout.px,layout.tracking).ceil() as u32;
    let (w,h)=(tw+76,52);
    let mut face=UndoFace{rgba:vec![0;(w*h*4) as usize],w,h};
    let mut words=vec![0;(tw*h*4) as usize];
    text::draw_centred(&mut words,tw,h,&layout,colours.filled_ink);
    crate::plate::blit(&mut face.rgba,w,&words,tw,h,52,0);
    let svg=format!(r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path fill="#{:02x}{:02x}{:02x}" d="M13 1 4 14h7l-1 9 10-14h-7z"/></svg>"##,colours.filled_ink[0],colours.filled_ink[1],colours.filled_ink[2]);
    let bolt=crate::art::render_svg(&svg,26,30).expect("charging bolt");
    crate::plate::blit(&mut face.rgba,w,&bolt,26,30,18,11);
    let level=percent.trim_end_matches('%').parse::<f32>().unwrap_or(0.0).clamp(0.0,100.0)/100.0;
    for (i,pixel) in face.rgba.chunks_exact_mut(4).enumerate(){if (i as u32%w)as f32>=w as f32*level && pixel[3]>0 {pixel[..3].copy_from_slice(&colours.empty_ink);}}
    face
}
