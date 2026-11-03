//! Filled capsules share the charging palette and contrast on both sides of the level.
use crate::{Icon, UndoFace, text};
pub const METER_W: u32 = 176;
pub const METER_H: u32 = 52;

pub fn filled_meter_face(label: &str, icon: Option<Icon>, fraction: f32, light: bool, w: u32, h: u32) -> UndoFace {
    let colours=crate::battery::charge_colours(light);
    let edge=w as f32*fraction.clamp(0.0,1.0);
    let mut face=UndoFace{rgba:vec![0;(w*h*4)as usize],w,h};
    crate::expressive::raster_round(&mut face,0.0,0.0,w as f32,h as f32,h as f32/2.0,colours.track);
    for (i,p) in face.rgba.chunks_exact_mut(4).enumerate() {
        if ((i as u32%w) as f32) < edge {for c in 0..3 {p[c]=(colours.fill[c]*255.0).round()as u8;}}
    }
    let font=text::font(text::Role::Value).expect("meter font");
    let px=if h>=48 {22.0}else{20.0};
    let layout=text::fit_preserving_case(font,label,w as f32-76.0,1,px,px);
    let tw=layout.lines.first().map_or(0.0,|line|text::line_width(font,line,layout.px,layout.tracking)).ceil()as u32+2;
    let mut words=vec![0;(tw*h*4)as usize];
    text::draw_centred(&mut words,tw,h,&layout,[255;3]);
    let size=24;
    let left=w.saturating_sub(tw+size+12)/2;
    let mut mask=vec![0;(w*h*4)as usize];
    crate::plate::blit(&mut mask,w,&words,tw,h,left+size+12,0);
    let glyph=if let Some(icon)=icon {crate::control_icon_face(icon,size,[255;3])}else{
        let svg=r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><g fill="none" stroke="#fff" stroke-width="2" stroke-linecap="round"><circle cx="12" cy="12" r="9"/><path d="M12 6v6l4 2"/></g></svg>"##;
        crate::CartFace{rgba:crate::art::render_svg(svg,size,size).expect("time icon"),w:size,h:size}
    };
    crate::plate::blit(&mut mask,w,&glyph.rgba,glyph.w,glyph.h,left,(h-glyph.h)/2);
    for (i,(p,ink)) in face.rgba.chunks_exact_mut(4).zip(mask.chunks_exact(4)).enumerate() {
        let colour=if ((i as u32%w)as f32) < edge {colours.filled_ink}else{colours.empty_ink};
        let a=ink[3]as f32/255.0;
        for c in 0..3 {p[c]=(colour[c]as f32*a+p[c]as f32*(1.0-a)).round()as u8;}
    }
    face
}
