//! Exercise the production GL background and export a short review clip.
#[path = "egl_surface.rs"] mod egl_surface;
use slot_gfx::{Compositor, Draw, lava_phase};
use std::io::Write;
use std::process::{Command, Stdio};
fn render(c: &mut Compositor, now: u64, marker: bool) -> Vec<u8> {
    c.begin_frame();
    let mut draws = vec![Draw::Lava { w: 720.0, h: 480.0, phase: lava_phase(now) }];
    if marker { draws.push(Draw::Rect { x: 20.0, y: 20.0, w: 20.0, h: 20.0, colour: [1.0, 0.0, 0.0, 1.0] }); }
    c.draw_list(&draws); c.read_frame()
}
fn main() {
    let out = std::env::args().nth(1).unwrap();
    let surface = egl_surface::HeadlessSurface::new().unwrap();
    let mut c = Compositor::new(&surface).unwrap();
    let a = render(&mut c, 0, false);
    let b = render(&mut c, 9000, false);
    assert_ne!(a, b, "gradient did not move");
    assert_eq!(&a[440*720*4..], &b[440*720*4..], "the slot's lower backdrop must remain steady");
    assert_eq!(a, render(&mut c, 36_000, false), "the motion must loop cleanly");
    let overlay = render(&mut c, 9000, true);
    let at = (25*720+25)*4;
    assert_eq!(&overlay[at..at+4], &[255, 0, 0, 255], "background shader leaked into UI sprites");
    let mut encoder = png::Encoder::new(std::fs::File::create(format!("{out}/lava.png")).unwrap(), 720, 480);
    encoder.set_color(png::ColorType::Rgba); encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header().unwrap().write_image_data(&a).unwrap();
    let mut ffmpeg = Command::new("ffmpeg").args(["-v","error","-y","-f","rawvideo","-pix_fmt","rgba","-s","720x480","-r","30","-i","-","-c:v","libx264","-crf","18","-pix_fmt","yuv420p","-movflags","+faststart",&format!("{out}/lava.mp4")]).stdin(Stdio::piped()).spawn().unwrap();
    let mut pipe = ffmpeg.stdin.take().unwrap();
    for frame in 0..360 { pipe.write_all(&render(&mut c, frame*1000/30, false)).unwrap(); }
    drop(pipe); assert!(ffmpeg.wait().unwrap().success());
}
