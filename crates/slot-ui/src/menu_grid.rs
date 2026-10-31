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
    // Filled silhouettes with transparent inset details stay legible on the handheld.
    let (label,paths)=match row {
        QuickRow::General=>("General",r#"<path fill-rule="evenodd" d="M40.34 19.31 L44.46 19.28 L44.46 28.72 L40.34 28.69 L38.87 32.24 L41.81 35.13 L35.13 41.81 L32.24 38.87 L28.69 40.34 L28.72 44.46 L19.28 44.46 L19.31 40.34 L15.76 38.87 L12.87 41.81 L6.19 35.13 L9.13 32.24 L7.66 28.69 L3.54 28.72 L3.54 19.28 L7.66 19.31 L9.13 15.76 L6.19 12.87 L12.87 6.19 L15.76 9.13 L19.31 7.66 L19.28 3.54 L28.72 3.54 L28.69 7.66 L32.24 9.13 L35.13 6.19 L41.81 12.87 L38.87 15.76z M31 24a7 7 0 1 0-14 0 7 7 0 0 0 14 0"/>"#),
        QuickRow::Screen=>("Screen",r#"<path fill-rule="evenodd" d="M10 3h28a4 4 0 0 1 4 4v18H6V7a4 4 0 0 1 4-4z M12 8v12h24V8z M6 28h36v13a4 4 0 0 1-4 4H10a4 4 0 0 1-4-4z M14 31v4h-4v3h4v4h3v-4h4v-3h-4v-4z M34 33a2.5 2.5 0 1 0 0 5 2.5 2.5 0 0 0 0-5z M39 38a2 2 0 1 0 0 4 2 2 0 0 0 0-4z"/>"#),
        QuickRow::Bluetooth=>("Sound",r#"<path d="M5 18h9l12-10v32L14 30H5zM31 16v16a11 11 0 0 0 0-16M36 8v4a15 15 0 0 1 0 24v4a19 19 0 0 0 0-32"/>"#),
        QuickRow::Game=>("Gameplay",r#"<path fill-rule="evenodd" d="M8 5h32a3 3 0 0 1 3 3v32a3 3 0 0 1-3 3H8a3 3 0 0 1-3-3V8a3 3 0 0 1 3-3z M12 11v18h24V11z M14 35v4h3v-4z M22 35v4h3v-4z M30 35v4h3v-4z"/>"#),
        QuickRow::Power=>("Power & lid",r#"<path d="M22 3h4v20h-4zM16 9v5a14 14 0 1 0 16 0V9a19 19 0 1 1-16 0"/>"#),
        QuickRow::Controls=>("Device\ncontrols",r#"<path fill-rule="evenodd" d="M17 4h14v13h13v14H31v13H17V31H4V17h13z M21 21v6h6v-6z"/>"#),
        QuickRow::Kids=>("Parental\ncontrols",r#"<path fill-rule="evenodd" d="M24 4a20 20 0 1 0 0 40 20 20 0 0 0 0-40z M17 15a2.5 2.5 0 1 0 0 5 2.5 2.5 0 0 0 0-5z M31 15a2.5 2.5 0 1 0 0 5 2.5 2.5 0 0 0 0-5z M13 26a11 11 0 0 0 22 0h-4a7 7 0 0 1-14 0z"/>"#),
        _=>("About",r#"<path fill-rule="evenodd" d="M24 4a20 20 0 1 0 0 40 20 20 0 0 0 0-40z M22 21v14h4V21z M24 12a2.5 2.5 0 1 0 0 5 2.5 2.5 0 0 0 0-5z"/>"#),
    };
    let ink=if lit {"#effbf8"}else{"#dce4e7"};
    let svg=format!(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 48 48"><g fill="{ink}">{paths}</g></svg>"#);
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
    for (i,row) in menu.row.shown(menu.hidden).into_iter().enumerate() {
        let [x,y,w,h]=rect(canvas,i);
        let (tex,tw,th)=faces.labels[row.index()][usize::from(i==selected)];
        out.push(Draw::Tex{x:x+(w-tw as f32)/2.0,y:y+(h-th as f32)/2.0,w:tw as f32,h:th as f32,tex,alpha:1.0});
    }
}
