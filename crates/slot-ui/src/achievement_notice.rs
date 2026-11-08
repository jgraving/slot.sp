//! Achievement pills reserve independent space for complete badges and readable copy.
use crate::{text, Canvas, UndoFace};

pub fn achievement_notice_face(title: &str, detail: &str, badge: Option<(&[u8], u32, u32)>, canvas: Canvas, light: bool) -> UndoFace {
    let max_w = (canvas.w as u32).saturating_sub(64).min(600);
    let left = if badge.is_some() {88} else {28};
    let words_w = max_w.saturating_sub(left + 28);
    let title = fitted(title, text::Role::Focus, words_w, 2, 19.0, 17.0);
    let detail = fitted(detail, text::Role::Detail, words_w, 2, 16.0, 15.0);
    let longest = [&title, &detail].into_iter().flat_map(|layout| {
        let font = text::font(layout.role).unwrap();
        layout.lines.iter().map(move |line| text::line_width(font, line, layout.px, layout.tracking).ceil() as u32)
    }).max().unwrap_or(0);
    let w = (longest + left + 32).min(max_w).max(240);
    let title_h = title.lines.len() as u32 * 25;
    let detail_h = detail.lines.len() as u32 * 21;
    let h = (title_h + detail_h + 32).max(84);
    let mut face = UndoFace {w, h, rgba: vec![0; (w * h * 4) as usize]};
    let ink = if light {slot_gfx::ui_palette::INK} else {[250,248,243]};
    let secondary = if light {[74,83,87]} else {[211,216,220]};
    lines(&mut face, &title, left, 14, 25, ink);
    lines(&mut face, &detail, left, 18 + title_h, 21, secondary);
    if let Some((rgba, sw, sh)) = badge.filter(|(rgba, sw, sh)|
        *sw > 0 && *sh > 0 && rgba.len() == (*sw * *sh * 4) as usize) {
        // Contain the full image, never crop or stretch. Never enlarge small sources.
        let scale = (48.0 / sw as f32).min(48.0 / sh as f32).min(1.0);
        let bw = (sw as f32 * scale).round().max(1.0) as u32;
        let bh = (sh as f32 * scale).round().max(1.0) as u32;
        let (ox, oy) = (24 + (48 - bw) / 2, (h - bh) / 2);
        for y in 0..bh {
            for x in 0..bw {
                let pixel = crate::art::box_average(rgba, sw, sh,
                    x as f32 * sw as f32 / bw as f32, y as f32 * sh as f32 / bh as f32,
                    (x + 1) as f32 * sw as f32 / bw as f32, (y + 1) as f32 * sh as f32 / bh as f32);
                let to = (((oy+y)*w+ox+x)*4) as usize;
                face.rgba[to..to+4].copy_from_slice(&pixel);
            }
        }
    }
    face
}

fn fitted(value: &str, role: text::Role, width: u32, count: usize, max: f32, min: f32) -> text::Layout {
    let font = text::font(role).expect("bundled notification font");
    let mut layout = text::fit_preserving_case(font, value, width as f32, count, max, min);
    if layout.lines.join(" ").split_whitespace().collect::<String>() != value.split_whitespace().collect::<String>() {
        if let Some(last) = layout.lines.last_mut() {
            last.push('…');
            while text::line_width(font, last, layout.px, layout.tracking) > width as f32 {
                last.pop();
                if last.pop().is_none() {break;}
                last.push('…');
            }
        }
    }
    layout
}

fn lines(face: &mut UndoFace, layout: &text::Layout, x: u32, y: u32, pitch: u32, ink: [u8;3]) {
    let font = text::font(layout.role).unwrap();
    for (i, line) in layout.lines.iter().enumerate() {
        let width = (text::line_width(font, line, layout.px, layout.tracking).ceil() as u32 + 4).min(face.w-x-24);
        let one = text::Layout {lines:vec![line.clone()], px:layout.px, tracking:layout.tracking, role:layout.role};
        let mut rgba = vec![0; (width*pitch*4) as usize];
        text::draw_centred(&mut rgba, width, pitch, &one, ink);
        crate::plate::blit(&mut face.rgba, face.w, &rgba, width, pitch, x, y + i as u32*pitch);
    }
}
