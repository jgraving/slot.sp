//! Parent code uses the same controller artwork as the menu legends.
use crate::{bitmaps::ButtonLegend, quick_menu::controller_button_face, UndoFace};

/// A masked 16-button credential code. The parent lock's four-button code is separate.
pub fn credential_code_face_on(canvas: crate::Canvas, instruction: &str, count: usize) -> UndoFace {
    let mut face=crate::parent_footer_face_on(canvas,1,false);
    let title=crate::quick_heading_face("Security PIN");
    let x=face.w.saturating_sub(title.w)/2; crate::plate::blit(&mut face.rgba,face.w,&title.rgba,title.w,title.h,x,0);
    let words=crate::quick_value_face(instruction,true);
    let x=face.w.saturating_sub(words.w)/2; crate::plate::blit(&mut face.rgba,face.w,&words.rgba,words.w,words.h,x,146);
    let left=face.w.saturating_sub(320)/2;
    for i in 0..16 {
        let x=left+(i%8)*40+12; let y=208+(i/8)*44;
        let colour=if (i as usize)<count {[164,220,211,255]}else{[120,120,130,160]};
        crate::page::rounded(&mut face.rgba,face.w,(x,y,16,16),8,colour);
    }
    let detail=crate::quick_description_face(&["A/B/X/Y, D-pad and shoulder buttons", &format!("{} of 16 · checks after the last button",count.min(16))]);
    let x=face.w.saturating_sub(detail.w)/2; crate::plate::blit(&mut face.rgba,face.w,&detail.rgba,detail.w,detail.h,x,302);
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
