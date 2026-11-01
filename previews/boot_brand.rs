//! Capture the exact boot overlay drawing path against a quiet preview backdrop.
#[path="egl_surface.rs"] mod egl_surface;
use slot_gfx::{Compositor,Draw,Canvas};
use slot::boot_motion::{BootMotion,BOOT_MS};
use std::{io::Write,process::{Command,Stdio}};
fn main(){
 let out=std::env::args().nth(1).unwrap();
 let surface=egl_surface::HeadlessSurface::new().unwrap();let mut c=Compositor::new(&surface).unwrap();
 let brand=BootMotion::upload(&mut c).unwrap();
 let mut encoder=Command::new("ffmpeg").args(["-v","error","-y","-f","rawvideo","-pix_fmt","rgba","-s","720x480","-r","60","-i","-","-c:v","libx264","-crf","16","-pix_fmt","yuv420p","-movflags","+faststart",&format!("{out}/boot-brand.mp4")]).stdin(Stdio::piped()).spawn().unwrap();
 let mut pipe=encoder.stdin.take().unwrap();
 for frame in 0..=90 {
  let ms=frame*1000/60;
  let mut draws=vec![Draw::Lava{w:720.,h:480.,phase:0.}];
  if ms<BOOT_MS {brand.draw(ms,Canvas::FRAMED,&mut draws);}
  c.begin_frame();c.draw_list(&draws);pipe.write_all(&c.read_frame()).unwrap();
 }
 drop(pipe);assert!(encoder.wait().unwrap().success());brand.release(&mut c);
}
