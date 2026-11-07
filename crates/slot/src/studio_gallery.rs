//! Sticker previews and GPU motion adapted from Slot's on-device label printer.
//! Raster work runs on the Studio worker; animation only moves cached quads.
use crate::studio::LabelKind;
use slot_ui::{Canvas, Draw, MenuFrame, TexId, UndoFace};

pub const CELL_W: u32 = 160;
pub const CELL_H: u32 = 96;
pub const APPLY_MS: u64 = 760;
pub const SHEET_PITCH: f32 = 120.0;

pub struct Gallery {
    pub pixels: Option<UndoFace>,
    pub slots: Vec<(LabelKind, u32, u32)>,
}
impl Gallery {
    pub fn new(labels: Vec<(LabelKind, UndoFace)>) -> Self {
        let mut pixels = UndoFace { w: CELL_W, h: CELL_H * labels.len().max(1) as u32,
            rgba: vec![0; (CELL_W * CELL_H * labels.len().max(1) as u32 * 4) as usize] };
        let mut slots = Vec::new();
        for (i, (kind, face)) in labels.into_iter().enumerate() {
            for y in 0..face.h {
                let from = (y * face.w * 4) as usize;
                let to = (((i as u32 * CELL_H + y) * CELL_W) * 4) as usize;
                pixels.rgba[to..to + face.w as usize * 4]
                    .copy_from_slice(&face.rgba[from..from + face.w as usize * 4]);
            }
            slots.push((kind, face.w, face.h));
        }
        Self { pixels: Some(pixels), slots }
    }
    pub fn rect(canvas: Canvas, index: usize, w: u32, h: u32) -> [f32; 4] {
        let x = canvas.w * 0.47 + 12.0;
        let width = canvas.w - 76.0 - x;
        let scale = (width / w as f32).min(1.0).min(60.0 / h as f32);
        [x + (width - w as f32 * scale) / 2.0,
            116.0 + index as f32 * SHEET_PITCH + (60.0 - h as f32 * scale) / 2.0,
            w as f32 * scale, h as f32 * scale]
    }
    pub fn draw(&self, tex: TexId, kind: LabelKind, rect: [f32; 4], alpha: f32, out: &mut Vec<Draw>) {
        let Some((i, (_, w, h))) = self.slots.iter().enumerate().find(|(_, (k, _, _))| *k == kind) else { return };
        let [x,y,w0,h0] = rect;
        out.push(Draw::Part { x,y,w:w0,h:h0,tex,alpha,
            uv:[0.0, i as f32 / self.slots.len().max(1) as f32,
                *w as f32 / CELL_W as f32, *h as f32 / (CELL_H * self.slots.len().max(1) as u32) as f32] });
    }
}

pub fn transform(canvas: Canvas, frame: MenuFrame, out: &mut Vec<Draw>, begin: usize) {
    let dx = frame.page_shift * (canvas.w - 2.0 * slot_ui::expressive::MENU_INSET);
    for draw in &mut out[begin..] {
        if let Draw::Part { x, alpha, .. } = draw { *x += dx; *alpha *= frame.opacity; }
    }
    slot_ui::clip_menu_body(out, begin, canvas);
}

/// Closed-form damped spring: deterministic at any frame rate, with a quiet settle.
pub fn spring(elapsed: u64, stiffness: f32, damping: f32, finish: u64) -> f32 {
    if elapsed >= finish { return 1.0; }
    let t = elapsed as f32 / 1000.0;
    let omega = stiffness.sqrt();
    let frequency = omega * (1.0 - damping * damping).sqrt();
    1.0 - (-damping * omega * t).exp()
        * ((frequency * t).cos() + damping * omega / frequency * (frequency * t).sin())
}

pub fn entry_rect(from: [f32; 4], to: [f32; 4], progress: f32) -> [f32; 4] {
    std::array::from_fn(|i| from[i] + (to[i] - from[i]) * progress)
}

/// Peel from the sheet, carry on an arc, then spring into the label well.
pub fn application_rect(from: [f32;4], to: [f32;4], elapsed: u64) -> [f32;4] {
    let p = spring(elapsed.saturating_sub(60), 360.0, 0.78, APPLY_MS - 60);
    let mut rect = entry_rect(from, to, p);
    let lift = (p.clamp(0.0,1.0) * std::f32::consts::PI).sin() * 30.0;
    rect[1] -= lift;
    rect
}

/// Only the option column participates in submenu travel. Left-hand title,
/// cartridge and outer chrome have independent, stable ownership.
pub fn draw_menu(canvas: Canvas, tex: TexId, frame: MenuFrame, out: &mut Vec<Draw>) {
    let begin = out.len();
    let edge = canvas.w * 0.47;
    let dx = frame.page_shift * (canvas.w - 2.0 * slot_ui::expressive::MENU_INSET);
    let top = slot_ui::expressive::CARD_TOP;
    let h = slot_ui::expressive::CARD_BOTTOM - top;
    let left=(edge+dx).max(edge-4.0);
    let right=(canvas.w+dx).min(canvas.w-slot_ui::expressive::MENU_INSET);
    if right>left {
        out.push(Draw::Part { x:left,y:top,w:right-left,h,tex,alpha:frame.opacity,
            uv:[(left-dx)/canvas.w,top/canvas.h,(right-left)/canvas.w,h/canvas.h] });
    }
    slot_ui::clip_menu_body(out, begin, canvas);
    if !frame.outgoing {
        let h = slot_ui::expressive::TITLE_H;
        out.push(Draw::Part { x:0.0,y:0.0,w:canvas.w,h,tex,alpha:1.0,uv:[0.0,0.0,1.0,h/canvas.h] });
        let y = slot_ui::expressive::FOOTER_Y;
        out.push(Draw::Part { x:0.0,y,w:canvas.w,h:44.0,tex,alpha:1.0,uv:[0.0,y/canvas.h,1.0,44.0/canvas.h] });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn submenu_text_stays_in_the_options_column_and_outer_chrome_is_anchored() {
        for w in [640.0,720.0] {for shift in [-0.8,-0.2,0.0,0.4] {
            let canvas=Canvas{w,h:480.0};
            let mut frame=MenuFrame::settled(slot_ui::QuickRow::Kids,&[]);frame.page_shift=shift;
            let mut out=Vec::new();draw_menu(canvas,TexId::from_raw(101),frame,&mut out);
            for draw in out {if let Draw::Part{x,y,w:width,..}=draw {
                if y==slot_ui::expressive::CARD_TOP {assert!(x>=w*0.47-4.0);assert!(x+width<=w-56.0+0.01);}
                else {assert_eq!(x,0.0);}
            }}
        }}
    }
    #[test]
    fn cartridge_spring_is_continuous_and_locks_to_its_final_pose() {
        let mut previous=spring(0,340.0,0.86,720);
        assert_eq!(previous,0.0);
        for ms in 1..=720 {let p=spring(ms,340.0,0.86,720);assert!((p-previous).abs()<0.02);previous=p;}
        assert_eq!(previous,1.0);
    }
    #[test]
    fn label_travel_starts_on_the_sheet_and_lands_on_the_well() {
        let from=[400.0,120.0,160.0,80.0]; let to=[110.0,170.0,130.0,64.0];
        assert_eq!(application_rect(from,to,0),from);
        for (a,b) in application_rect(from,to,APPLY_MS).iter().zip(to) {assert!((a-b).abs()<0.001);}
        let mid=application_rect(from,to,APPLY_MS/4);
        assert!(mid[0]>to[0] && mid[0]<from[0]);
        assert!(mid[1]<(from[1]+to[1])/2.0);
    }
    #[test]
    fn sticker_tiles_fit_the_card_in_both_layouts() {
        for w in [640.0,720.0] {for i in 0..2 {
            let r=Gallery::rect(Canvas{w,h:480.0},i,160,96);
            assert!(r[0]>=w*0.47 && r[0]+r[2]<=w-76.0);
            assert!(r[1]>=112.0 && r[1]+r[3]<slot_ui::expressive::CARD_BOTTOM);
        }}
    }
}
