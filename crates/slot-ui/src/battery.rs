use slot_gfx::{Draw, TexId};
use slot_power::{Battery, Charge};

use crate::footer::Printed;
use crate::plate::HINT_H;

pub const BOLT_PX: f32 = 24.0;
pub const GAUGE_W: f32 = 82.0;
pub const GAUGE_H: f32 = 30.0;
pub const WALL: f32 = 2.0;
const NUB_W: f32 = 3.0;
const BOLT_W: f32 = 18.0;

pub fn draw_gauge(x:f32,y:f32,battery:Option<Battery>,percent:Printed,bolt:Option<TexId>,ink:[u8;3],out:&mut Vec<Draw>) {
    draw_gauge_sized(x,y,battery,percent,bolt,ink,HINT_H,out);
}

pub(crate) fn draw_gauge_sized(x:f32,y:f32,battery:Option<Battery>,percent:Printed,bolt:Option<TexId>,ink:[u8;3],text_h:u32,out:&mut Vec<Draw>) {
    let Some(b)=battery else {return;};
    let charging=b.charge==Charge::Charging;
    let colour=if matches!(b.charge,Charge::Charging|Charge::Full) {[0.38,0.86,0.62,1.0]}
        else if b.percent<=20 {[1.0,0.31,0.34,1.0]}
        else {[ink[0] as f32/255.0,ink[1] as f32/255.0,ink[2] as f32/255.0,1.0]};
    out.push(Draw::RoundRect{x,y,w:GAUGE_W,h:GAUGE_H,radius:5.0,colour});
    out.push(Draw::RoundRect{x:x+GAUGE_W,y:y+GAUGE_H/2.0-5.0,w:NUB_W,h:10.0,radius:1.5,colour});
    out.push(Draw::RoundRect{x:x+WALL,y:y+WALL,w:GAUGE_W-2.0*WALL,h:GAUGE_H-2.0*WALL,radius:3.0,colour:[0.08,0.09,0.11,1.0]});
    let fill=(GAUGE_W-4.0*WALL)*b.percent.min(100) as f32/100.0;
    if fill>0.0 {
        out.push(Draw::RoundRect{x:x+2.0*WALL,y:y+2.0*WALL,w:fill,h:GAUGE_H-4.0*WALL,radius:1.5,colour:[colour[0]*0.32,colour[1]*0.32,colour[2]*0.32,1.0]});
    }
    let has_bolt=charging&&bolt.is_some();
    if let Some(tex)=bolt.filter(|_|charging) {
        out.push(Draw::Tex{x:x+4.0,y:y+(GAUGE_H-BOLT_W)/2.0,w:BOLT_W,h:BOLT_W,tex,alpha:1.0});
    }
    if let Some(tex)=percent.face {
        // The native status face is reduced uniformly to fit inside the body.
        let available=GAUGE_W-12.0-if has_bolt {BOLT_W+2.0}else{0.0};
        let scale=(22.0/text_h as f32).min(available/percent.w.max(1) as f32);
        let w=percent.w as f32*scale;
        let h=text_h as f32*scale;
        let left=x+if has_bolt {24.0}else{6.0};
        out.push(Draw::Tex{x:left+(available-w)/2.0,y:y+(GAUGE_H-h)/2.0,w,h,tex,alpha:1.0});
    }
}

/// Charge states share one fixed footprint, so neighbouring icons never jump.
pub(crate) fn gauge_width(_percent:Printed)->f32 {GAUGE_W+NUB_W}
