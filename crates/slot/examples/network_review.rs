//! Walk real settings pages using the production frontend and compositor.
#[cfg(target_os="linux")]
#[path="../../slot-ui/examples/support/mod.rs"] mod support;
#[cfg(target_os="linux")]
fn main(){
    use slot::{frontend::Frontend,app::Phase};
    use slot_input::{InputSource,RawEvent,Btn,Millis};
    use slot_gfx::Compositor;
    use slot_ui::QuickRow;
    struct Input(Vec<RawEvent>);impl InputSource for Input {fn poll(&mut self,_:Millis)->Vec<RawEvent>{std::mem::take(&mut self.0)}}
    struct Review {f:Frontend,g:Compositor,now:u64,dir:std::path::PathBuf,width:u32}
    impl Review {
        fn frame(&mut self,events:Vec<RawEvent>){self.now+=17;self.f.advance_at(&mut Input(events),self.now,1.0/60.0);self.f.compose(&mut self.g);}
        fn tap(&mut self,b:Btn){self.frame(vec![RawEvent::Down(b)]);self.frame(vec![RawEvent::Up(b)]);}
        fn settle(&mut self){for _ in 0..45 {self.frame(vec![]);}}
        fn save(&mut self,name:&str){self.settle();self.capture(name);}
        fn capture(&mut self,name:&str){let mut e=png::Encoder::new(std::fs::File::create(self.dir.join(format!("{name}.png"))).unwrap(),self.width,480);e.set_color(png::ColorType::Rgba);e.set_depth(png::BitDepth::Eight);e.write_header().unwrap().write_image_data(&self.g.read_frame()).unwrap();}
        fn nav(&mut self,to:QuickRow){for _ in 0..32 {let at=self.f.app().quick_menu().unwrap();if at==to{return;}assert_eq!(at.page(),to.page());let b=if at.page()==QuickRow::MAIN {let a=at.position(&[]);let b=to.position(&[]);if a%4<b%4 {Btn::Right}else if a%4>b%4 {Btn::Left}else {Btn::Down}}else {Btn::Down};self.tap(b);}panic!("cannot reach {to:?}");}
        fn home(&mut self){for _ in 0..12 {if matches!(self.f.app().phase(),Phase::Shelf)&&self.f.app().kid_menu.is_none()&&self.f.app().bluetooth.is_none()&&self.f.app().display_schedule.is_none()&&!self.f.app().network.as_ref().is_some_and(|m|m.visible){self.tap(Btn::Menu);self.settle();return;}if self.f.app().kid_menu.as_ref().is_some_and(|m|m.entering_code()){self.tap(Btn::Menu);}else{self.tap(Btn::B);}}panic!("cannot close settings");}
    }
    let dir=std::path::PathBuf::from(std::env::args().nth(1).unwrap());std::fs::create_dir_all(&dir).unwrap();
    let width=std::env::args().nth(3).and_then(|s|s.parse().ok()).unwrap_or(640);
    let root=dir.join("card");slot::root::ensure(&root);
    let mut state=slot_store::SlotState::default();state.clock_set=true;state.clock_12h=true;state.appearance=if std::env::args().nth(4).as_deref()==Some("light"){slot_store::Appearance::Light}else{slot_store::Appearance::Dark};slot_store::write_slot_state(&root,&state).unwrap();
    for (i,name) in ["Aurora","Borealis","Celeste"].into_iter().enumerate(){let mut rom=vec![0;256];rom[0xa0..0xa6].copy_from_slice(b"REVIEW");rom[0]=i as u8;std::fs::write(root.join("Games/GBA").join(format!("{name}.gba")),rom).unwrap();}
    let surface=support::Egl::new(std::env::args().nth(2).as_deref()!=Some("gl"));let mut g=Compositor::new(&surface).unwrap();g.fit((width,480));
    let mut f=Frontend::boot(Box::new(slot_power::SimPlatform::at(root)));f.upload_faces(&mut g);f.render(&mut g,(width,480));
    let mut a=Review{f,g,now:0,dir,width};a.save("carousel");a.home();a.save("settings");
    a.nav(QuickRow::Wifi);a.save("wifi-tile");a.tap(Btn::A);a.save("wifi-off");
    a.tap(Btn::A);a.save("wifi-on");a.tap(Btn::Down);a.tap(Btn::Down);a.tap(Btn::A);a.save("network-keyboard");
    a.tap(Btn::Menu);a.settle();a.tap(Btn::B);a.nav(QuickRow::Game);a.tap(Btn::A);a.nav(QuickRow::Achievements);a.save("gameplay-achievements");a.tap(Btn::A);a.save("achievements-off");
    for _ in 0..5 {a.tap(Btn::Down);}a.tap(Btn::A);a.save("credential-storage");a.tap(Btn::Down);a.tap(Btn::A);a.save("vault-keyboard");
}
#[cfg(not(target_os="linux"))] fn main(){}
