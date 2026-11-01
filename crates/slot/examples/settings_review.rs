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
        fn home(&mut self){for _ in 0..12 {if matches!(self.f.app().phase(),Phase::Shelf)&&self.f.app().kid_menu.is_none()&&self.f.app().bluetooth.is_none()&&self.f.app().display_schedule.is_none(){self.tap(Btn::Menu);self.settle();return;}if self.f.app().kid_menu.as_ref().is_some_and(|m|m.entering_code()){self.tap(Btn::Menu);}else{self.tap(Btn::B);}}panic!("cannot close settings");}
    }
    let dir=std::path::PathBuf::from(std::env::args().nth(1).unwrap());std::fs::create_dir_all(&dir).unwrap();
    let width=std::env::args().nth(3).and_then(|s|s.parse().ok()).unwrap_or(640);
    let root=dir.join("card");slot::root::ensure(&root);
    let mut state=slot_store::SlotState::default();state.clock_set=true;state.clock_12h=true;slot_store::write_slot_state(&root,&state).unwrap();
    for (i,name) in ["Aurora","Borealis","Celeste"].into_iter().enumerate(){let mut rom=vec![0;256];rom[0xa0..0xa6].copy_from_slice(b"REVIEW");rom[0]=i as u8;std::fs::write(root.join("Games/GBA").join(format!("{name}.gba")),rom).unwrap();}
    let surface=support::Egl::new(std::env::args().nth(2).as_deref()!=Some("gl"));let mut g=Compositor::new(&surface).unwrap();g.fit((width,480));
    let mut f=Frontend::boot(Box::new(slot_power::SimPlatform::at(root)));f.upload_faces(&mut g);f.render(&mut g,(width,480));
    let mut a=Review{f,g,now:0,dir,width};a.save("carousel");a.home();a.save("settings");
    for root in QuickRow::MAIN {
        a.nav(root);a.save(&format!("root-{root:?}"));a.tap(Btn::A);a.save(&format!("page-{root:?}"));
        if let Some(row)=a.f.app().quick_menu().filter(|r|r.page()!=QuickRow::MAIN) {
            let rows=row.shown(&a.f.app().hidden_rows());
            for target in rows {a.nav(target);a.save(&format!("row-{target:?}"));
                if matches!(target,QuickRow::ScreenGba|QuickRow::ScreenGbc|QuickRow::ScreenGb|QuickRow::ControlsShelf|QuickRow::Shortcuts|QuickRow::ControlsInGame){a.tap(Btn::A);a.save(&format!("sub-{target:?}"));for _ in 0..12 {a.tap(Btn::Down);}a.save(&format!("sub-end-{target:?}"));a.tap(Btn::B);}
                if target==QuickRow::DisplaySchedule {a.tap(Btn::A);a.save("night-mode");a.tap(Btn::Down);a.tap(Btn::A);a.save("night-time-editor");a.tap(Btn::B);a.tap(Btn::B);}
                if target==QuickRow::DateTime {a.tap(Btn::A);a.save("clock");a.tap(Btn::B);}
                if target==QuickRow::Reference {a.tap(Btn::A);a.save("reference");for _ in 0..9 {a.tap(Btn::Right);}a.save("reference-end");a.tap(Btn::B);}
            }
        }
        if root==QuickRow::Kids {a.tap(Btn::Down);a.save("parent-code-row");a.tap(Btn::A);a.save("parent-code-editor");}
        a.home();
    }
    // Exercise the state that previously collapsed the parent rows and clipped
    // the meter: a real approval, code setup, then activation in the open menu.
    a.tap(Btn::B);a.tap(Btn::X);a.save("approved-carousel");a.tap(Btn::Menu);a.nav(QuickRow::Kids);a.tap(Btn::A);
    a.tap(Btn::Down);a.tap(Btn::A);
    a.save("parent-code-choose");
    for b in [Btn::L1,Btn::R1,Btn::L2]{a.tap(b);}
    a.save("parent-code-shoulders");a.tap(Btn::R2);
    a.save("parent-code-confirm");
    for b in [Btn::L1,Btn::R1,Btn::L2,Btn::R2]{a.tap(b);}
    a.tap(Btn::Up);a.save("parent-before-enable");a.tap(Btn::A);a.save("parent-enabled");
    assert!(a.f.app().kid_mode());assert_eq!(a.f.app().kid_menu.as_ref().unwrap().scroll_info(),Some((0,5,0,5)));
    a.tap(Btn::VolUp);a.save("parent-volume-overlay");a.tap(Btn::B);
    for i in 0..30 {a.frame(vec![]);if [0,2,5,9,15,29].contains(&i){a.capture(&format!("parent-close-{i:02}"));}}
    assert!(matches!(a.f.app().phase(),Phase::Shelf));a.tap(Btn::Menu);a.save("kid-settings-grid");
    a.tap(Btn::A);a.save("kid-sound");a.tap(Btn::VolDown);a.save("kid-sound-overlay");a.tap(Btn::B);
    a.tap(Btn::Right);a.tap(Btn::A);a.save("kid-screen");a.tap(Btn::B);a.tap(Btn::Down);a.tap(Btn::Left);a.tap(Btn::A);a.save("kid-shortcuts");a.tap(Btn::B);
    a.tap(Btn::B);
    for i in 0..45 {a.frame(vec![]);if [0,2,5,9,15,24,44].contains(&i){a.capture(&format!("return-{i:02}"));}}
}
#[cfg(not(target_os="linux"))] fn main(){}
