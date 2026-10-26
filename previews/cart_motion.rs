//! Capture the production cartridge motion, geometry and recorded contact sounds.
#[path = "egl_surface.rs"]
mod egl_surface;

use slot::app::App;
use slot_gfx::{Canvas, Compositor, Draw};
use slot_input::Action;
use slot_ui::{Shelf, SlotChrome};
use std::{fs::File, io::Write, process::{Command, Stdio}};

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let [root, out] = args.as_slice() else { panic!("ROOT OUT") };
    let carts = slot_store::scan(std::path::Path::new(root)).unwrap();
    let cart = carts.iter().find(|c| c.stem == "Emerald").unwrap().clone();
    let other = carts.iter().find(|c| c.stem == "Fusion").unwrap().clone();
    let mut app = App::new(vec![cart.clone(), other]);
    let surface = egl_surface::HeadlessSurface::new().unwrap();
    let mut c = Compositor::new(&surface).unwrap();
    let face = slot_ui::cart_face(&cart);
    let face = c.create_texture(face.w, face.h, &face.rgba);
    let interior = slot_ui::cart_interior(&cart);
    let interior = c.create_texture(interior.w, interior.h, &interior.rgba);
    let backing = slot_ui::cart_backing(None);
    let backing = c.create_texture(backing.w, backing.h, &backing.rgba);
    let kid = slot_ui::kid_smiley_face();
    let kid = (c.create_texture(kid.w, kid.h, &kid.rgba), kid.w as f32, kid.h as f32);
    let approved = slot::kid_policy::Policy::read(std::path::Path::new(root)).unwrap().allows(&cart);
    let [back, front] = slot_ui::shell_slot_faces(slot_store::ShellColour::Graphite);
    let back = c.create_texture(720, slot_ui::SLOT_H, back);
    let front = c.create_texture(720, slot_ui::SLOT_H, front);
    let shelf = Shelf::new(vec![cart.clone()]);
    let (rest, scale) = shelf.selected_at(Canvas::FRAMED);
    let silent = format!("{out}/cart-motion-silent.mp4");
    let mut encoder = Command::new("ffmpeg").args([
        "-v", "error", "-y", "-f", "rawvideo", "-pix_fmt", "rgba", "-s", "720x480",
        "-r", "60", "-i", "-", "-c:v", "libx264", "-preset", "slow", "-crf", "16",
        "-pix_fmt", "yuv420p", &silent,
    ]).stdin(Stdio::piped()).spawn().unwrap();
    let mut pipe = encoder.stdin.take().unwrap();
    let mut audio = vec![0i16; 240 * 800 * 2];
    let mut trace = File::create(format!("{out}/cart-motion.csv")).unwrap();
    writeln!(trace, "frame,seconds,depth,sound,contact_seconds").unwrap();
    let mut heard = Vec::new();
    for frame in 0..240usize {
        if frame == 24 { app.apply(Action::Insert); app.on_core_ready(); }
        if frame == 132 { app.apply(Action::Eject); }
        app.tick_ms((frame * 1_000 / 60) as u64);
        app.update(1.0 / 60.0);
        let sound = app.take_sfx();
        if let Some(sound) = sound {
            let offset = frame * 800 * 2;
            for (dst, src) in audio[offset..].iter_mut().zip(sound.render(48_000)) {
                *dst = dst.saturating_add(src);
            }
            heard.push(sound);
        }
        let depth = app.seat();
        writeln!(trace, "{frame},{:.6},{depth:.6},{:?},{:.6}", frame as f32 / 60.0,
            sound, frame as f32 / 60.0 + sound.map_or(0.0, |s| s.lead())).unwrap();
        let mut draws = vec![Draw::Lava {
            w: 720.0, h: 480.0, phase: slot_gfx::lava_phase((frame * 1_000 / 60) as u64),
        }];
        SlotChrome {
            cart: &cart, face: Some(face), rest, scale, seat: depth,
            alert: None, dim: 0.0, screen: 0.0, game: false, over: true,
            behind: [Some(interior), Some(backing)], slot: Some((back, front)),
            canvas: Canvas::FRAMED,
        }.draw_with_marks(None, approved.then_some(kid), &mut draws);
        c.begin_frame(); c.draw_list(&draws);
        pipe.write_all(&c.read_frame()).unwrap();
    }
    assert_eq!(heard, vec![slot::audio::Sfx::Insert, slot::audio::Sfx::Eject]);
    assert_eq!(app.seat(), 0.0);
    drop(pipe);
    assert!(encoder.wait().unwrap().success());
    let pcm = format!("{out}/cart-motion.pcm");
    let mut raw = File::create(&pcm).unwrap();
    for sample in audio { raw.write_all(&sample.to_le_bytes()).unwrap(); }
    drop(raw);
    assert!(Command::new("ffmpeg").args([
        "-v", "error", "-y", "-i", &silent, "-f", "s16le", "-ar", "48000", "-ac", "2",
        "-i", &pcm, "-c:v", "copy", "-c:a", "aac", "-b:a", "192k", "-movflags", "+faststart",
        &format!("{out}/cart-motion.mp4"),
    ]).status().unwrap().success());
    std::fs::remove_file(pcm).unwrap();
    std::fs::remove_file(silent).unwrap();
}
