//! Baked variable-font glass reveal for the website hero.
use slot_gfx::{Compositor, Draw, TexId};

pub const FRAMES: usize = 48;
pub const REVEAL_MS: u64 = 900;
const COLS: usize = 4;
const TILE_W: u32 = 480;
const TILE_H: u32 = 128;
const ATLAS: &[u8] = include_bytes!("../../../assets/brand-motion.png");

pub fn frame_at(ms: u64) -> usize {
    (ms.min(REVEAL_MS) * (FRAMES as u64 - 1) / REVEAL_MS) as usize
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

    // Return the atlas's 11.8 MB of GPU storage as soon as the intro is over.
    pub fn release(self, c: &mut Compositor) { c.update_texture(self.tex,1,1,&[0,0,0,0]); }
}
