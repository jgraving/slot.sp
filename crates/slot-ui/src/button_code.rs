//! Parent code uses the same controller artwork as the menu legends.
use crate::{bitmaps::ButtonLegend, quick_menu::button_face, UndoFace};

pub fn button_code_face(code: &[u8]) -> UndoFace {
    let (w, h) = (320, 64);
    let mut rgba = vec![0; (w * h * 4) as usize];
    let buttons = [ButtonLegend::A, ButtonLegend::B, ButtonLegend::X, ButtonLegend::Y,
        ButtonLegend::Up, ButtonLegend::Down, ButtonLegend::Left, ButtonLegend::Right];
    for i in 0..4 {
        let x = i * 80 + 8;
        if let Some(which) = code.get(i as usize).and_then(|b| buttons.get(*b as usize)) {
            let face = button_face(*which);
            crate::plate::blit(&mut rgba, w, &face.rgba, face.w, face.h,
                x + (64 - face.w) / 2, (h - face.h) / 2);
        } else {
            // A visible empty slot makes progress clear before the first press.
            for y in 53..56 {
                for xx in x + 8..x + 56 {
                    let at = ((y * w + xx) * 4) as usize;
                    rgba[at..at + 4].copy_from_slice(&[120, 120, 130, 255]);
                }
            }
        }
    }
    UndoFace { rgba, w, h }
}
