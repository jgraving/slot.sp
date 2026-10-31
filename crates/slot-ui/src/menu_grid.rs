//! Cached vector tiles and spatial navigation for the carousel's settings home.
use crate::{Canvas,Draw,QuickRow,UndoFace,text};
use crate::expressive::{MENU_INSET,rounded};
pub const COLUMNS:usize=4;
pub const TOP:f32=80.0;
pub const PITCH:f32=142.0;
pub const TILE_H:f32=120.0;

pub fn is_page(page:&[QuickRow])->bool {page==QuickRow::MAIN || page==QuickRow::SETTINGS || page==QuickRow::PLAY}
pub fn columns(row:QuickRow,hidden:&[QuickRow])->usize {if row.page()==QuickRow::MAIN {4}else{row.shown(hidden).len().min(3).max(1)}}
pub fn position(row:QuickRow,hidden:&[QuickRow])->[f32;2]{let n=columns(row,hidden);let i=row.position(hidden);[(i%n)as f32,(i/n)as f32]}
pub fn rect(canvas:Canvas,index:usize)->[f32;4] {cell(canvas,index,8,4,false)}
pub fn cell(canvas:Canvas,index:usize,count:usize,cols:usize,pause:bool)->[f32;4] {
    let rows=count.div_ceil(cols);
    let pitch=if pause {120.0}else{(canvas.w-2.0*MENU_INSET-24.0)/cols as f32};
    let in_row=(count-index/cols*cols).min(cols);
    let x=(canvas.w-in_row as f32*pitch)/2.0+(index%cols)as f32*pitch+4.0;
    let settings=!pause && cols==3;
    let top=if pause {(canvas.h-(rows as f32*124.0-12.0))/2.0}else if settings {104.0}else{TOP};
    [x,top+(index/cols)as f32*if pause{124.0}else if settings{112.0}else{PITCH},pitch-8.0,if pause{112.0}else if settings{100.0}else{TILE_H}]
}
pub fn pause_rect(canvas:Canvas,count:usize)->[f32;4] {
    let cols=count.min(3).max(1);let rows=count.div_ceil(cols);
    let w=cols as f32*120.0+32.0;let h=rows as f32*124.0+20.0;
    [canvas.centred(w),(canvas.h-h)/2.0,w,h]
}

pub fn navigate(row:QuickRow,hidden:&[QuickRow],dx:i32,dy:i32)->QuickRow {
    let rows=row.shown(hidden);
    let columns=columns(row,hidden);
    let i=row.position(hidden);
    let next=if dx!=0 {
        let start=i/columns*columns;
        let n=(rows.len()-start).min(columns);
        start+((i-start) as i32+dx).rem_euclid(n as i32) as usize
    }else {
        let col=i%columns;
        let count=(rows.len()+columns-1-col)/columns;
        ((i/columns) as i32+dy).rem_euclid(count as i32) as usize*columns+col
    };
    rows[next]
}

pub fn tile_face(row:QuickRow,lit:bool)->UndoFace {
    // Filled silhouettes with transparent inset details stay legible on the handheld.
    let (label,paths)=match row {
        QuickRow::General|QuickRow::Settings=>("General",r##"<path fill-rule="evenodd" d="M40.34 19.31 L44.46 19.28 L44.46 28.72 L40.34 28.69 L38.87 32.24 L41.81 35.13 L35.13 41.81 L32.24 38.87 L28.69 40.34 L28.72 44.46 L19.28 44.46 L19.31 40.34 L15.76 38.87 L12.87 41.81 L6.19 35.13 L9.13 32.24 L7.66 28.69 L3.54 28.72 L3.54 19.28 L7.66 19.31 L9.13 15.76 L6.19 12.87 L12.87 6.19 L15.76 9.13 L19.31 7.66 L19.28 3.54 L28.72 3.54 L28.69 7.66 L32.24 9.13 L35.13 6.19 L41.81 12.87 L38.87 15.76z M31 24a7 7 0 1 0-14 0 7 7 0 0 0 14 0"/>"##),
        QuickRow::Screen|QuickRow::PlayScreen=>("Screen",r##"<path fill-rule="evenodd" d="M7 7h34a4 4 0 0 1 4 4v23a4 4 0 0 1-4 4H7a4 4 0 0 1-4-4V11a4 4 0 0 1 4-4z M8 12v21h32V12z"/><path d="M20 37h8v4h8v3H12v-3h8z"/>"##),
        QuickRow::Bluetooth|QuickRow::PlaySound=>("Sound",r##"<path d="M5 18h9l12-10v32L14 30H5zM31 16v16a11 11 0 0 0 0-16M36 8v4a15 15 0 0 1 0 24v4a19 19 0 0 0 0-32"/>"##),
        QuickRow::Game|QuickRow::PlayGameplay=>("Gameplay",r##"<path fill-rule="evenodd" d="M14 12h20q7 0 9 9l3 14q1 7-5 7-2 0-7-6H14q-5 6-7 6-6 0-5-7l3-14q2-9 9-9z M12 19v4H8v4h4v4h4v-4h4v-4h-4v-4z M31 20a3 3 0 1 0 0 6 3 3 0 0 0 0-6z M38 27a3 3 0 1 0 0 6 3 3 0 0 0 0-6z"/>"##),
        QuickRow::Power=>("Power & lid",r##"<path d="M22 3h4v20h-4zM16 9v5a14 14 0 1 0 16 0V9a19 19 0 1 1-16 0"/>"##),
        QuickRow::Controls|QuickRow::PlayControls=>("Device\ncontrols",r##"<path fill-rule="evenodd" d="M17 4h14v13h13v14H31v13H17V31H4V17h13z M21 21v6h6v-6z"/>"##),
        QuickRow::Kids=>("Parental\ncontrols",r##"<path fill-rule="evenodd" d="M24 4a20 20 0 1 0 0 40 20 20 0 0 0 0-40z M17 15a2.5 2.5 0 1 0 0 5 2.5 2.5 0 0 0 0-5z M31 15a2.5 2.5 0 1 0 0 5 2.5 2.5 0 0 0 0-5z M13 26a11 11 0 0 0 22 0h-4a7 7 0 0 1-14 0z"/>"##),
        QuickRow::Resume=>("Resume",r##"<path d="M15 6q-3-2-3 2v32q0 4 3 2l27-16q3-2 0-4z"/>"##),
        QuickRow::ExitGame=>("Exit game",r##"<path d="M7 5h18v5H12v28h13v5H7zM30 13l13 11-13 11v-8H18v-6h12z"/>"##),
        QuickRow::States|QuickRow::UndoSave|QuickRow::UndoLoad=>("Save states",r##"<path fill-rule="evenodd" d="M8 3h27l8 8v32H5V6z M13 7v13h19V7z M13 29v10h22V29z"/>"##),
        QuickRow::Screenshots=>("Screenshots",r##"<path fill-rule="evenodd" d="M6 12h8l4-6h12l4 6h8a4 4 0 0 1 4 4v24H2V16a4 4 0 0 1 4-4z M33 26a9 9 0 1 0-18 0 9 9 0 0 0 18 0"/>"##),
        QuickRow::PlayReference=>("Shortcuts",r##"<path d="M7 7h34v34H7z"/><path fill="#192125" d="M13 15h6v6h-6zM23 15h6v6h-6zM33 15h3v6h-3zM13 27h23v5H13z"/>"##),
        _=>("About",r##"<path fill-rule="evenodd" d="M24 4a20 20 0 1 0 0 40 20 20 0 0 0 0-40z M22 21v14h4V21z M24 12a2.5 2.5 0 1 0 0 5 2.5 2.5 0 0 0 0-5z"/>"##),
    };
    let label=match row {QuickRow::Settings=>"Settings",QuickRow::UndoSave=>"Undo save",QuickRow::UndoLoad=>"Undo load",_=>label};
    let ink=if lit {"#effbf8"}else{"#dce4e7"};
    let svg=format!(r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 48 48"><g fill="{ink}">{paths}</g></svg>"##);
    let icon=crate::art::render_svg(&svg,44,44).expect("menu tile SVG");
    let (w,h)=(112,100);
    let mut rgba=vec![0;(w*h*4) as usize];
    for y in 0..44usize {let dst=((y+4)*w as usize+34)*4;rgba[dst..dst+44*4].copy_from_slice(&icon[y*44*4..(y+1)*44*4]);}
    let layout=text::Layout{lines:label.lines().map(str::to_owned).collect(),px:18.0,tracking:0.0,role:if lit{text::Role::Focus}else{text::Role::Menu}};
    let mut words=vec![0;(w*48*4) as usize];
    text::draw_centred(&mut words,w,48,&layout,if lit{[246,249,248]}else{[211,219,222]});
    rgba[(52*w*4) as usize..].copy_from_slice(&words);
    if row.page()==QuickRow::PLAY {let offset=(w*4*3)as usize;let len=rgba.len();rgba.copy_within(..len-offset,offset);rgba[..offset].fill(0);}
    UndoFace{rgba,w,h}
}

pub(crate) fn draw(menu:&crate::quick_menu::QuickMenu<'_>,frame:crate::MenuFrame,out:&mut Vec<Draw>) {
    let Some(faces)=menu.faces else{return;};
    let canvas=menu.canvas;let rows=menu.row.shown(menu.hidden);let cols=columns(menu.row,menu.hidden);
    let pause=menu.row.page()==QuickRow::PLAY;
    if frame.background && !menu.over_game {crate::expressive::draw_card(canvas,out);}
    if !frame.outgoing {if let Some((_,(tex,w,h)))=faces.headings.iter().find(|(r,_)|*r==menu.row.page()[0]) {
        out.push(Draw::Tex{x:canvas.centred(*w as f32),y:0.0,w:*w as f32,h:*h as f32,tex:*tex,alpha:1.0});
    }}
    let selected=menu.row.position(menu.hidden);
    let [x,y,w,h]=cell(canvas,selected,rows.len(),cols,pause);
    let target=position(menu.row,menu.hidden);
    let pitch=if pause {120.0}else{(canvas.w-2.0*MENU_INSET-24.0)/cols as f32};
    rounded(out,x+(frame.grid_focus[0]-target[0])*pitch,y+(frame.grid_focus[1]-target[1])*if pause{124.0}else if menu.row.page()==QuickRow::SETTINGS{112.0}else{PITCH},w,h,22.0,crate::expressive::SELECTED);
    for (i,row) in rows.into_iter().enumerate(){let [x,y,w,h]=cell(canvas,i,menu.row.shown(menu.hidden).len(),cols,pause);let (tex,tw,th)=faces.labels[row.index()][usize::from(i==selected)];out.push(Draw::Tex{x:x+(w-tw as f32)/2.0,y:y+(h-th as f32)/2.0,w:tw as f32,h:th as f32,tex,alpha:1.0});}
}
