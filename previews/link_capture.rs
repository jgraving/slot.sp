// Render the actual cable artwork and timing. No network session is started.
use slot::{app::{GameMenu,LinkRow},link_kind::LinkKind,link_screen::{Sprite,LinkSprites,draw_link_art}};
use slot_gfx::{Compositor};
use slot_ui::*;
pub fn capture(c:&mut Compositor,out:&str){
 fn sprite(c:&mut Compositor,f:CartFace)->Sprite{Sprite{tex:c.create_texture(f.w,f.h,&f.rgba),w:f.w,h:f.h}}
 let a=link_art();
 let s=LinkSprites{port:sprite(c,a.port_finishes.into_iter().next().unwrap()),plug_host:sprite(c,a.plug_host),plug_join:sprite(c,a.plug_join),adapter:sprite(c,a.adapter),arcs_right:a.arcs_right.map(|f|sprite(c,f)),arcs_left:a.arcs_left.map(|f|sprite(c,f)),clicks:sprite(c,a.clicks),arrow_left:sprite(c,a.arrow_left),arrow_right:sprite(c,a.arrow_right)};
 let ground=c.create_texture(720,480,default_background().unwrap());
 let mut clip=super::Clip::start(format!("{out}/link.mp4"));
 for frame in 0..240 {
  let now=frame*1000/60;
  let menu=if now<600{GameMenu::Pick(LinkRow::Host)}else if now<1800{GameMenu::Working{role:LinkRow::Host,step:slot::link_start::LinkStep::Waiting,since:600}}else if now<3000{GameMenu::Linked{role:LinkRow::Host,worked:600,since:1800,opened:true}}else{GameMenu::Unplug{role:LinkRow::Host,since:3000}};
  let mut draws=vec![];draw_ground(Canvas::FRAMED,Some((ground,720)),&mut draws);
  draw_link_art(Canvas::FRAMED,menu,LinkKind::Cable,now,&s,&mut draws);
  c.begin_frame();c.draw_list(&draws);clip.frames.send(c.read_frame()).unwrap();clip.written+=1;
 }
 clip.finish();
}
