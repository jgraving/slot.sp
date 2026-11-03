//! Parent code uses the same controller artwork as the menu legends.
use crate::{bitmaps::ButtonLegend, quick_menu::controller_button_face, UndoFace};

/// A masked variable-length credential code. The parent lock's four-button code is separate.
pub fn credential_code_face_on(canvas: crate::Canvas, instruction: &str, count: usize) -> UndoFace {
    credential_code_face_with_lock_on(canvas,instruction,count,false)
}
pub fn credential_code_face_with_lock_on(canvas:crate::Canvas,instruction:&str,count:usize,locked:bool)->UndoFace {
    masked_pin_face_on(canvas,"Device code",instruction,count,if locked {9}else{8})
}
pub fn masked_pin_face_on(canvas:crate::Canvas,title:&str,instruction:&str,count:usize,footer:u8)->UndoFace {
    masked_pin_state_face_on(canvas,title,instruction,count,footer,0,0)
}
pub fn masked_pin_state_face_on(canvas:crate::Canvas,title:&str,instruction:&str,count:usize,footer:u8,minimum:usize,shake:i32)->UndoFace {
    let mut face=crate::parent_footer_face_on(canvas,footer,false);
    let title=crate::quick_heading_face(title);
    let x=face.w.saturating_sub(title.w)/2;crate::plate::blit(&mut face.rgba,face.w,&title.rgba,title.w,title.h,x,0);
    let words=crate::quick_value_face(instruction,true);
    let x=(face.w.saturating_sub(words.w)as i32/2+shake).max(0)as u32;crate::plate::blit(&mut face.rgba,face.w,&words.rgba,words.w,words.h,x,146);
    let dots=count.min(12)as u32;
    let left=face.w.saturating_sub(dots*32)/2;
    for i in 0..dots {
        let colour=if count>=minimum {[164,220,211,255]}else{[120,120,130,190]};
        crate::page::rounded(&mut face.rgba,face.w,((left as i32+i as i32*32+7+shake).max(0)as u32,222,18,18),8,colour);
    }
    face
}

pub fn button_code_face(code: &[u8]) -> UndoFace {
    let (w, h) = (256, 56);
    let mut rgba = vec![0; (w * h * 4) as usize];
    let buttons = [ButtonLegend::A, ButtonLegend::B, ButtonLegend::X, ButtonLegend::Y,
        ButtonLegend::Up, ButtonLegend::Down, ButtonLegend::Left, ButtonLegend::Right,
        ButtonLegend::L1, ButtonLegend::R1, ButtonLegend::L2, ButtonLegend::R2];
    for i in 0..4 {
        let x = i * 64;
        if let Some(which) = code.get(i as usize).and_then(|b| buttons.get(*b as usize)) {
            let face = controller_button_face(*which, 40);
            crate::plate::blit(&mut rgba, w, &face.rgba, face.w, face.h,
                x + (64 - face.w) / 2, (h - face.h) / 2);
        } else {
            // A visible empty slot makes progress clear before the first press.
            for y in 26..30 {
                for xx in x + 12..x + 52 {
                    let at = ((y * w + xx) * 4) as usize;
                    rgba[at..at + 4].copy_from_slice(&[120, 120, 130, 255]);
                }
            }
        }
    }
    UndoFace { rgba, w, h }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn credential_dots_render_at_every_entry_length_on_both_panels() {
        for w in [640.0,720.0] {for count in [0,1,3,4,5,8,12,13,32,384] {
            let face=credential_code_face_on(crate::Canvas{w,h:480.0},"Enter your button code",count);
            assert_eq!(face.rgba.len(),(face.w*face.h*4)as usize);
            let left=face.w.saturating_sub((count.min(12) as u32)*32)/2+7;
            if count>0 {assert!(face.rgba[((222*face.w+left+9)*4+3)as usize]>0);} else {assert!(face.rgba[(220*face.w*4)as usize..(244*face.w*4)as usize].iter().all(|&v|v==0));}
        }}
    }
}
