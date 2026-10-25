//! Product hero using the real App insertion/ejection clock and SlotChrome geometry.
#[path="egl_surface.rs"] mod egl_surface;
use slot::app::App;
use slot_gfx::{Compositor,Draw,TexId,Canvas};
use slot_input::Action;
use slot_ui::{CartFace,SlotChrome,Shelf};
use std::{fs::File,io::Write,process::{Command,Stdio}};
fn texture(c:&mut Compositor, face:CartFace)->TexId { c.create_texture(face.w,face.h,&face.rgba) }
fn png(c:&mut Compositor,path:&str)->TexId {
    let mut reader=png::Decoder::new(File::open(path).unwrap()).read_info().unwrap();
    let mut bytes=vec![0;reader.output_buffer_size()];let info=reader.next_frame(&mut bytes).unwrap();
    assert_eq!(info.color_type,png::ColorType::Rgba);
    c.create_texture(info.width,info.height,&bytes[..info.buffer_size()])
}
fn main(){
 let args:Vec<_>=std::env::args().skip(1).collect();let [root,out]=args.as_slice() else{panic!("ROOT OUT")};
 let carts=slot_store::scan(std::path::Path::new(root)).unwrap();
 let cart=carts.iter().find(|c|c.stem=="Emerald").unwrap().clone();
 let other=carts.iter().find(|c|c.stem=="Fusion").unwrap().clone();
 let mut clock=App::new(vec![cart.clone(),other]);
 let surface=egl_surface::HeadlessSurface::new().unwrap();let mut c=Compositor::new(&surface).unwrap();
 let face=texture(&mut c,slot_ui::cart_face(&cart));
 let interior=texture(&mut c,slot_ui::cart_interior(&cart));
 let backing=texture(&mut c,slot_ui::cart_backing(None));
 let star=slot_ui::star_face(3,[246,195,44]);let (sw,sh)=(star.w as f32,star.h as f32);let star=texture(&mut c,star);
 let [back,front]=slot_ui::shell_slot_faces(slot_store::ShellColour::Graphite);
 let back=c.create_texture(720,slot_ui::SLOT_H,back);let front=c.create_texture(720,slot_ui::SLOT_H,front);
 let vignette=png(&mut c,"assets/brand-vignette.png");
 let brand=slot::brand_motion::BrandMotion::upload(&mut c).unwrap();let light=png(&mut c,&format!("{out}/hero-light.png"));
 let shelf=Shelf::new(vec![cart.clone()]);let (rest,scale)=shelf.selected_at(Canvas::FRAMED);
 let mut encoder=Command::new("ffmpeg").args(["-v","error","-y","-f","rawvideo","-pix_fmt","rgba","-s","720x480","-r","60","-i","-","-c:v","libx264","-preset","slow","-crf","16","-pix_fmt","yuv420p","-movflags","+faststart",&format!("{out}/hero-glass.mp4")]).stdin(Stdio::piped()).spawn().unwrap();
 let mut pipe=encoder.stdin.take().unwrap();let mut first=None;let mut last=None;let mut seated=false;let mut trace=Vec::new();
 for frame in 0..390 {
  if frame==48 {clock.apply(Action::Insert);clock.on_core_ready();}
  if frame==290 {clock.apply(Action::Eject);}
  clock.tick_ms(frame*1000/60);clock.update(1.0/60.0);
  let seat=clock.seat();seated|=seat>=0.999;trace.push(seat);
  // A short, eased excursion through the production wallpaper keeps this
  // brief insertion loop calm and makes its endpoints exactly identical.
  let drift=(1000.0 * (1.0-(std::f64::consts::TAU * frame as f64 / 389.0).cos())).round() as u64;
  let mut draws=vec![Draw::Lava{w:720.,h:480.,phase:slot_gfx::lava_phase(drift)}];
  let enter=((frame as f32-52.)/24.).clamp(0.,1.);
  let leave=((290.-frame as f32)/28.).clamp(0.,1.);
  let focus=(enter*enter*(3.-2.*enter))*(leave*leave*(3.-2.*leave));
  draws.push(Draw::Tex{x:0.,y:0.,w:720.,h:480.,tex:vignette,alpha:focus});
  SlotChrome {cart:&cart,face:Some(face),rest,scale,seat,alert:None,dim:0.0,screen:0.0,game:false,over:true,behind:[Some(interior),Some(backing)],slot:Some((back,front)),canvas:Canvas::FRAMED}.draw_with_star(Some((star,sw,sh)),&mut draws);
  draws.push(Draw::Tex{x:0.,y:0.,w:720.,h:480.,tex:light,alpha:1.});
  // The cart vacates the centre before the mark enters; clear the mark
  // before ejection so the two objects never compete for the same space.
  if (68..280).contains(&frame) {
   let reveal=(frame-68) as f32/108.;
   let alpha=if frame>256 {(280-frame) as f32/24.} else {1.};
   assert!(seat>0.6,"logo entered before the cartridge cleared its space");
   let progress=reveal.clamp(0.,1.);
   let spring=(1.-progress).powi(2)*(progress*8.).cos();
   let width=560.-20.*spring;
   let ms=((frame-68)*slot::brand_motion::REVEAL_MS/108) as u64;
   brand.draw(ms,(720.-width)/2.,96.+18.*spring,width,alpha,&mut draws);
  }
  c.begin_frame();c.draw_list(&draws);let pixels=c.read_frame();
  if frame==0 {first=Some(pixels.clone());}
  if frame==200 {let mut enc=png::Encoder::new(File::create(format!("{out}/hero-glass.png")).unwrap(),720,480);enc.set_color(png::ColorType::Rgba);enc.set_depth(png::BitDepth::Eight);enc.write_header().unwrap().write_image_data(&pixels).unwrap();}
  if frame==389 {last=Some(pixels.clone());}
  pipe.write_all(&pixels).unwrap();
 }
 assert!(seated,"hero never seated the cartridge");assert!(clock.seat()<0.001,"hero did not return to the shelf");
 assert_eq!(first,last,"hero must loop without a pose or lighting jump");
 assert!(trace[48..78].windows(2).all(|w|w[1]>=w[0]),"insertion reversed");
 drop(pipe);assert!(encoder.wait().unwrap().success());
}
