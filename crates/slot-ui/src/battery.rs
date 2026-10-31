//! Compact horizontal status battery, following Android's unified layout.
use slot_gfx::{Draw,TexId};
use slot_power::{Battery,Charge};
use crate::{Printed,UndoFace,text};
pub const BOLT_PX:f32=20.0;
pub const GAUGE_W:f32=70.0;
pub const GAUGE_H:f32=26.0;
pub const WALL:f32=0.0;
const BODY:f32=54.0;

pub fn status_battery_face(b:Battery)->UndoFace {
    let (w,h)=(GAUGE_W as u32,GAUGE_H as u32);
    let mut face=UndoFace{rgba:vec![0;(w*h*4)as usize],w,h};
    let charging=matches!(b.charge,Charge::Charging|Charge::Full);
    let (track,fill,dark)=if charging {([0.20,0.36,0.28,1.0],[0.67,0.94,0.78,1.0],[18,47,32])}
        else if b.percent<=20 {([0.38,0.14,0.15,1.0],[1.0,0.70,0.68,1.0],[65,6,9])}
        else {([0.36,0.39,0.42,1.0],[0.94,0.96,0.98,1.0],[17,24,29])};
    crate::expressive::raster_round(&mut face,0.0,0.0,BODY,GAUGE_H,6.0,track);
    let edge=BODY*b.percent.min(100)as f32/100.0;
    // Intersect the level with the capsule's existing mask; no square fill corners.
    for y in 0..h {for x in 0..edge.ceil()as u32 {let i=((y*w+x)*4)as usize;for c in 0..3 {face.rgba[i+c]=(fill[c]*255.0)as u8;}}}
    crate::expressive::raster_round(&mut face,BODY,9.0,2.0,8.0,1.0,fill);
    let label=b.percent.min(100).to_string();
    let layout=text::Layout{lines:vec![label],px:18.0,tracking:0.0,role:text::Role::Focus};
    let mut ink=vec![0;(BODY as u32*h*4)as usize];
    text::draw_centred(&mut ink,BODY as u32,h,&layout,[255,255,255]);
    for y in 0..h {for x in 0..BODY as u32 {let i=((y*w+x)*4)as usize;let a=ink[((y*BODY as u32+x)*4+3)as usize]as f32/255.0;let colour=if (x as f32)<edge {dark}else{[245,248,250]};for c in 0..3{face.rgba[i+c]=(colour[c]as f32*a+face.rgba[i+c]as f32*(1.0-a)).round()as u8;}}}
    if charging {
        let svg=r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path fill="#b0f2cd" d="M13 1 4 14h7l-1 9L20 9h-7z"/></svg>"##;
        let bolt=crate::art::render_svg(svg,12,20).expect("battery bolt");
        crate::plate::blit(&mut face.rgba,w,&bolt,12,20,58,3);
    }
    face
}
pub fn draw_gauge(x:f32,y:f32,battery:Option<Battery>,percent:Printed,bolt:Option<TexId>,ink:[u8;3],out:&mut Vec<Draw>){draw_gauge_sized(x,y,battery,percent,bolt,ink,crate::HINT_H,out);}
pub(crate) fn draw_gauge_sized(x:f32,y:f32,battery:Option<Battery>,percent:Printed,_bolt:Option<TexId>,_ink:[u8;3],_text_h:u32,out:&mut Vec<Draw>){
    if battery.is_none(){return;}
    if let Some(tex)=percent.face {out.push(Draw::Tex{x,y,w:GAUGE_W,h:GAUGE_H,tex,alpha:1.0});}
    else {out.push(Draw::RoundRect{x,y,w:GAUGE_W,h:GAUGE_H,radius:6.0,colour:[0.9,0.94,0.96,1.0]});}
}
pub(crate) fn gauge_width(_p:Printed)->f32{GAUGE_W}
