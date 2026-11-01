//! A short handoff from the firmware's exact mark into the cartridge opening.
use slot_gfx::{Canvas, Compositor, Draw, TexId};

pub const BOOT_MS: u64 = 900;
const COLLAPSE_MS: u64 = 320;
const LAND_MS: u64 = 760;
fn smooth(t: f32) -> f32 { let t=t.clamp(0.0,1.0); t*t*(3.0-2.0*t) }
fn mix(a: f32,b: f32,t: f32)->f32 { a+(b-a)*t }

pub struct BootMotion { tex: TexId, w: f32, h: f32 }
impl BootMotion {
    pub fn upload(c: &mut Compositor) -> Option<Self> {
        // Keep the firmware's native pixel size and centering. The 48-frame
        // glass atlas belongs to the website; boot uploads only this small mark.
        let logo=crate::bootlogo::decode(include_bytes!("../../../card/System/bootlogo.bmp"))?;
        let background=*logo.rgb.first()?;
        let rgba:Vec<u8>=logo.rgb.iter().flat_map(|rgb| [rgb[0],rgb[1],rgb[2],if *rgb==background {0}else{255}]).collect();
        Some(Self{tex:c.create_texture(logo.width as u32,logo.height as u32,&rgba),w:logo.width as f32,h:logo.height as f32})
    }
    pub fn draw(&self, ms:u64, canvas:Canvas, out:&mut Vec<Draw>) {
        if ms>=BOOT_MS {return;}
        let collapse=smooth(ms as f32/COLLAPSE_MS as f32);
        let merge=smooth((ms as f32-160.0)/160.0);
        let travel=smooth((ms.saturating_sub(COLLAPSE_MS))as f32/(LAND_MS-COLLAPSE_MS)as f32);
        let fade=1.0-smooth(ms.saturating_sub(LAND_MS)as f32/(BOOT_MS-LAND_MS)as f32);
        let ground=1.0-smooth((ms as f32-220.0)/430.0);
        if ground>0.0 {out.push(Draw::Rect{x:0.0,y:0.0,w:canvas.w,h:canvas.h,colour:[11.0/255.0,14.0/255.0,17.0/255.0,ground]});}
        if merge<1.0 {
            let w=mix(self.w,56.0,collapse);let h=mix(self.h,20.0,collapse);
            out.push(Draw::Tex{x:mix(canvas.centred(self.w).floor(),canvas.centred(56.0),collapse),y:mix(((canvas.h-self.h)/2.0).floor(),(canvas.h-20.0)/2.0,collapse),w,h,tex:self.tex,alpha:1.0-merge});
        }
        if merge>0.0 {
            let w=mix(mix(120.0,56.0,collapse),308.0,travel);
            let h=mix(20.0,9.0,travel);
            let cy=mix(canvas.h/2.0,canvas.h-slot_ui::MOUTH_H+36.0,travel);
            out.push(Draw::RoundRect{x:canvas.centred(w),y:cy-h/2.0,w,h,radius:h/2.0,colour:[246.0/255.0,247.0/255.0,242.0/255.0,merge*fade]});
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
        // Reveal the bay with the scene, then let it occlude the descending mark.
        let reveal=smooth((ms as f32-220.0)/430.0);
        if reveal==0.0 {return;}
        let begin=out.len();slot_ui::draw_empty_slot(canvas,slot,out);
        for draw in &mut out[begin..] {match draw {
            Draw::Tex{alpha,..}|Draw::Part{alpha,..}=>*alpha*=reveal,
            Draw::Rect{colour,..}|Draw::RoundRect{colour,..}=>colour[3]*=reveal,
            _=>{}
        }}
    }
    pub fn release(self,c:&mut Compositor){c.update_texture(self.tex,1,1,&[0,0,0,0]);}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn handoff_starts_at_firmware_size_stays_centered_and_leaves_no_overlay() {
        let logo=crate::bootlogo::decode(include_bytes!("../../../card/System/bootlogo.bmp")).unwrap();
        let motion=BootMotion{tex:TexId::from_raw(1),w:logo.width as f32,h:logo.height as f32};
        for width in [640.0,720.0] {
            let canvas=Canvas{w:width,h:480.0};let mut out=vec![];
            motion.draw(0,canvas,&mut out);
            assert!(matches!(out[1],Draw::Tex{x,y,w,h,alpha:1.0,..} if w==motion.w&&h==motion.h&&x==((width-w)/2.0).floor()&&y==((480.0-h)/2.0).floor()));
            let mut last_y=240.0;
            for ms in COLLAPSE_MS..=LAND_MS {
                out.clear();motion.draw(ms,canvas,&mut out);
                let Draw::RoundRect{x,y,w,h,colour,..}=*out.last().unwrap() else{panic!()};
                assert!((x+w/2.0-width/2.0).abs()<0.01);assert!(y+h/2.0>=last_y-0.01);last_y=y+h/2.0;
                assert!(w>0.0&&h>0.0&&colour[3]>0.0&&colour[3]<=1.0);
            }
            for ms in [BOOT_MS,u64::MAX] {out.clear();motion.draw(ms,canvas,&mut out);assert!(out.is_empty());}
        }
    }
}
