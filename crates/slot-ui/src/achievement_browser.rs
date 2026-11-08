//! Expanded, cached achievement sheets. Shared geometry keeps focus and text aligned.
use crate::{text, Canvas, UndoFace, Draw, TexId, MenuFrame};
pub const ACHIEVEMENT_VISIBLE:usize=5;
pub const ACHIEVEMENT_DETAIL_LINES:usize=9;
const EDGE:u32=40;
const ROW_TOP:u32=132;
const PITCH:u32=54;
const BOTTOM:f32=420.0;
const FOOTER:u32=432;
pub struct AchievementRow<'a>{pub title:&'a str,pub points:u32,pub earned:bool,pub badge:Option<(&'a [u8],u32,u32)>}

pub fn achievement_browser_face(canvas:Canvas,light:bool,title:&str,summary:&str,rows:&[AchievementRow<'_>],empty:&str)->UndoFace {
    let mut face=browser_footer(canvas,false,!rows.is_empty());
    heading(&mut face,"Achievements");
    let (ink,secondary)=inks(light);let width=face.w-2*EDGE;
    words(&mut face,title,EDGE,74,width,1,18.0,17.0,24,ink);
    words(&mut face,summary,EDGE,102,width,1,14.0,14.0,20,secondary);
    if rows.is_empty() {words(&mut face,empty,EDGE,164,width,4,19.0,17.0,26,ink);return face;}
    for (i,row) in rows.iter().take(ACHIEVEMENT_VISIBLE).enumerate() {
        let y=ROW_TOP+i as u32*PITCH;
        badge(&mut face,row,EDGE,y+3,40,secondary);
        words(&mut face,row.title,92,y,width-124,1,19.0,17.0,25,ink);
        words(&mut face,if row.earned {"Earned"}else{"Locked"},92,y+25,width-124,1,14.0,14.0,18,secondary);
        let right=face.w-104;
        words(&mut face,&format!("{} pt",row.points),right,y+10,64,1,16.0,14.0,24,secondary);
    }
    face
}
fn inks(light:bool)->([u8;3],[u8;3]) {if light {(slot_gfx::ui_palette::INK,[74,83,87])}else{([246,244,239],[193,202,207])}}
fn heading(face:&mut UndoFace,value:&str) {
    let title=crate::quick_heading_face(value);let x=(face.w-title.w)/2;
    crate::plate::blit(&mut face.rgba,face.w,&title.rgba,title.w,title.h,x,0);
}
fn words(face:&mut UndoFace,value:&str,x:u32,y:u32,width:u32,count:usize,max:f32,min:f32,pitch:u32,ink:[u8;3]) {
    let layout=super::achievement_notice::fitted(value,text::Role::Detail,width,count,max,min);
    super::achievement_notice::lines(face,&layout,x,y,pitch,ink);
}
/// Contain the complete image, downfiltering when needed and never enlarging a small badge.
fn badge(face:&mut UndoFace,row:&AchievementRow<'_>,x:u32,y:u32,size:u32,ink:[u8;3]) {
    if let Some((rgba,sw,sh))=row.badge.filter(|(rgba,sw,sh)|*sw>0&&*sh>0&&rgba.len()==(*sw as usize * *sh as usize *4)) {
        let scale=(size as f32/sw as f32).min(size as f32/sh as f32).min(1.0);
        let w=(sw as f32*scale).round().max(1.0)as u32;let h=(sh as f32*scale).round().max(1.0)as u32;
        for py in 0..h {for px in 0..w {
            let pixel=crate::art::box_average(rgba,sw,sh,px as f32*sw as f32/w as f32,py as f32*sh as f32/h as f32,(px+1)as f32*sw as f32/w as f32,(py+1)as f32*sh as f32/h as f32);
            let at=(((y+(size-h)/2+py)*face.w+x+(size-w)/2+px)*4)as usize;
            face.rgba[at..at+4].copy_from_slice(&pixel);
        }}
    }else{
        let shape=if row.earned {r#"<path d="m5 12 4 4L19 6"/>"#}else{r#"<rect x="5" y="10" width="14" height="11" rx="3"/><path d="M8 10V7a4 4 0 0 1 8 0v3"/>"#};
        let svg=format!(r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><g fill="none" stroke="#{:02x}{:02x}{:02x}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">{shape}</g></svg>"##,ink[0],ink[1],ink[2]);
        let icon=crate::art::render_svg(&svg,28,28).unwrap();crate::plate::blit(&mut face.rgba,face.w,&icon,28,28,x+(size-28)/2,y+(size-28)/2);
    }
}
fn browser_footer(canvas:Canvas,detail:bool,has_items:bool)->UndoFace {
    use crate::{quick_menu::{button_face,button_pair,with_hint},bitmaps::ButtonLegend as B};
    let mut face=UndoFace{w:canvas.w as u32,h:canvas.h as u32,rgba:vec![0;(canvas.w*canvas.h*4.0)as usize]};
    let ink=Some([246,244,239]);let back=with_hint(button_face(B::B),"Back",ink,true);
    crate::plate::blit(&mut face.rgba,face.w,&back.rgba,back.w,back.h,16,FOOTER);
    if !has_items {return face;}
    let mut hints=vec![with_hint(button_pair(B::Left,B::Right),if detail {"Previous / next"}else{"Filter"},ink,false)];
    if detail {hints.insert(0,with_hint(button_pair(B::Up,B::Down),"Scroll",ink,false));}
    if !detail {hints.push(with_hint(button_face(B::A),"Details",ink,false));}
    let mut x=face.w-16;
    for hint in hints.iter().rev() {x-=hint.w;crate::plate::blit(&mut face.rgba,face.w,&hint.rgba,hint.w,hint.h,x,FOOTER);x-=16;}
    face
}
pub fn achievement_description_lines(detail:&str,canvas:Canvas)->Vec<String> {
    let font=text::font(text::Role::Detail).unwrap();
    text::fit_preserving_case(font,detail,canvas.w-2.0*EDGE as f32,usize::MAX,17.0,17.0).lines
}
pub fn achievement_detail_face(canvas:Canvas,light:bool,game:&str,row:&AchievementRow<'_>,detail:&str,scroll:usize)->UndoFace {
    let mut face=browser_footer(canvas,true,true);heading(&mut face,"Achievement");
    let (ink,secondary)=inks(light);let width=face.w-2*EDGE;
    words(&mut face,game,EDGE,74,width,1,18.0,17.0,24,secondary);
    badge(&mut face,row,EDGE,112,64,secondary);
    words(&mut face,row.title,120,108,width-80,3,20.0,17.0,25,ink);
    words(&mut face,&format!("{} · {} points",if row.earned {"Earned"}else{"Locked"},row.points),EDGE,194,width,1,16.0,16.0,22,secondary);
    let lines=achievement_description_lines(detail,canvas);let start=scroll.min(lines.len().saturating_sub(ACHIEVEMENT_DETAIL_LINES));
    let layout=text::Layout{lines:lines.into_iter().skip(start).take(ACHIEVEMENT_DETAIL_LINES).collect(),role:text::Role::Detail,px:17.0,tracking:0.0};
    super::achievement_notice::lines(&mut face,&layout,EDGE,228,20,ink);
    face
}
/// The browser owns the extra space normally occupied by the decorative cartridge bay.
/// Its body slides with the existing page spring; status, heading and controls stay anchored.
pub fn draw_achievement_sheet(canvas:Canvas,tex:TexId,frame:MenuFrame,scroll:Option<(usize,usize,usize,usize)>,out:&mut Vec<Draw>) {
    use crate::expressive::{rounded,SURFACE,SELECTED};
    if !frame.outgoing {rounded(out,16.0,56.0,canvas.w-32.0,BOTTOM-56.0,24.0,SURFACE);}
    let begin=out.len();let dx=frame.page_shift*(canvas.w-32.0);
    if !frame.suppress_focus {if let Some((_,total,_,visible))=scroll.filter(|s|s.1>0&&s.3>0) {
        let offset=(frame.focus_y-crate::expressive::ROW_TOP)/crate::QUICK_PITCH;
        rounded(out,28.0+dx,ROW_TOP as f32-2.0+offset.clamp(0.0,(total.min(visible)-1)as f32)*PITCH as f32,canvas.w-56.0,50.0,12.0,SELECTED);
    }}
    out.push(Draw::Part{x:dx,y:56.0,w:canvas.w,h:BOTTOM-56.0,tex,alpha:frame.opacity,uv:[0.0,56.0/canvas.h,1.0,(BOTTOM-56.0)/canvas.h]});
    crate::quick_menu::clip_quads(out,begin,[16.0,56.0,canvas.w-16.0,BOTTOM]);
    if let Some((_,total,start,visible))=scroll.filter(|s|s.1>s.3&&s.3>0) {
        let h=PITCH as f32*visible as f32-6.0;let thumb=(h*visible as f32/total as f32).max(24.0);
        let y=ROW_TOP as f32+(h-thumb)*(start as f32/(total-visible)as f32).clamp(0.0,1.0);
        rounded(out,canvas.w-24.0,ROW_TOP as f32,3.0,h,1.5,[0.55,0.64,0.65,0.22]);
        rounded(out,canvas.w-24.0,y,3.0,thumb,1.5,[0.76,0.84,0.83,1.0]);
    }
    if !frame.outgoing {
        out.push(Draw::Part{x:0.0,y:0.0,w:canvas.w,h:56.0,tex,alpha:1.0,uv:[0.0,0.0,1.0,56.0/canvas.h]});
        out.push(Draw::Part{x:0.0,y:FOOTER as f32,w:canvas.w,h:40.0,tex,alpha:1.0,uv:[0.0,FOOTER as f32/canvas.h,1.0,40.0/canvas.h]});
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detail_badges_keep_all_edges_and_never_enlarge_at_both_widths() {
        for width in [640.0,720.0] {for (sw,sh) in [(64,64),(128,64),(64,128),(16,16)] {
            let rgba=[220,80,40,255].repeat((sw*sh)as usize);
            let row=AchievementRow{title:"A long achievement title that wraps alongside its complete badge",points:25,earned:true,badge:Some((&rgba,sw,sh))};
            let face=achievement_detail_face(Canvas{w:width,h:480.0},false,"A long game name",&row,"Full description",0);
            let mut xs=vec![];let mut ys=vec![];
            for y in 100..190 {for x in 0..120 {if face.rgba[((y*face.w+x)*4)as usize..][..4]==[220,80,40,255]{xs.push(x);ys.push(y);}}}
            let w=xs.iter().max().unwrap()-xs.iter().min().unwrap()+1;
            let h=ys.iter().max().unwrap()-ys.iter().min().unwrap()+1;
            assert!(w<=64&&h<=64&&w<=sw&&h<=sh);assert_eq!(w*sh,h*sw);
        }}
    }
    #[test]
    fn expanded_list_pads_the_game_heading_and_uses_five_complete_rows() {
        for width in [640.0,720.0] {for light in [false,true] {
            let rows:Vec<_>=(0..5).map(|_|AchievementRow{title:"Catch Rayquaza",points:10,earned:false,badge:None}).collect();
            let face=achievement_browser_face(Canvas{w:width,h:480.0},light,"Pokémon Pinball: Ruby & Sapphire","4/55 earned · All",&rows,"");
            let ink=|y:u32|(EDGE..face.w-EDGE).any(|x|face.rgba[((y*face.w+x)*4+3)as usize]>0);
            assert!((56..74).all(|y|!ink(y)),"game title needs top padding");
            assert!((74..100).any(ink));assert!((348..398).any(ink),"fifth row must render");
            assert!((420..432).all(|y|!ink(y)),"body and controls need a gap");
        }}
    }
}
