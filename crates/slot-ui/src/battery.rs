//! Compact horizontal status battery, following Android's unified layout.
use slot_gfx::{Draw,TexId};
use slot_power::{Battery,Charge};
use crate::{Printed,UndoFace,text};
pub const BOLT_PX:f32=20.0;
pub const GAUGE_W:f32=70.0;
pub const GAUGE_H:f32=26.0;
pub const WALL:f32=0.0;
const BODY:f32=GAUGE_W;

pub(crate) struct ChargeColours {
    pub track:[f32;4],pub fill:[f32;4],pub filled_ink:[u8;3],pub empty_ink:[u8;3],
}
pub(crate) fn charge_colours(light:bool)->ChargeColours {
    if light {ChargeColours{track:[0.78,0.88,0.82,1.0],fill:[0.07,0.40,0.27,1.0],filled_ink:[248,252,249],empty_ink:[22,42,31]}}
    else {ChargeColours{track:[0.20,0.36,0.28,1.0],fill:[0.67,0.94,0.78,1.0],filled_ink:[18,47,32],empty_ink:[245,248,250]}}
}
pub fn status_battery_face(b:Battery)->UndoFace {status_battery_face_on(b,false)}
/// Battery artwork owns both ink colours so a theme switch cannot invert its
/// fill without also adapting the percentage crossing the level boundary.
pub fn status_battery_face_on(b:Battery,light:bool)->UndoFace {
    let (w,h)=(GAUGE_W as u32,GAUGE_H as u32);
    let mut face=UndoFace{rgba:vec![0;(w*h*4)as usize],w,h};
    let charging=matches!(b.charge,Charge::Charging|Charge::Full);
    let palette=charge_colours(light);
    let (track,fill,filled_ink,empty_ink)=if charging {(palette.track,palette.fill,palette.filled_ink,palette.empty_ink)}
        else if b.percent<=20 {
            if light {([0.95,0.82,0.82,1.0],[0.66,0.14,0.17,1.0],[255,249,249],[65,6,9])}
            else {([0.38,0.14,0.15,1.0],[1.0,0.70,0.68,1.0],[65,6,9],[245,248,250])}
        } else if light {([0.79,0.83,0.85,1.0],[0.25,0.34,0.40,1.0],[248,251,253],[17,24,29])}
        else {([0.36,0.39,0.42,1.0],[0.94,0.96,0.98,1.0],[17,24,29],[245,248,250])};
    crate::expressive::raster_round(&mut face,0.0,0.0,BODY,GAUGE_H,GAUGE_H/2.0,track);
    let edge=BODY*b.percent.min(100)as f32/100.0;
    // Intersect the level with the capsule's existing mask; no square fill corners.
    for y in 0..h {for x in 0..edge.ceil()as u32 {let i=((y*w+x)*4)as usize;for c in 0..3 {face.rgba[i+c]=(fill[c]*255.0)as u8;}}}
    if light {
        // A narrow native outline keeps the empty capsule visible on a pale backdrop.
        let radius=GAUGE_H/2.0;
        for y in 0..h {for x in 0..BODY as u32 {
            let dx=(radius-(x as f32+0.5)).max(x as f32+0.5-(BODY-radius)).max(0.0);
            let dy=y as f32+0.5-radius;
            let inset=radius-(dx*dx+dy*dy).sqrt();
            let border=(1.5-inset).clamp(0.0,1.0);let i=((y*w+x)*4)as usize;
            for c in 0..3 {face.rgba[i+c]=(slot_gfx::ui_palette::DIM[c]as f32*border+face.rgba[i+c]as f32*(1.0-border)).round()as u8;}
        }}
    }
    // The bolt and number share both the capsule and its two contrast inks.
    let ink=content_mask(b.percent,charging);
    for y in 0..h {for x in 0..w {let i=((y*w+x)*4)as usize;let a=ink[i+3]as f32/255.0;let colour=if (x as f32)<edge {filled_ink}else{empty_ink};for c in 0..3{face.rgba[i+c]=(colour[c]as f32*a+face.rgba[i+c]as f32*(1.0-a)).round()as u8;}}}
    face
}

fn content_mask(percent:u8,charging:bool)->Vec<u8> {
    let label=percent.min(100).to_string();
    let font=text::font(text::Role::Focus).expect("bundled battery font");
    let tw=text::line_width(font,&label,18.0,0.0).ceil()as u32+2;
    let (w,h)=(GAUGE_W as u32,GAUGE_H as u32);
    let prefix=if charging {14}else{0};
    let left=(w-tw-prefix)/2;
    let layout=text::Layout{lines:vec![label],px:18.0,tracking:0.0,role:text::Role::Focus};
    let mut text=vec![0;(tw*h*4)as usize];
    text::draw_centred(&mut text,tw,h,&layout,[255,255,255]);
    let mut mask=vec![0;(w*h*4)as usize];
    crate::plate::blit(&mut mask,w,&text,tw,h,left+prefix,0);
    if charging {
        let svg=r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path fill="#ffffff" d="M13 1 4 14h7l-1 9L20 9h-7z"/></svg>"##;
        let bolt=crate::art::render_svg(svg,10,18).expect("battery bolt");
        crate::plate::blit(&mut mask,w,&bolt,10,18,left,4);
    }
    mask
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn charging_content_fits_inside_capsule_for_every_digit_count() {
        for percent in 0..=100 {
            let mask=content_mask(percent,true);let w=GAUGE_W as u32;
            let bounds:Vec<_>=mask.chunks_exact(4).enumerate().filter(|(_,p)|p[3]>0).map(|(i,_)|((i as u32)%w,(i as u32)/w)).collect();
            let x0=bounds.iter().map(|p|p.0).min().unwrap();let x1=bounds.iter().map(|p|p.0).max().unwrap();
            assert!(x0>=8&&x1<w-8,"{percent}: charging content meets rounded ends");
            assert!(x0.abs_diff(w-1-x1)<=3,"{percent}: content is not centered");
            assert!(bounds.iter().all(|&(_,y)|y>=4&&y<GAUGE_H as u32-3));
            let occupied=|x:u32|(0..GAUGE_H as u32).any(|y|mask[((y*w+x)*4+3)as usize]>0);
            assert!((x0+2..x1-2).any(|x|!occupied(x)&&!occupied(x+1)),"{percent}: no gap between bolt and digits");
        }
    }
}
pub fn draw_gauge(x:f32,y:f32,battery:Option<Battery>,percent:Printed,bolt:Option<TexId>,ink:[u8;3],out:&mut Vec<Draw>){draw_gauge_sized(x,y,battery,percent,bolt,ink,crate::HINT_H,out);}
pub(crate) fn draw_gauge_sized(x:f32,y:f32,battery:Option<Battery>,percent:Printed,_bolt:Option<TexId>,_ink:[u8;3],_text_h:u32,out:&mut Vec<Draw>){
    if battery.is_none(){return;}
    if let Some(tex)=percent.face {out.push(Draw::Tex{x,y,w:GAUGE_W,h:GAUGE_H,tex,alpha:1.0});}
    else {out.push(Draw::RoundRect{x,y,w:GAUGE_W,h:GAUGE_H,radius:6.0,colour:[0.9,0.94,0.96,1.0]});}
}
pub(crate) fn gauge_width(_p:Printed)->f32{GAUGE_W}
