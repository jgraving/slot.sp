//! Deterministic website media from the production frontend and compositor.
//! These are documentation fixtures, not captures of commercial game ROMs.
#[path = "egl_surface.rs"]
mod egl_surface;
use slot::{app::Phase, boot_motion::{BootMotion,BOOT_MS}, frontend::Frontend};
use slot_gfx::Compositor;
use slot_input::{Btn,InputSource,Millis,RawEvent};
use slot_power::{Battery,Charge,LedState,Platform};
use slot_store::{Appearance,Recent,ShellColour,SlotState};
use slot_ui::QuickRow;
use std::{fs,io::Write,path::{Path,PathBuf},process::{Child,ChildStdin,Command,Stdio}};
const W:u32=720;const H:u32=480;const FPS:u64=60;
const CLOCK:i64=1_793_523_660;
const NAMES:[&str;5]=["Emerald","Fusion","Ruby","Sapphire","WarioWare"];
struct Input(Vec<RawEvent>);impl InputSource for Input{fn poll(&mut self,_:Millis)->Vec<RawEvent>{std::mem::take(&mut self.0)}}
struct CapturePlatform{root:PathBuf,clock:i64}
impl Platform for CapturePlatform{
 fn set_backlight(&mut self,_:u8){}fn battery(&self)->Option<Battery>{Some(Battery{percent:57,charge:Charge::Charging})}fn charge(&self)->Charge{Charge::Charging}
 fn set_led(&mut self,_:LedState){}fn poweroff(&mut self)->!{panic!("capture poweroff")}fn restart(&mut self)->!{panic!("capture restart")}
 fn root(&self)->&Path{&self.root}fn now(&self)->i64{self.clock}fn set_clock(&mut self,clock:i64){self.clock=clock}fn set_rumble(&mut self,_:u16){}
}
struct Clip{child:Child,pipe:Option<ChildStdin>,frames:u32}
impl Clip{
 fn new(path:&Path)->Self{
  let mut child=Command::new("ffmpeg").args(["-nostdin","-v","error","-y","-f","rawvideo","-pix_fmt","rgba","-s","720x480","-r","60","-i","-","-an","-c:v","libx264","-threads","2","-preset","fast","-crf","23","-pix_fmt","yuv420p","-movflags","+faststart"]).arg(path).stdin(Stdio::piped()).spawn().expect("ffmpeg");
  let pipe=child.stdin.take();Self{child,pipe,frames:0}
 }
 fn frame(&mut self,pixels:&[u8]){self.pipe.as_mut().unwrap().write_all(pixels).unwrap();self.frames+=1;}
 fn finish(mut self){drop(self.pipe.take());assert!(self.child.wait().unwrap().success(),"H.264 encoding failed");assert!(self.frames>0);}
}
struct Capture{f:Frontend,g:Compositor,frame:u64,out:PathBuf,clip:Option<Clip>}
impl Capture{
 fn new(surface:&egl_surface::HeadlessSurface,root:&Path,out:&Path,light:bool,finish:ShellColour,playing:bool,kid:bool)->Self{
  fixture(root,light,finish,playing,kid);fs::create_dir_all(out).unwrap();
  let mut g=Compositor::new(surface).unwrap();g.fit((W,H));
  let mut f=Frontend::boot_with(Box::new(CapturePlatform{root:root.into(),clock:CLOCK}),Box::new(slot::audio::StubSink::draining()));f.upload_faces(&mut g);f.render(&mut g,(W,H));
  let mut c=Self{f,g,frame:0,out:out.into(),clip:None};c.wait(180);c
 }
 fn frame(&mut self,events:Vec<RawEvent>){self.frame+=1;self.f.advance_at(&mut Input(events),self.frame*1000/FPS,1.0/FPS as f32);self.f.compose(&mut self.g);if let Some(clip)=&mut self.clip{clip.frame(&self.g.read_frame());}}
 fn wait(&mut self,n:u64){for _ in 0..n{self.frame(vec![]);}}
 fn tap(&mut self,b:Btn){self.frame(vec![RawEvent::Down(b)]);self.frame(vec![RawEvent::Up(b)]);}
 fn chord(&mut self,modifier:Btn,key:Btn){self.frame(vec![RawEvent::Down(modifier),RawEvent::Down(key)]);self.frame(vec![RawEvent::Up(key),RawEvent::Up(modifier)]);}
 fn nav_play(&mut self,to:QuickRow){if self.f.app().in_game_menu().is_none(){self.tap(Btn::Menu);}for _ in 0..24{let at=self.f.app().in_game_menu().expect("game menu");if at==to{return;}let hidden=self.f.app().in_game_hidden();assert!(!hidden.contains(&to),"game menu row is hidden: {to:?}");let cols=slot_ui::menu_grid::columns(at,&hidden);let a=at.position(&hidden);let b=to.position(&hidden);self.tap(if a%cols<b%cols{Btn::Right}else if a%cols>b%cols{Btn::Left}else if a/cols<b/cols{Btn::Down}else{Btn::Up});}panic!("missing game menu {to:?}");}
 fn close_menus(&mut self){for _ in 0..6{if matches!(self.f.app().phase(),Phase::Playing{..})&&self.f.app().in_game_menu().is_none()&&self.f.app().quick_menu().is_none()&&self.f.app().shot_viewer().is_none()&&!self.f.app().game_menu_open(){return;}self.tap(Btn::B);self.wait(35);}panic!("menus did not close");}
 fn save(&mut self,name:&str){self.wait(45);self.shot(name);}
 fn shot(&self,name:&str){save_png(&self.out.join(format!("{name}.png")),&self.g.read_frame());}
 fn rec(&mut self,name:&str){assert!(self.clip.is_none());self.clip=Some(Clip::new(&self.out.join(format!("{name}.mp4"))));}
 fn cut(&mut self){if let Some(clip)=self.clip.take(){clip.finish();}}
 fn nav(&mut self,to:QuickRow){for _ in 0..45{let at=self.f.app().quick_menu().expect("quick menu");if at==to{return;}assert_eq!(at.page(),to.page(),"cannot navigate {at:?} to {to:?}");let key=if at.page()==QuickRow::MAIN{let a=at.position(&[]);let b=to.position(&[]);if a%4<b%4{Btn::Right}else if a%4>b%4{Btn::Left}else{Btn::Down}}else{Btn::Down};self.tap(key);}panic!("missing menu row {to:?}");}
 fn hero(&mut self){
  self.shot("hero");self.rec("hero-native");self.wait(48);
  // Record the real launch gesture and every frame of the production slot
  // transition. The earlier website loop never pressed A, so it could only
  // show the carousel and logo, not the cartridge entering its physical bay.
  let insert_at=self.clip.as_ref().unwrap().frames;
  self.tap(Btn::A);assert!(matches!(self.f.app().phase(),Phase::Inserting{..}),"hero did not begin cartridge insertion");
  let mut inserting=0;let mut visible_game_frames=0;
  for frame in 0..35 {
   self.frame(vec![]);
   if matches!(self.f.app().phase(),Phase::Inserting{..}) {inserting+=1;}
   if self.f.app().game_visible() {visible_game_frames+=1;}
   match frame {7=>self.shot("hero-insert-early"),17=>self.shot("hero-insert-middle"),32=>{assert!(self.f.app().seat()>0.95,"hero cartridge never seated");self.shot("hero-insert-seated");},_=>{}}
  }
  assert!(inserting>=24,"hero skipped the visible insertion");
  // The fixture uses a synthetic game, not commercial gameplay. Keep the real
  // insertion, then omit the test screen while the app launches and exits.
  let clip=self.clip.take().expect("hero recording");self.wait(96-35);
  assert!(matches!(self.f.app().phase(),Phase::Playing{..}),"hero game did not launch");self.shot("hero-playing");
  // The hold-to-exit gesture raises the slot, then ejects the same cartridge.
  let exit_at=clip.frames;
  self.frame(vec![RawEvent::Down(Btn::Menu)]);self.wait(65);
  assert!(matches!(self.f.app().phase(),Phase::Ejecting{..}),"hero did not enter ejection");
  self.frame(vec![RawEvent::Up(Btn::Menu)]);
  for _ in 0..120 {if self.f.app().screen_power()<=0.0 {break;}self.frame(vec![]);}
  assert!(self.f.app().screen_power()<=0.0,"hero test screen did not close");
  assert!(matches!(self.f.app().phase(),Phase::Ejecting{..}),"hero skipped ejection while hiding the test screen");
  self.shot("hero-ejecting");self.clip=Some(clip);
  for _ in 0..90 {self.frame(vec![]);if self.f.app().game_visible(){visible_game_frames+=1;}}
  assert!(matches!(self.f.app().phase(),Phase::Shelf),"hero did not return to carousel");self.wait(24);
  // Launch is the first demonstration, followed by browsing the collection.
  for _ in 0..4{self.tap(Btn::Right);self.wait(30);}for _ in 0..4{self.tap(Btn::Left);self.wait(30);}self.wait(24);
  assert_eq!(visible_game_frames,0,"hero included the synthetic game screen");
  let frames=self.clip.as_ref().unwrap().frames;
  fs::write(self.out.join("hero-timeline.json"),format!("{{\"fps\":60,\"frames\":{frames},\"insert_frame\":{insert_at},\"visible_insertion_frames\":{inserting},\"exit_frame\":{exit_at},\"visible_game_frames\":{visible_game_frames},\"omitted_test_pattern\":true,\"assertions\":[\"inserting\",\"seated\",\"playing\",\"ejecting\",\"shelf\"]}}\n")).unwrap();self.cut();
  // Native boot composition retains the physical slot's occlusion and current
  // spring timing. Its completed launcher pose becomes the return to the loop.
  self.boot("hero-boot");
 }
 fn boot(&mut self,name:&str){
  let motion=BootMotion::upload(&mut self.g).unwrap();let mut clip=Clip::new(&self.out.join(format!("{name}.mp4")));
  for frame in 0..((BOOT_MS*FPS).div_ceil(1000)+52){self.f.compose_boot_frame(&mut self.g,&motion,frame.saturating_sub(if name=="hero-boot"{18}else{0})*1000/FPS);let pixels=self.g.read_frame();if frame==0{save_png(&self.out.join(format!("{name}.png")),&pixels);}clip.frame(&pixels);}
  clip.finish();motion.release(&mut self.g);
 }
 fn shelf_specials(&mut self){
  self.save("approved");self.tap(Btn::Start);self.wait(100);assert!(self.f.app().core_picker().is_some());self.shot("core-picker");self.tap(Btn::Start);self.wait(75);
  for target in ["Favorites","Recents"] {for _ in 0..8{if self.f.app().shelf_platform_name()==Some(target){break;}self.tap(Btn::R1);self.wait(30);}assert_eq!(self.f.app().shelf_platform_name(),Some(target));self.save(if target=="Favorites"{"favorites"}else{"recents"});}
  self.tap(Btn::Menu);self.nav(QuickRow::Controls);self.tap(Btn::A);self.nav(QuickRow::Reference);self.tap(Btn::A);for _ in 0..3{self.tap(Btn::Right);}self.save("buttons-shortcuts");
 }
 fn game_specials(&mut self){
  self.close_menus();
  for _ in 0..4 {self.chord(Btn::Select,Btn::R1);self.wait(35);}
  for _ in 0..6 {self.chord(Btn::Menu,Btn::X);self.wait(25);}
  self.wait(90);
  let root=self.f.app().root().unwrap().to_owned();let deadline=std::time::Instant::now()+std::time::Duration::from_secs(12);
  while slot_store::ShotDir::new(&root,slot_store::Platform::Gba,"Emerald").list().unwrap().len()<6 {assert!(std::time::Instant::now()<deadline,"fixture screenshots were not written");self.frame(vec![]);std::thread::sleep(std::time::Duration::from_millis(10));}
  assert!(!slot_store::StateRing::new(&root,slot_store::Platform::Gba,slot_store::Core::Mgba,"Emerald").list().unwrap().is_empty(),"fixture save state was not written");
  self.nav_play(QuickRow::States);self.tap(Btn::A);self.wait(100);assert!(matches!(self.f.app().phase(),Phase::Polaroids{..}));self.shot("save-states");self.close_menus();
  self.nav_play(QuickRow::Screenshots);self.tap(Btn::A);self.wait(60);assert!(self.f.app().shot_viewer().is_some());let expected=self.f.app().shot_viewer().unwrap().thumb_order(self.f.app().canvas()).len();let deadline=std::time::Instant::now()+std::time::Duration::from_secs(12);while self.f.gallery_textures()<expected{assert!(std::time::Instant::now()<deadline,"gallery thumbnails did not finish");self.frame(vec![]);std::thread::sleep(std::time::Duration::from_millis(10));}self.wait(10);self.shot("screenshots");
  self.tap(Btn::Select);self.tap(Btn::A);self.tap(Btn::Right);self.tap(Btn::A);self.save("screenshots-select");self.tap(Btn::B);
  self.tap(Btn::A);let deadline=std::time::Instant::now()+std::time::Duration::from_secs(12);
  while !self.f.app().shot_viewer().unwrap().has_face(){assert!(std::time::Instant::now()<deadline,"full screenshot did not load");self.frame(vec![]);std::thread::sleep(std::time::Duration::from_millis(10));}
  self.save("screenshots-full");self.tap(Btn::A);self.save("screenshots-zoom");self.close_menus();
  self.chord(Btn::Select,Btn::Menu);self.wait(90);assert!(self.f.app().game_menu_open());self.shot("link-host");self.tap(Btn::Right);self.save("link-join");self.tap(Btn::B);
 }
 fn kid_warning(&mut self){if matches!(self.f.app().phase(),Phase::Shelf){self.tap(Btn::A);self.wait(180);}assert!(matches!(self.f.app().phase(),Phase::Playing{..}),"kid game did not open");self.wait(60);self.frame+=75*60*FPS;self.f.advance_at(&mut Input(vec![]),self.frame*1000/FPS,75.0*60.0);self.f.compose(&mut self.g);assert!(self.f.app().kid_menu.as_ref().is_some_and(|m|m.time_warning()),"time milestone did not pause for a reminder");self.save("kid-warning");}
 fn sound(&mut self){
  // Desktop recordings explicitly model a Bluetooth-capable handheld. No
  // hardware commands or pairings are performed by these synthetic devices.
  fs::write(self.f.app().root().unwrap().join("Config/bluetooth-enabled"), "").unwrap();
  self.tap(Btn::Menu);self.nav(QuickRow::Bluetooth);self.tap(Btn::A);self.wait(45);
  let menu=self.f.app_mut().bluetooth.as_mut().unwrap();
  menu.bluetooth_available=true;
  menu.devices=vec![slot::bluetooth::Headphones{address:"00:11:22:33:44:55".into(),name:"Headphones".into(),paired:true,connected:false}];
  menu.selected=3;menu.face=None;
  assert!(menu.rows().iter().any(|r|r.starts_with("BLUETOOTH ")));
  assert!(menu.rows().iter().any(|r|r=="Pair audio device"));
  self.save("sound");
  let menu=self.f.app_mut().bluetooth.as_mut().unwrap();menu.selected=menu.rows().len()-1;menu.face=None;
  self.save("sound-options");
  let menu=self.f.app_mut().bluetooth.as_mut().unwrap();menu.page=slot::bluetooth::Page::Pair;menu.selected=0;menu.face=None;
  self.save("sound-pair");
 }
 fn settings(&mut self,light:bool){
  self.save("carousel");self.boot("boot");self.tap(Btn::Menu);self.save("settings");
  self.nav(QuickRow::General);self.rec("settings");self.tap(Btn::A);self.wait(65);self.nav(QuickRow::Theme);self.wait(45);self.shot("general");
  self.nav(QuickRow::CarouselSounds);self.tap(Btn::A);self.wait(45);self.tap(Btn::A);self.wait(45);self.tap(Btn::B);self.wait(50);self.cut();
  self.nav(QuickRow::Screen);self.tap(Btn::A);self.save("screen");self.tap(Btn::B);self.wait(35);
  self.nav(QuickRow::Bluetooth);self.tap(Btn::A);self.save("sound");self.tap(Btn::B);self.wait(35);
  self.nav(QuickRow::Power);self.tap(Btn::A);self.save("power");self.tap(Btn::B);self.wait(35);
  self.nav(QuickRow::Controls);self.tap(Btn::A);self.save("controls");self.tap(Btn::B);self.wait(35);
  self.nav(QuickRow::About);self.tap(Btn::A);self.save("about");self.tap(Btn::B);self.wait(35);
  self.nav(QuickRow::Kids);self.tap(Btn::A);self.save("parent-settings");self.tap(Btn::Down);self.rec("parent-code");self.tap(Btn::A);self.wait(30);
  for b in [Btn::L1,Btn::R1,Btn::L2]{self.tap(b);self.wait(16);}self.shot("parent-code");self.wait(20);self.tap(Btn::R2);self.wait(30);for b in [Btn::L1,Btn::R1,Btn::L2,Btn::R2]{self.tap(b);self.wait(12);}self.wait(45);self.cut();
  self.tap(Btn::Down);self.tap(Btn::Down);self.tap(Btn::A);self.save("parent-time");self.tap(Btn::B);self.tap(Btn::B);self.wait(45);
  // Use the production feedback component over a real settings scene, matching
  // the fixture's 57% header battery and the actual transient notification.
  let face=slot_ui::settings_notice_face_on("Charging: 57%",self.f.app().canvas(),light);let tex=self.g.create_texture(face.w,face.h,&face.rgba);
  let mut notice=slot_ui::SettingsNotice::default();notice.observe(Some((1,"Charging: 57%".into())),0);let mut clip=Clip::new(&self.out.join("charging.mp4"));
  for frame in 0..180{self.frame(vec![]);let mut draws=vec![];notice.draw_on(self.f.app().canvas(),Some((tex,face.w,face.h)),frame*1000/FPS,light,&mut draws);self.g.draw_list(&draws);let pixels=self.g.read_frame();if frame==24{save_png(&self.out.join("charging.png"),&pixels);}clip.frame(&pixels);}clip.finish();
 }
}
fn save_png(path:&Path,pixels:&[u8]){let mut e=png::Encoder::new(fs::File::create(path).unwrap(),W,H);e.set_color(png::ColorType::Rgba);e.set_depth(png::BitDepth::Eight);e.write_header().unwrap().write_image_data(pixels).unwrap();}
fn fixture(root:&Path,light:bool,finish:ShellColour,playing:bool,kid:bool){
 if root.exists(){fs::remove_dir_all(root).unwrap();}slot::root::ensure(root);let repo=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
 for name in NAMES{let mut bytes=vec![0;256];let title=name.to_uppercase();bytes[0xa0..0xa0+title.len()].copy_from_slice(title.as_bytes());fs::write(root.join("Games/GBA").join(format!("{name}.gba")),bytes).unwrap();let art=repo.join("previews/artwork").join(format!("{}.png",name.replace(' ',"-")));fs::copy(art,root.join("Labels/GBA").join(format!("{name}.png"))).unwrap();}
 fs::write(root.join(slot_store::CART_SHELL_FILE),"Emerald = auto 239c47 clear\nRuby = auto b5253b clear\nSapphire = auto 2854b5 clear\n").unwrap();
 let state=SlotState{clock_set:true,cart:playing.then(||"Emerald".into()),appearance:if light{Appearance::Light}else{Appearance::Dark},shell_colour:finish,show_recents:true,recents:NAMES[..3].iter().map(|name|Recent{platform:slot_store::Platform::Gba,stem:(*name).into()}).collect(),shelf:Some("gba".into()),shelf_cart:Some(Recent{platform:slot_store::Platform::Gba,stem:"Emerald".into()}),favorites:NAMES.into_iter().map(|name|Recent{platform:slot_store::Platform::Gba,stem:name.into()}).collect(),..Default::default()};slot_store::write_slot_state(root,&state).unwrap();
 let mut policy=slot::kid_policy::Policy::default();policy.minutes=90;policy.volume_limit=50;policy.approved=NAMES[..3].iter().map(|s|(slot_store::Platform::Gba,(*s).into())).collect();if kid{policy.begin(CLOCK).unwrap();let mut gate=slot::parent_code::Gate::new(slot::kid_policy::Policy::directory(root).join("code"));gate.set(&[8,9,10,11]).unwrap();}policy.save(root).unwrap();
}
fn main(){
 let out=PathBuf::from(std::env::args().nth(1).expect("capture output root"));fs::create_dir_all(&out).unwrap();
 let mode=std::env::args().nth(2).unwrap_or_else(||"all".into());assert!(["all","docs","hero","special","hero-gallery","audio"].contains(&mode.as_str()),"capture mode");
 let surface=egl_surface::HeadlessSurface::new().unwrap();
 for light in [false,true]{let theme=if light{"light"}else{"dark"};
  if mode=="audio" {let mut c=Capture::new(&surface,&out.join(format!("cards/{theme}-audio")),&out.join(format!("raw/current/{theme}")),light,ShellColour::Platinum,false,false);c.sound();drop(c);}
  if mode=="all"||mode=="docs"{let mut c=Capture::new(&surface,&out.join(format!("cards/{theme}-docs")),&out.join(format!("raw/current/{theme}")),light,ShellColour::Platinum,false,false);c.settings(light);drop(c);
   let mut c=Capture::new(&surface,&out.join(format!("cards/{theme}-kid")),&out.join(format!("raw/current/{theme}")),light,ShellColour::Platinum,false,true);c.tap(Btn::Menu);c.save("kid-menu");drop(c);
   let mut c=Capture::new(&surface,&out.join(format!("cards/{theme}-game")),&out.join(format!("raw/current/{theme}")),light,ShellColour::Platinum,true,false);assert!(matches!(c.f.app().phase(),Phase::Playing{..}));c.tap(Btn::Menu);c.save("in-game-menu");c.nav_play(QuickRow::Settings);c.tap(Btn::A);c.save("in-game-settings");drop(c);
  }
  if mode=="all"||mode=="special" {let mut c=Capture::new(&surface,&out.join(format!("cards/{theme}-special-shelf")),&out.join(format!("raw/current/{theme}")),light,ShellColour::Platinum,false,false);c.shelf_specials();drop(c);let mut c=Capture::new(&surface,&out.join(format!("cards/{theme}-special-game")),&out.join(format!("raw/current/{theme}")),light,ShellColour::Platinum,true,false);c.game_specials();drop(c);let mut c=Capture::new(&surface,&out.join(format!("cards/{theme}-kid-warning")),&out.join(format!("raw/current/{theme}")),light,ShellColour::Platinum,true,true);c.kid_warning();drop(c);}
  if mode=="hero-gallery" {let mut c=Capture::new(&surface,&out.join(format!("cards/{theme}-gallery")),&out.join(format!("raw/current/{theme}")),light,ShellColour::Platinum,true,false);c.game_specials();drop(c);}
  if mode=="all"||mode=="hero"||mode=="hero-gallery"{for finish in ShellColour::FINISHES{let key=finish.key().replace(' ',"_");let mut c=Capture::new(&surface,&out.join(format!("cards/{theme}-{key}")),&out.join(format!("raw/homepage/{theme}-{key}")),light,finish,false,false);c.hero();}}
 }
 fs::write(out.join(format!("capture-{mode}-complete.txt")),"Native website media capture completed\n").unwrap();
}
