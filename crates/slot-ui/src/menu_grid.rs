//! Cached vector tiles and spatial navigation for the carousel's settings home.
use crate::{Canvas,Draw,QuickRow,UndoFace,text};
use crate::expressive::{MENU_INSET,rounded};
pub const COLUMNS:usize=4;
pub const TOP:f32=100.0;
pub const PITCH:f32=124.0;
pub const TILE_H:f32=112.0;

pub fn rect(canvas:Canvas,index:usize)->[f32;4] {
    let pitch=(canvas.w-2.0*MENU_INSET-24.0)/COLUMNS as f32;
    [MENU_INSET+12.0+(index%COLUMNS) as f32*pitch,TOP+(index/COLUMNS) as f32*PITCH,pitch-8.0,TILE_H]
}
pub fn navigate(row:QuickRow,hidden:&[QuickRow],dx:i32,dy:i32)->QuickRow {
    let rows=row.shown(hidden);
    let i=row.position(hidden);
    let next=if dx!=0 {
        let start=i/COLUMNS*COLUMNS;
        let n=(rows.len()-start).min(COLUMNS);
        start+((i-start) as i32+dx).rem_euclid(n as i32) as usize
    }else {
        let col=i%COLUMNS;
        let count=(rows.len()+COLUMNS-1-col)/COLUMNS;
        ((i/COLUMNS) as i32+dy).rem_euclid(count as i32) as usize*COLUMNS+col
    };
    rows[next]
}

pub(crate) fn tile_face(row:QuickRow,lit:bool)->UndoFace {
    let (label,paths)=match row {
        QuickRow::General=>("General",r#"<path d="m20 4 1-3h6l1 3 4 2 3-1 3 5-2 3v5l2 3-3 5-3-1-4 2-1 3h-6l-1-3-4-2-3 1-3-5 2-3v-5l-2-3 3-5 3 1z" transform="translate(0 7)"/><circle cx="24" cy="24" r="6"/>"#),
        QuickRow::Screen=>("Screen",r#"<rect x="5" y="8" width="38" height="26" rx="4"/><path d="M17 41h14M24 34v7"/>"#),
        QuickRow::Bluetooth=>("Sound",r#"<path d="m7 19 8 0 10-9v28l-10-9H7zM31 17q7 7 0 14M36 10q14 14 0 28"/>"#),
        QuickRow::Game=>("Gameplay",r#"<path d="M15 14h18q6 0 8 8l3 13q1 8-6 5l-8-7H18l-8 7q-7 3-6-5l3-13q2-8 8-8zM11 23h10M16 18v10"/><circle cx="32" cy="22" r="1.5"/><circle cx="37" cy="27" r="1.5"/>"#),
        QuickRow::Power=>("Power & lid",r#"<path d="M24 4v18M14 10a17 17 0 1 0 20 0"/>"#),
        QuickRow::Controls=>("Device\ncontrols",r#"<path d="M18 5h12v13h13v12H30v13H18V30H5V18h13z"/><path d="M22 13h4M12 22v4M35 22v4M22 35h4"/>"#),
        QuickRow::Kids=>("Parental\ncontrols",r#"<circle cx="24" cy="24" r="18"/><path d="M14 27q10 13 20 0"/><circle cx="17" cy="18" r="1.2"/><circle cx="31" cy="18" r="1.2"/>"#),
        _=>("About",r#"<circle cx="24" cy="24" r="18"/><path d="M24 22v12M24 14v1"/>"#),
    };
    let ink=if lit {"#b7f3e8"}else{"#dce4e7"};
    let svg=format!(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 48 48"><g fill="none" stroke="{ink}" stroke-width="2.8" stroke-linecap="round" stroke-linejoin="round">{paths}</g></svg>"#);
    let icon=crate::art::render_svg(&svg,44,44).expect("menu tile SVG");
    let (w,h)=(112,100);
    let mut rgba=vec![0;(w*h*4) as usize];
    for y in 0..44usize {let dst=((y+4)*w as usize+34)*4;rgba[dst..dst+44*4].copy_from_slice(&icon[y*44*4..(y+1)*44*4]);}
    let layout=text::Layout{lines:label.lines().map(str::to_owned).collect(),px:18.0,tracking:0.0,role:if lit{text::Role::Focus}else{text::Role::Menu}};
    let mut words=vec![0;(w*48*4) as usize];
    text::draw_centred(&mut words,w,48,&layout,if lit{[246,249,248]}else{[211,219,222]});
    rgba[(52*w*4) as usize..].copy_from_slice(&words);
    UndoFace{rgba,w,h}
}

pub(crate) fn draw(menu:&crate::quick_menu::QuickMenu<'_>,frame:crate::MenuFrame,out:&mut Vec<Draw>) {
    let Some(faces)=menu.faces else{return;};
    let canvas=menu.canvas;
    rounded(out,MENU_INSET,88.0,canvas.w-2.0*MENU_INSET,260.0,28.0,[0.095,0.12,0.14,1.0]);
    if let Some((_,(tex,w,h)))=faces.headings.iter().find(|(r,_)|*r==QuickRow::MAIN[0]) {
        out.push(Draw::Tex{x:canvas.centred(*w as f32),y:40.0,w:*w as f32,h:*h as f32,tex:*tex,alpha:1.0});
    }
    let selected=menu.row.position(menu.hidden);
    let [_,_,w,h]=rect(canvas,0);
    let pitch=(canvas.w-2.0*MENU_INSET-24.0)/COLUMNS as f32;
    let x=MENU_INSET+12.0+frame.grid_focus[0]*pitch;
    let y=TOP+frame.grid_focus[1]*PITCH;
    rounded(out,x,y,w,h,22.0,crate::expressive::SELECTED);
    // A tonal icon well keeps focus readable without dimming other options.
    rounded(out,x+w/2.0-28.0,y+3.0,56.0,56.0,20.0,[0.22,0.40,0.40,1.0]);
    for (i,row) in menu.row.shown(menu.hidden).into_iter().enumerate() {
        let [x,y,w,h]=rect(canvas,i);
        let (tex,tw,th)=faces.labels[row.index()][usize::from(i==selected)];
        out.push(Draw::Tex{x:x+(w-tw as f32)/2.0,y:y+(h-th as f32)/2.0,w:tw as f32,h:th as f32,tex,alpha:1.0});
    }
}
