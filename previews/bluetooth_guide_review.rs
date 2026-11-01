//! Native Bluetooth guide screenshots; hardware responses are deterministic fixtures.
#[path="../../slot-ui/examples/support/mod.rs"] mod support;
use slot::{frontend::Frontend,bluetooth::Page};
use slot_gfx::Compositor;
use slot_input::{InputSource,RawEvent,Btn,Millis};
use slot_power::{Platform,Battery,Charge,LedState};
use slot_store::{SlotState,Appearance};
use slot_ui::QuickRow;
use std::{path::{Path,PathBuf},fs};
struct CapturePlatform(PathBuf);
impl Platform for CapturePlatform {
 fn set_backlight(&mut self,_:u8){}fn battery(&self)->Option<Battery>{Some(Battery{percent:57,charge:Charge::Charging})}fn charge(&self)->Charge{Charge::Charging}
 fn set_led(&mut self,_:LedState){}fn poweroff(&mut self)->!{panic!("capture poweroff")}fn restart(&mut self)->!{panic!("capture restart")}
 fn root(&self)->&Path{&self.0}fn now(&self)->i64{1_793_523_660}fn set_clock(&mut self,_:i64){}fn set_rumble(&mut self,_:u16){}
}
struct Input(Vec<RawEvent>);impl InputSource for Input{fn poll(&mut self,_:Millis)->Vec<RawEvent>{std::mem::take(&mut self.0)}}
struct Capture{f:Frontend,g:Compositor,now:u64,out:PathBuf}
impl Capture{
 fn frame(&mut self,e:Vec<RawEvent>){self.now+=17;self.f.advance_at(&mut Input(e),self.now,1./60.);self.f.compose(&mut self.g);std::thread::sleep(std::time::Duration::from_millis(1));}
 fn wait(&mut self,n:usize){for _ in 0..n{self.frame(vec![]);}}
 fn tap(&mut self,b:Btn){self.frame(vec![RawEvent::Down(b)]);self.frame(vec![RawEvent::Up(b)]);}
 fn save(&mut self,name:&str){self.wait(60);let path=self.out.join(format!("{name}.png"));let mut e=png::Encoder::new(fs::File::create(path).unwrap(),720,480);e.set_color(png::ColorType::Rgba);e.set_depth(png::BitDepth::Eight);e.write_header().unwrap().write_image_data(&self.g.read_frame()).unwrap();}
 fn nav(&mut self,to:QuickRow){for _ in 0..24{let at=self.f.app().quick_menu().unwrap();if at==to{return;}let a=at.position(&[]);let b=to.position(&[]);self.tap(if a%4<b%4{Btn::Right}else if a%4>b%4{Btn::Left}else{Btn::Down});}panic!("missing root menu");}
 fn idle(&mut self){let end=std::time::Instant::now()+std::time::Duration::from_secs(6);while self.f.app().bluetooth.as_ref().unwrap().busy(){assert!(std::time::Instant::now()<end);self.frame(vec![]);}self.wait(45);}
}
fn main(){
 let out=PathBuf::from(std::env::args().nth(1).expect("output root"));fs::create_dir_all(&out).unwrap();
 let egl=support::Egl::new(false);
 for light in [false,true]{
  let theme=if light{"light"}else{"dark"};let dir=out.join(theme);fs::create_dir_all(&dir).unwrap();let root=dir.join("card");slot::root::ensure(&root);
  let state=SlotState{clock_set:true,appearance:if light{Appearance::Light}else{Appearance::Dark},..Default::default()};slot_store::write_slot_state(&root,&state).unwrap();
  fs::write(root.join("Config/bluetooth-enabled"),"1\n").unwrap();
  fs::write(root.join("System/bluetooth.sh"),r#"case "$1" in
supported) exit 0;;
scan) printf 'DEVICES\nDEVICE 12:34:56:78:9A:BC Headphones\nSTATE 12:34:56:78:9A:BC yes no\nDEVICE 12:34:56:78:9A:BD Wireless headphones\nSTATE 12:34:56:78:9A:BD no no\nOUTPUT speaker\n';;
connect) printf 'DEVICES\nDEVICE 12:34:56:78:9A:BC Headphones\nSTATE 12:34:56:78:9A:BC yes no\nDEVICE 12:34:56:78:9A:BD Wireless headphones\nSTATE 12:34:56:78:9A:BD yes yes\nOUTPUT 12:34:56:78:9A:BD\n';;
*) printf 'DEVICES\nDEVICE 12:34:56:78:9A:BC Headphones\nSTATE 12:34:56:78:9A:BC yes no\nOUTPUT speaker\n';;
esac
"#).unwrap();
  let mut g=Compositor::new(&egl).unwrap();g.fit((720,480));let mut f=Frontend::boot_with(Box::new(CapturePlatform(root)),Box::new(slot::audio::StubSink::draining()));f.upload_faces(&mut g);f.render(&mut g,(720,480));let mut c=Capture{f,g,now:0,out:dir};c.wait(120);c.tap(Btn::Menu);c.wait(45);c.nav(QuickRow::Bluetooth);c.tap(Btn::A);c.idle();
  assert!(c.f.app().bluetooth.as_ref().unwrap().bluetooth_available,"fixture radio not detected");eprintln!("capture rows {:?} status {:?}",c.f.app().bluetooth.as_ref().unwrap().rows(),c.f.app().bluetooth.as_ref().unwrap().status);assert_eq!(c.f.app().bluetooth.as_ref().unwrap().rows().len(),7);c.tap(Btn::Down);c.save("sound-bluetooth");
  for _ in 0..3{c.tap(Btn::Down);}assert_eq!(c.f.app().bluetooth.as_ref().unwrap().selected,4);c.tap(Btn::A);c.idle();assert_eq!(c.f.app().bluetooth.as_ref().unwrap().page,Page::Pair);c.tap(Btn::A);c.idle();c.tap(Btn::Down);c.tap(Btn::Down);c.save("bluetooth-pairing");
  assert!(c.f.app().bluetooth.as_ref().unwrap().lines().iter().any(|s|s.contains("SELECTED DEVICE Wireless headphones")));
 }
}
