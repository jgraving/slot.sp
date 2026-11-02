//! Short, joinable native UI actions for the homepage. No game core is run.
#[path="egl_surface.rs"] mod egl_surface;
use slot::{app::Phase, boot_motion::{BootMotion, BOOT_MS}, frontend::Frontend};
use slot_gfx::Compositor;
use slot_input::{Action, Btn};
use slot_power::{Battery, Charge, LedState, Platform};
use slot_store::{Recent, ShellColour, SlotState};
use std::{fs, io::Write, path::{Path, PathBuf}, process::{Child, ChildStdin, Command, Stdio}};
const NAMES: [&str; 5] = ["Emerald", "Fusion", "Ruby", "Sapphire", "WarioWare"];
struct Device(PathBuf);
impl Platform for Device {
    fn set_backlight(&mut self,_:u8){} fn battery(&self)->Option<Battery>{Some(Battery{percent:57,charge:Charge::Discharging})}
    fn charge(&self)->Charge{Charge::Discharging} fn set_led(&mut self,_:LedState){}
    fn poweroff(&mut self)->!{panic!("capture poweroff")} fn restart(&mut self)->!{panic!("capture restart")}
    fn root(&self)->&Path{&self.0} fn now(&self)->i64{1_793_523_660} fn set_clock(&mut self,_:i64){} fn set_rumble(&mut self,_:u16){}
}
struct Film { child: Child, pipe: Option<ChildStdin>, frames: u32, entries: Vec<String> }
impl Film {
    fn new(path:&Path)->Self {
        let mut child=Command::new("ffmpeg").args(["-nostdin","-v","error","-y","-f","rawvideo","-pix_fmt","rgba","-s","720x480","-r","60","-i","-","-an","-c:v","libx264","-threads","2","-preset","slow","-crf","20","-g","15","-bf","0","-pix_fmt","yuv420p","-movflags","+faststart"]).arg(path).stdin(Stdio::piped()).spawn().unwrap();
        let pipe=child.stdin.take();Self{child,pipe,frames:0,entries:vec![]}
    }
    fn push(&mut self,g:&Compositor){self.pipe.as_mut().unwrap().write_all(&g.read_frame()).unwrap();self.frames+=1;}
    fn mark(&mut self,name:&str,start:u32){self.entries.push(format!("\"{name}\":[{start},{}]",self.frames-1));}
    fn finish(mut self,out:&Path){drop(self.pipe.take());assert!(self.child.wait().unwrap().success());fs::write(out,format!("{{\"fps\":60,\"frames\":{},\"segments\":{{{}}}}}\n",self.frames,self.entries.join(","))).unwrap();}
}
fn fixture(root:&Path,finish:ShellColour){
    slot::root::ensure(root);
    let repo=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    for name in NAMES {let mut bytes=vec![0;256];let title=name.to_uppercase();bytes[0xa0..0xa0+title.len()].copy_from_slice(title.as_bytes());fs::write(root.join("Games/GBA").join(format!("{name}.gba")),bytes).unwrap();fs::copy(repo.join("previews/artwork").join(format!("{name}.png")),root.join("Labels/GBA").join(format!("{name}.png"))).unwrap();}
    fs::write(root.join(slot_store::CART_SHELL_FILE),"Emerald = auto 239c47 clear\nRuby = auto b5253b clear\nSapphire = auto 2854b5 clear\n").unwrap();
    let state=SlotState{clock_set:true,shell_colour:finish,shelf:Some("gba".into()),shelf_cart:Some(Recent{platform:slot_store::Platform::Gba,stem:"Emerald".into()}),..Default::default()};
    slot_store::write_slot_state(root,&state).unwrap();
}
fn step(f:&mut Frontend,g:&mut Compositor,action:Option<Action>){f.preview_step(action);f.compose(g);}
fn wait(f:&mut Frontend,g:&mut Compositor,frames:u32){for _ in 0..frames{step(f,g,None);}}
fn scene(f:&mut Frontend,g:&mut Compositor,film:&mut Film,name:&str,action:Action,frames:u32){
    let start=film.frames;film.push(g);step(f,g,Some(action));film.push(g);
    // Release direction holds before they can repeat; preserve native spring timing.
    step(f,g,Some(Action::GbaUp(Btn::Right)));film.push(g);step(f,g,Some(Action::GbaUp(Btn::Left)));film.push(g);
    for _ in 0..frames{step(f,g,None);film.push(g);}film.mark(name,start);
}
fn main(){
    let out=PathBuf::from(std::env::args().nth(1).expect("output directory"));fs::create_dir_all(&out).unwrap();
    let surface=egl_surface::HeadlessSurface::new().unwrap();
    for finish in ShellColour::FINISHES {
        let name=finish.key().replace(' ',"_");let root=out.join(format!("card-{name}"));fixture(&root,finish);
        let mut g=Compositor::new(&surface).unwrap();g.fit((720,480));
        let mut f=Frontend::boot_with(Box::new(Device(root)),Box::new(slot::audio::StubSink::draining()));f.upload_faces(&mut g);f.render(&mut g,(720,480));wait(&mut f,&mut g,240);
        let mut film=Film::new(&out.join(format!("{name}.mp4")));
        let boot=BootMotion::upload(&mut g).unwrap();let start=film.frames;
        for frame in 0..(BOOT_MS*60/1000+16){f.compose_boot_frame(&mut g,&boot,frame*1000/60);film.push(&g);}
        film.mark("boot",start);boot.release(&mut g);f.compose(&mut g);
        for (index,cart) in NAMES.iter().enumerate(){
            assert_eq!(f.app().selected_stem(),Some(*cart));assert!(matches!(f.app().phase(),Phase::Shelf));
            scene(&mut f,&mut g,&mut film,&format!("insert-{index}"),Action::Insert,36);
            assert!(f.app().seat()>0.999);assert!(!f.app().game_visible());
            scene(&mut f,&mut g,&mut film,&format!("eject-{index}"),Action::Eject,78);
            assert!(matches!(f.app().phase(),Phase::Shelf));
            scene(&mut f,&mut g,&mut film,&format!("right-{index}"),Action::ShelfRight,42);
            assert_eq!(f.app().selected_stem(),Some(NAMES[(index+1)%NAMES.len()]));
            scene(&mut f,&mut g,&mut film,&format!("left-{}",(index+1)%NAMES.len()),Action::ShelfLeft,42);
            assert_eq!(f.app().selected_stem(),Some(*cart));
            step(&mut f,&mut g,Some(Action::ShelfRight));step(&mut f,&mut g,Some(Action::GbaUp(Btn::Right)));wait(&mut f,&mut g,90);
        }
        film.finish(&out.join(format!("{name}.json")));
    }
}
