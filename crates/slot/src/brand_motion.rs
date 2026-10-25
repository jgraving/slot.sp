//! Shared baked variable-font glass reveal for the hero and the device intro.
use slot_gfx::{Canvas, Compositor, Draw, TexId};

pub const FRAMES: usize = 48;
pub const REVEAL_MS: u64 = 900;
pub const HOLD_MS: u64 = 220;
pub const FADE_MS: u64 = 180;
pub const BOOT_MS: u64 = REVEAL_MS + HOLD_MS + FADE_MS;
const COLS: usize = 4;
const TILE_W: u32 = 480;
const TILE_H: u32 = 128;
const ATLAS: &[u8] = include_bytes!("../../../assets/brand-motion.png");

pub fn frame_at(ms: u64) -> usize {
    (ms.min(REVEAL_MS) * (FRAMES as u64 - 1) / REVEAL_MS) as usize
}

/// The device intro plays the reveal backwards: it opens on the finished
/// mark, so it matches the firmware's static boot logo, and dissolves to
/// nothing instead of revealing the glass from empty.
pub fn boot_frame_at(ms: u64) -> usize {
    FRAMES - 1 - frame_at(ms)
}

pub fn boot_opacity(ms: u64) -> f32 {
    let t = (ms.saturating_sub(REVEAL_MS + HOLD_MS) as f32 / FADE_MS as f32).clamp(0.0, 1.0);
    1.0 - t * t * (3.0 - 2.0 * t)
}

pub struct BrandMotion { tex: TexId }
impl BrandMotion {
    pub fn upload(c: &mut Compositor) -> Option<Self> {
        let mut decoder = png::Decoder::new(std::io::Cursor::new(ATLAS));
        decoder.set_transformations(png::Transformations::normalize_to_color8());
        let mut reader = decoder.read_info().ok()?;
        let mut bytes = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut bytes).ok()?;
        if info.color_type != png::ColorType::Rgba || info.width != TILE_W * COLS as u32
            || info.height != TILE_H * (FRAMES / COLS) as u32 { return None; }
        Some(Self { tex: c.create_texture(info.width, info.height, &bytes[..info.buffer_size()]) })
    }

    pub fn draw(&self, ms: u64, x: f32, y: f32, width: f32, alpha: f32, out: &mut Vec<Draw>) {
        let frame = frame_at(ms);
        self.part(frame, x, y, width, alpha, out);
    }

    fn part(&self, frame: usize, x: f32, y: f32, width: f32, alpha: f32, out: &mut Vec<Draw>) {
        out.push(Draw::Part { x, y, w: width, h: width * TILE_H as f32 / TILE_W as f32,
            tex: self.tex, alpha,
            uv: [(frame % COLS) as f32 / COLS as f32,
                 (frame / COLS) as f32 / (FRAMES / COLS) as f32,
                 1.0 / COLS as f32, 1.0 / (FRAMES / COLS) as f32] });
    }

    pub fn boot(&self, ms: u64, canvas: Canvas, out: &mut Vec<Draw>) {
        let alpha = boot_opacity(ms);
        out.push(Draw::Rect { x:0., y:0., w:canvas.w, h:canvas.h,
            colour:[11./255.,14./255.,17./255.,alpha] });
        let width = 480.0_f32.min(canvas.w - 32.0);
        self.part(boot_frame_at(ms), canvas.centred(width), (canvas.h - width * 128. / 480.) / 2., width, alpha, out);
    }

    // Return the atlas's 11.8 MB of GPU storage as soon as the intro is over.
    pub fn release(self, c: &mut Compositor) { c.update_texture(self.tex,1,1,&[0,0,0,0]); }
}
