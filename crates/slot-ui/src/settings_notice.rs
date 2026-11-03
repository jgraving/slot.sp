//! Transient settings feedback, cached as text and animated without re-rasterising.
use crate::{text, Canvas, Draw, TexId, UndoFace, MOUTH_H};

#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub enum NoticeTone { #[default] Neutral, Warning, Danger }

#[derive(Default)]
pub struct SettingsNotice {
    source: Option<(u64, String)>,
    shown: u64,
    observed: Option<(u64,String)>,
    motion: crate::expressive::Spring,
    tick: u64,
    tone: NoticeTone,
}
impl SettingsNotice {
    pub fn set_tone(&mut self,tone:NoticeTone) {self.tone=tone;}
    pub fn tone(&self)->NoticeTone {self.tone}
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
        let deadline=if self.observed.as_ref().is_some_and(|(_,s)|s=="CONFIRM_UNENCRYPTED") {u64::MAX}else{self.shown.saturating_add(2400)};
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
        crate::expressive::rounded(out, x, y, w, h, h / 2.0, if charging {colours.track}else{match self.tone {NoticeTone::Neutral=>[0.10,0.12,0.14,1.0],NoticeTone::Warning=>[0.98,0.77,0.24,1.0],NoticeTone::Danger=>[0.65,0.08,0.12,1.0]}});
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
    settings_notice_face_toned_on(message,canvas,light,NoticeTone::Neutral)
}
pub fn settings_notice_face_toned_on(message:&str,canvas:Canvas,light:bool,tone:NoticeTone)->UndoFace {
    if message=="CONFIRM_UNENCRYPTED" {return unencrypted_confirmation_face();}
    if message=="PIN_BUTTONS" || message=="PIN_BUTTONS_MIN" {return pin_buttons_notice_face(light,message=="PIN_BUTTONS_MIN");}
    let prefix=if tone==NoticeTone::Neutral {0}else{36};
    if let Some(percent)=message.strip_prefix("Charging: ") {return charging_notice_face(percent,light);}
    let Some(font) = text::font(text::Role::Detail) else {
        return UndoFace {
            rgba: vec![],
            w: 0,
            h: 0,
        };
    };
    let layout =
        text::fit_preserving_case(font, message, (canvas.w - 112.0 - prefix as f32).min(520.0), 2, 18.0, 16.0);
    let width = layout
        .lines
        .iter()
        .map(|line| text::line_width(font, line, layout.px, layout.tracking))
        .fold(0.0_f32, f32::max);
    let w = (width.ceil() as u32 + 32 + prefix).min(canvas.w as u32 - 64).max(64);
    let h = 24 * layout.lines.len().max(1) as u32 + 20;
    let mut face = UndoFace {
        rgba: vec![0; (w * h * 4) as usize],
        w,
        h,
    };
    let ink=match tone {NoticeTone::Warning=>[45,33,8],NoticeTone::Danger=>[255,248,245],NoticeTone::Neutral=>if light {slot_gfx::ui_palette::INK}else{[246,244,239]}};
    let mut words=vec![0;((w-prefix)*h*4)as usize];
    text::draw_centred(&mut words,w-prefix,h,&layout,ink);
    crate::plate::blit(&mut face.rgba,w,&words,w-prefix,h,prefix,0);
    if prefix>0 {
        let outline=if tone==NoticeTone::Danger {r#"<circle cx="12" cy="12" r="9"/>"#}else{r#"<path d="M10.3 3.8a2 2 0 0 1 3.4 0l8 14a2 2 0 0 1-1.7 3H4a2 2 0 0 1-1.7-3z"/>"#};
        let svg=format!(r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><g fill="none" stroke="#{:02x}{:02x}{:02x}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">{outline}<path d="M12 8v5m0 3v.2"/></g></svg>"##,ink[0],ink[1],ink[2]);
        let icon=crate::art::render_svg(&svg,26,26).expect("severity icon");
        crate::plate::blit(&mut face.rgba,w,&icon,26,26,14,(h-26)/2);
    }
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

/// Only invalid PIN input requests this transient row of permitted controller icons.
fn pin_buttons_notice_face(light:bool,minimum:bool)->UndoFace {
    use crate::bitmaps::ButtonLegend::*;
    let buttons=[A,B,X,Y,Up,Down,Left,Right,L1,R1,L2,R2];
    let faces:Vec<_>=buttons.into_iter().map(|button|crate::quick_menu::controller_button_face(button,26)).collect();
    let label=minimum.then(|| {
        let layout=text::Layout{lines:vec!["4+ presses".into()],px:16.0,tracking:0.0,role:text::Role::Detail};
        let font=text::font(text::Role::Detail).expect("PIN hint font");
        let w=text::line_width(font,"4+ presses",16.0,0.0).ceil()as u32+2;
        let mut rgba=vec![0;(w*48*4)as usize];
        text::draw_centred(&mut rgba,w,48,&layout,if light {slot_gfx::ui_palette::INK}else{[246,244,239]});
        UndoFace{rgba,w,h:48}
    });
    let prefix=label.as_ref().map_or(0,|f|f.w+12);
    let width=faces.iter().map(|f|f.w).sum::<u32>()+8*11+32+prefix;
    let mut face=UndoFace{rgba:vec![0;(width*48*4)as usize],w:width,h:48};
    if let Some(label)=label {crate::plate::blit(&mut face.rgba,width,&label.rgba,label.w,label.h,16,0);}
    let mut x=16+prefix;
    for glyph in faces {
        let rgba=if light {slot_gfx::ui_palette::light_texture(&glyph.rgba)}else{glyph.rgba};
        crate::plate::blit(&mut face.rgba,width,&rgba,glyph.w,glyph.h,x,(48-glyph.h)/2);
        x+=glyph.w+8;
    }
    face
}

/// A pending choice stays on its selector until a deliberate confirm or cancellation.
fn unencrypted_confirmation_face() -> UndoFace {
    let (w,h)=(424,76);
    let mut face=UndoFace{rgba:vec![0;(w*h*4)as usize],w,h};
    let font=text::font(text::Role::Detail).expect("confirmation font");
    let ink=[45,33,8];
    let title=text::Layout{lines:vec!["Store unencrypted?".into()],px:20.0,tracking:0.0,role:text::Role::Detail};
    let mut line=vec![0;(w*32*4)as usize];
    text::draw_centred(&mut line,w,32,&title,ink);
    crate::plate::blit(&mut face.rgba,w,&line,w,32,0,2);
    let mut x=64;
    for (button,label) in [(crate::bitmaps::ButtonLegend::A,"Confirm"),(crate::bitmaps::ButtonLegend::B,"Cancel")] {
        let glyph=crate::quick_menu::controller_button_face(button,28);
        let rgba=slot_gfx::ui_palette::light_texture(&glyph.rgba);
        crate::plate::blit(&mut face.rgba,w,&rgba,glyph.w,glyph.h,x,40);
        x+=glyph.w+8;
        let tw=text::line_width(font,label,18.0,0.0).ceil()as u32+2;
        let mut words=vec![0;(tw*32*4)as usize];
        let layout=text::Layout{lines:vec![label.into()],px:18.0,tracking:0.0,role:text::Role::Detail};
        text::draw_centred(&mut words,tw,32,&layout,ink);
        crate::plate::blit(&mut face.rgba,w,&words,tw,32,x,38);
        x+=tw+28;
    }
    face
}
