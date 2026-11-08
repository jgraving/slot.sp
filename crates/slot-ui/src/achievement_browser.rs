//! Three roomy achievement rows with full selected copy below the list.
use crate::{text,Canvas,UndoFace};
pub struct AchievementRow<'a>{pub title:&'a str,pub points:u32,pub earned:bool,pub badge:Option<(&'a [u8],u32,u32)>}
pub fn achievement_browser_face(canvas:Canvas,light:bool,title:&str,summary:&str,rows:&[AchievementRow<'_>],selected:Option<(&str,&str)>,empty:&str)->UndoFace {
    let mut face=browser_footer(canvas,false,!rows.is_empty());
    let heading=crate::quick_heading_face("Achievements");let x=(face.w-heading.w)/2;
    crate::plate::blit(&mut face.rgba,face.w,&heading.rgba,heading.w,heading.h,x,0);
    let ink=if light {slot_gfx::ui_palette::INK}else{[246,244,239]};
    let secondary=if light {[74,83,87]}else{[193,202,207]};
    let width=face.w-192;
    words(&mut face,title,96,54,width-182,1,16.0,15.0,21,secondary);
    let right=face.w-272;words(&mut face,summary,right,54,176,1,14.0,13.0,21,secondary);
    if rows.is_empty() {words(&mut face,empty,96,132,width,3,19.0,17.0,26,ink);return face;}
    for (i,row) in rows.iter().take(3).enumerate() {
        let y=84+i as u32*54;
        if let Some((rgba,sw,sh))=row.badge.filter(|(rgba,sw,sh)|*sw>0&&*sh>0&&rgba.len()==(*sw**sh*4)as usize) {
            let scale=(40.0/sw as f32).min(40.0/sh as f32).min(1.0);
            let w=(sw as f32*scale).round().max(1.0)as u32;let h=(sh as f32*scale).round().max(1.0)as u32;
            for py in 0..h {for px in 0..w {
                let pixel=crate::art::box_average(rgba,sw,sh,px as f32*sw as f32/w as f32,py as f32*sh as f32/h as f32,(px+1)as f32*sw as f32/w as f32,(py+1)as f32*sh as f32/h as f32);
                let at=(((y+(40-h)/2+py)*face.w+96+(40-w)/2+px)*4)as usize;face.rgba[at..at+4].copy_from_slice(&pixel);
            }}
        }else{
            let shape=if row.earned {r#"<path d="m5 12 4 4L19 6"/>"#}else{r#"<rect x="5" y="10" width="14" height="11" rx="3"/><path d="M8 10V7a4 4 0 0 1 8 0v3"/>"#};
            let svg=format!(r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><g fill="none" stroke="#{:02x}{:02x}{:02x}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">{shape}</g></svg>"##,secondary[0],secondary[1],secondary[2]);
            let icon=crate::art::render_svg(&svg,28,28).unwrap();crate::plate::blit(&mut face.rgba,face.w,&icon,28,28,102,y+6);
        }
        words(&mut face,row.title,148,y,width-116,1,19.0,17.0,25,ink);
        words(&mut face,if row.earned {"Earned"}else{"Locked"},148,y+24,width-116,1,14.0,14.0,18,secondary);
        let right=face.w-148;
        words(&mut face,&format!("{} pt",row.points),right,y+9,52,1,16.0,14.0,24,secondary);
    }
    if let Some((title,detail))=selected {
        words(&mut face,title,96,252,width,2,18.0,16.0,22,ink);
        words(&mut face,detail,96,300,width,3,16.0,15.0,19,secondary);
    }
    face
}
fn words(face:&mut UndoFace,value:&str,x:u32,y:u32,width:u32,count:usize,max:f32,min:f32,pitch:u32,ink:[u8;3]) {
    let layout=super::achievement_notice::fitted(value,text::Role::Detail,width,count,max,min);
    super::achievement_notice::lines(face,&layout,x,y,pitch,ink);
}

fn browser_footer(canvas:Canvas,detail:bool,has_items:bool)->UndoFace {
    use crate::{quick_menu::{button_face,button_pair,with_hint,LEGEND_Y},bitmaps::ButtonLegend as B};
    let mut face=crate::parent_footer_face_on(canvas,7,false);
    if !has_items {return face;}
    let ink=Some([246,244,239]);
    let mut hints=vec![with_hint(button_pair(B::Left,B::Right),if detail {"Previous / next"}else{"Filter"},ink,false)];
    if detail {hints.insert(0,with_hint(button_pair(B::Up,B::Down),"Scroll",ink,false));}
    if !detail {hints.push(with_hint(button_face(B::A),"Details",ink,false));}
    let mut x=face.w-16;
    for hint in hints.iter().rev() {x-=hint.w;crate::plate::blit(&mut face.rgba,face.w,&hint.rgba,hint.w,hint.h,x,LEGEND_Y as u32);x-=16;}
    face
}

pub fn achievement_description_lines(detail:&str,canvas:Canvas)->Vec<String> {
    let font=text::font(text::Role::Detail).unwrap();
    text::fit_preserving_case(font,detail,canvas.w-192.0,usize::MAX,16.0,16.0).lines
}
pub fn achievement_detail_face(canvas:Canvas,light:bool,game:&str,row:&AchievementRow<'_>,detail:&str,scroll:usize)->UndoFace {
    let mut face=browser_footer(canvas,true,true);let heading=crate::quick_heading_face("Achievement");
    crate::plate::blit(&mut face.rgba,face.w,&heading.rgba,heading.w,heading.h,(face.w-heading.w)/2,0);
    let ink=if light {slot_gfx::ui_palette::INK}else{[246,244,239]};
    let secondary=if light {[74,83,87]}else{[193,202,207]};let width=face.w-192;
    words(&mut face,game,96,54,width,1,16.0,15.0,21,secondary);
    words(&mut face,row.title,96,84,width,3,20.0,17.0,25,ink);
    words(&mut face,&format!("{} · {} points",if row.earned {"Earned"}else{"Locked"},row.points),96,164,width,1,16.0,16.0,22,secondary);
    let lines=achievement_description_lines(detail,canvas);let start=scroll.min(lines.len().saturating_sub(8));
    let layout=text::Layout{lines:lines.into_iter().skip(start).take(8).collect(),role:text::Role::Detail,px:16.0,tracking:0.0};
    super::achievement_notice::lines(&mut face,&layout,96,194,20,ink);
    face
}
