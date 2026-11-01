use crate::{Canvas, Draw, MenuFrame, UndoFace, MENU_PAD};
pub const MENU_SLIDER_W: f32 = 140.0;
pub const MENU_SLIDER_SPAN: f32 = 280.0;
pub const SLIDER_TRACK_H: f32 = 6.0;
pub const SLIDER_THUMB: f32 = 14.0;
const THUMB_COLOUR: [f32; 4] = [0.92, 0.95, 0.98, 1.0];

/// One slider vocabulary for settings and transient volume/display feedback.
pub fn draw_slider(out: &mut Vec<Draw>, x: f32, y: f32, w: f32, value: f32, alpha: f32) {
    let value = value.clamp(0.0, 1.0);
    let track = [0.33, 0.36, 0.40, 0.65 * alpha];
    let mut fill = crate::expressive::ACCENT;
    fill[3] *= alpha;
    let mut thumb = THUMB_COLOUR;
    thumb[3] *= alpha;
    crate::expressive::rounded(out, x, y + 12.0, w, SLIDER_TRACK_H, 3.0, track);
    crate::expressive::rounded(out, x, y + 12.0, w * value, SLIDER_TRACK_H, 3.0, fill);
    let cx = x + SLIDER_THUMB / 2.0 + (w - SLIDER_THUMB) * value;
    crate::expressive::rounded(
        out,
        cx - SLIDER_THUMB / 2.0,
        y + 8.0,
        SLIDER_THUMB,
        SLIDER_THUMB,
        7.0,
        thumb,
    );
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MenuSlider {
    pub span:f32,
    pub width:f32,
    pub id: u64,
    pub fraction: f32,
    pub y: f32,
}
pub fn draw_menu_sliders(
    canvas: Canvas,
    sliders: &[MenuSlider],
    motion: &crate::expressive::SliderMotion,
    frame: MenuFrame,
    out: &mut Vec<Draw>,
) {
    let begin = out.len();
    let dx = frame.page_shift * (canvas.w - 2.0 * crate::expressive::MENU_INSET);
    for s in sliders {
        draw_slider(
            out,
            canvas.w - crate::QUICK_EDGE - s.span + dx,
            s.y,
            s.width,
            motion.value(s.id, s.fraction),
            frame.opacity,
        );
    }
    crate::quick_menu::clip_quads(
        out,
        begin,
        [
            crate::expressive::MENU_INSET,
            crate::expressive::CARD_TOP,
            canvas.w - crate::expressive::MENU_INSET,
            crate::expressive::CARD_BOTTOM,
        ],
    );
}
pub fn setting_slider(label: &str, value: &str) -> Option<f32> {
    matches!(
        label,
        "Volume" | "Volume on disconnect" | "Volume limit" | "Brightness" | "Warmth" | "Night warmth" | "Color temp"
    )
    .then(|| {
        value
            .trim_start_matches("Warm ")
            .trim_end_matches('%')
            .replace("Neutral", "0")
            .parse::<f32>()
            .ok()
            .map(|v| (v / 100.0).clamp(0.0, 1.0))
    })
    .flatten()
}
pub fn slider_face(value: f32) -> UndoFace {slider_face_with_width(value,MENU_SLIDER_W)}
pub fn slider_face_with_width(value:f32,width:f32)->UndoFace {
    let (w, h) = (width as u32 + 2 * MENU_PAD, 40);
    let mut face = UndoFace {
        rgba: vec![0; (w * h * 4) as usize],
        w,
        h,
    };
    let mut out = vec![];
    draw_slider(&mut out, MENU_PAD as f32, 5.0, width, value, 1.0);
    for d in out {
        if let Draw::RoundRect {
            x,
            y,
            w,
            h,
            radius,
            colour,
        } = d
        {
            crate::expressive::raster_round(&mut face, x, y, w, h, radius, colour);
        }
    }
    face
}
