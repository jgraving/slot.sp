//! A spring-led gathering of the firmware letters, then a drop behind the bay.
use slot_gfx::{Canvas, Compositor, Draw, TexId};

pub const BOOT_MS: u64 = 1120;
const DROP_MS: u64 = 510;
const LAND_MS: u64 = 1010;
const INK: [u8; 3] = [246,247,242];
// Blank columns between the seven glyphs in the native firmware bitmap.
const LETTER_EDGES: [f32; 8] = [0.0, 70.0, 94.0, 160.0, 202.0, 228.0, 284.0, 363.0];
fn smooth(t: f32) -> f32 { let t=t.clamp(0.0,1.0); t*t*t*(t*(t*6.0-15.0)+10.0) }
fn mix(a: f32,b: f32,t: f32)->f32 { a+(b-a)*t }

/// Closed-form unit-mass spring: elapsed time, not frame count, determines the
/// pose. M3 Expressive default spatial uses stiffness 380 and damping 0.8.
/// https://m3.material.io/styles/motion/overview/how-it-works
fn spatial(ms: f32) -> f32 {
    let t=ms.max(0.0)/1000.0;
    let omega=380.0_f32.sqrt();let damping=0.8;
    let damped=omega*(1.0_f32-damping*damping).sqrt();
    1.0-(-damping*omega*t).exp()*((damped*t).cos()+damping*omega/damped*(damped*t).sin())
}
fn reveal(ms:u64)->f32 {smooth((ms as f32-180.0)/460.0)}
fn blob_pose(ms:u64,canvas:Canvas)->[f32;4] {
    let t=((ms as f32-DROP_MS as f32)/(LAND_MS-DROP_MS)as f32).clamp(0.0,1.0);
    // Acceleration carries the blob through the opening instead of braking it
    // in mid-air. Stretch follows the fall, with approximately constant area.
    let stretch=(1.0+0.65*t*t).sqrt();
    let settling=spatial(ms as f32-220.0);
    let w=mix(36.0,26.0,settling)/stretch;
    let h=mix(19.0,26.0,settling)*stretch;
    let anticipation=7.0*16.0*t*t*(1.0-t)*(1.0-t);
    let cy=mix(canvas.h/2.0,canvas.h-slot_ui::MOUTH_H+40.0,t*t)-anticipation;
    [canvas.centred(w),cy-h/2.0,w,h]
}

pub struct BootMotion { tex: TexId, blob: TexId, w: f32, h: f32 }
impl BootMotion {
    pub fn upload(c: &mut Compositor) -> Option<Self> {
        // One small mark texture shared by all seven glyphs, plus a 64px blob.
        // No frame atlas,
        // texture uploads, or offscreen pass during the animation.
        let logo=crate::bootlogo::decode(include_bytes!("../../../card/System/bootlogo.bmp"))?;
        let background=*logo.rgb.first()?;
        let rgba:Vec<u8>=logo.rgb.iter().flat_map(|rgb| [rgb[0],rgb[1],rgb[2],if *rgb==background {0}else{255}]).collect();
        let blob:Vec<u8>=(0..64).flat_map(|y|(0..64).flat_map(move |x| {
            let distance=(x as f32+0.5-32.0).hypot(y as f32+0.5-32.0);
            [INK[0],INK[1],INK[2],((32.5-distance).clamp(0.0,1.0)*255.0).round()as u8]
        })).collect();
        Some(Self{tex:c.create_texture(logo.width as u32,logo.height as u32,&rgba),blob:c.create_texture(64,64,&blob),w:logo.width as f32,h:logo.height as f32})
    }
    pub fn draw(&self, ms:u64, canvas:Canvas, out:&mut Vec<Draw>) {
        if ms>=BOOT_MS {return;}
        let ground=1.0-reveal(ms);
        if ground>0.0 {out.push(Draw::Rect{x:0.0,y:0.0,w:canvas.w,h:canvas.h,colour:[11.0/255.0,14.0/255.0,17.0/255.0,ground]});}
        let origin=[canvas.centred(self.w).floor(),((canvas.h-self.h)/2.0).floor()];
        if ms==0 {
            out.push(Draw::Tex{x:origin[0],y:origin[1],w:self.w,h:self.h,tex:self.tex,alpha:1.0});
            return;
        }
        let morph=smooth((ms as f32-220.0)/220.0);
        if morph<1.0 {
            for (index,edges) in LETTER_EDGES.windows(2).enumerate() {
                let centre=(edges[0]+edges[1])/2.0;
                // The middle gathers first; the outer letters follow within
                // 27 ms, retaining a single connected visual rhythm.
                let gather=spatial(ms as f32-40.0-(index as f32-3.0).abs()*9.0);
                let scale_x=mix(1.0,0.14,gather);let scale_y=mix(1.0,0.18,gather);
                let w=(edges[1]-edges[0])*scale_x;let h=self.h*scale_y;
                let cx=mix(origin[0]+centre,canvas.w/2.0,gather);
                let cy=mix(origin[1]+self.h/2.0,canvas.h/2.0,gather);
                out.push(Draw::Part{x:cx-w/2.0,y:cy-h/2.0,w,h,tex:self.tex,alpha:1.0-morph,uv:[edges[0]/self.w,0.0,(edges[1]-edges[0])/self.w,1.0]});
            }
        }
        if morph>0.0 {
            let [x,y,w,h]=blob_pose(ms,canvas);
            // Bay artwork includes translucent plastic. Explicitly occlude at
            // its top edge so the falling blob cannot glow through the shell.
            let visible=h.min(canvas.h-slot_ui::MOUTH_H-y).max(0.0);
            if visible>0.0 {out.push(Draw::Part{x,y,w,h:visible,tex:self.blob,alpha:morph,uv:[0.0,0.0,1.0,visible/h]});}
        }
    }
    pub fn compose(&self,ms:u64,canvas:Canvas,slot:Option<(TexId,TexId)>,out:&mut Vec<Draw>) {
        if ms>=BOOT_MS {return;}
        // Remove the scene's original bay before composing it at its final depth.
        // Drawing a second translucent copy would change the plastic's shading.
        if let Some((back,front))=slot {
            out.retain(|d|!matches!(d,Draw::Tex{tex,..}|Draw::Part{tex,..} if *tex==back||*tex==front));
        }
        self.draw(ms,canvas,out);
        // Reveal the bay with the scene, then let it occlude the descending blob.
        let reveal=reveal(ms);
        if reveal==0.0 {return;}
        let begin=out.len();slot_ui::draw_empty_slot(canvas,slot,out);
        for draw in &mut out[begin..] {match draw {
            Draw::Tex{alpha,..}|Draw::Part{alpha,..}=>*alpha*=reveal,
            Draw::Rect{colour,..}|Draw::RoundRect{colour,..}=>colour[3]*=reveal,
            _=>{}
        }}
    }
    pub fn release(self,c:&mut Compositor){for tex in [self.tex,self.blob] {c.update_texture(tex,1,1,&[0,0,0,0]);}}
}

#[cfg(test)]
mod tests {
    use super::*;
    fn motion()->BootMotion {
        let logo=crate::bootlogo::decode(include_bytes!("../../../card/System/bootlogo.bmp")).unwrap();
        BootMotion{tex:TexId::from_raw(1),blob:TexId::from_raw(2),w:logo.width as f32,h:logo.height as f32}
    }
    #[test]
    fn handoff_starts_at_firmware_size_and_leaves_no_overlay() {
        let motion=motion();
        for width in [640.0,720.0] {
            let canvas=Canvas{w:width,h:480.0};let mut out=vec![];
            motion.draw(0,canvas,&mut out);
            assert!(matches!(out[1],Draw::Tex{x,y,w,h,alpha:1.0,..} if w==motion.w&&h==motion.h&&x==((width-w)/2.0).floor()&&y==((480.0-h)/2.0).floor()));
            for ms in [BOOT_MS,u64::MAX] {out.clear();motion.draw(ms,canvas,&mut out);assert!(out.is_empty());}
        }
    }
    #[test]
    fn letters_collect_before_a_small_blob_accelerates_below_the_slot() {
        let motion=motion();let canvas=Canvas::FRAMED;
        let mut out=vec![];motion.draw(250,canvas,&mut out);
        assert_eq!(out.iter().filter(|d|matches!(d,Draw::Part{tex,..} if *tex==motion.tex)).count(),7);
        out.clear();motion.draw(DROP_MS,canvas,&mut out);
        assert!(!out.iter().any(|d|matches!(d,Draw::Part{tex,..} if *tex==motion.tex)));
        let [_,y,w,h]=blob_pose(DROP_MS,canvas);
        assert!(w<32.0&&h<32.0&&(y+h/2.0-canvas.h/2.0).abs()<0.01);
        let mut last=blob_pose(DROP_MS,canvas);
        for ms in DROP_MS+1..=LAND_MS {
            let pose=blob_pose(ms,canvas);
            assert!((pose[0]+pose[2]/2.0-canvas.w/2.0).abs()<0.01);
            assert!(pose[1]+pose[3]/2.0>=last[1]+last[3]/2.0-0.01);
            assert!((pose[1]-last[1]).abs()<1.1,"discontinuous drop at {ms}");
            assert!(pose[2]>15.0&&pose[2]<40.0&&pose[3]>15.0&&pose[3]<40.0);
            last=pose;
        }
        assert!(last[1]>canvas.h-slot_ui::MOUTH_H);
    }
    #[test]
    fn letter_crops_cover_the_original_mark_without_cutting_any_ink() {
        let logo=crate::bootlogo::decode(include_bytes!("../../../card/System/bootlogo.bmp")).unwrap();
        assert_eq!(logo.width as f32,*LETTER_EDGES.last().unwrap());
        let background=logo.rgb[0];
        for &edge in &LETTER_EDGES[1..7] {for y in 0..logo.height {
            assert_eq!(logo.rgb[y*logo.width+edge as usize],background,"letter crop crosses a stroke");
        }}
    }
    #[test]
    fn gathering_is_continuous_and_has_only_a_quiet_spring_overshoot() {
        assert_eq!(spatial(0.0),0.0);
        let mut last=0.0;let mut peak=0.0_f32;
        for ms in 1..600 {let next=spatial(ms as f32);assert!((next-last).abs()<0.01);peak=peak.max(next);last=next;}
        assert!(peak>1.0&&peak<1.02);
    }
}
