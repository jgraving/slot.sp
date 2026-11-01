//! Native review of live theme changes, submenu entry, and switch/keycap edges.
#[cfg(target_os="linux")]
#[path="../../slot-ui/examples/support/mod.rs"] mod support;
#[cfg(target_os="linux")]
fn main(){
    use slot::{frontend::Frontend,app::Phase};
    use slot_input::{InputSource,RawEvent,Btn,Millis};
    use slot_gfx::{Compositor,Draw};use slot_ui::QuickRow;
    struct Input(Vec<RawEvent>);impl InputSource for Input{fn poll(&mut self,_:Millis)->Vec<RawEvent>{std::mem::take(&mut self.0)}}
    struct Review{f:Frontend,g:Compositor,now:u64,dir:std::path::PathBuf,width:u32}
    impl Review{
        fn frame(&mut self,events:Vec<RawEvent>){self.now+=17;self.f.advance_at(&mut Input(events),self.now,1.0/60.0);self.f.compose(&mut self.g);}
        fn tap(&mut self,b:Btn){self.frame(vec![RawEvent::Down(b)]);self.frame(vec![RawEvent::Up(b)]);}
        fn settle(&mut self){for _ in 0..40{self.frame(vec![]);}}
        fn capture(&mut self,name:&str){let mut e=png::Encoder::new(std::fs::File::create(self.dir.join(format!("{name}.png"))).unwrap(),self.width,480);e.set_color(png::ColorType::Rgba);e.set_depth(png::BitDepth::Eight);e.write_header().unwrap().write_image_data(&self.g.read_frame()).unwrap();}
        fn save(&mut self,name:&str){self.settle();self.capture(name);}
        fn exit_review(&mut self,name:&str){
            self.tap(Btn::B);let mut moved=false;
            for i in 0..42 {
                self.frame(vec![]);if [0,1,2,5,9,20,41].contains(&i){self.capture(&format!("{name}-exit-{i:02}"));}
                let mut out=vec![];self.f.app().draw(&mut out);
                for d in out {if let Draw::RoundRect{x,y,w,h,colour,..}=d {if colour==slot_ui::expressive::SELECTED&&(h-slot_ui::expressive::ROW_H).abs()<0.1 {
                    assert!(x>=55.9&&x+w<=self.width as f32-55.9&&y>=55.9&&y+h<=366.1,"outgoing focus escaped card");
                    moved|=x>85.0;
                }}}
            }
            assert!(moved,"{name} selector stayed still during exit");
        }
        fn nav(&mut self,to:QuickRow){for _ in 0..40{let at=self.f.app().quick_menu().unwrap();if at==to{return;}assert_eq!(at.page(),to.page());let b=if at.page()==QuickRow::MAIN{let a=at.position(&[]);let b=to.position(&[]);if a%4<b%4{Btn::Right}else if a%4>b%4{Btn::Left}else{Btn::Down}}else{Btn::Down};self.tap(b);}panic!("missing {to:?}");}
    }
    let dir=std::path::PathBuf::from(std::env::args().nth(1).unwrap());std::fs::create_dir_all(&dir).unwrap();
    let width:u32=std::env::args().nth(3).and_then(|s|s.parse().ok()).unwrap_or(640);
    let root=dir.join("card");slot::root::ensure(&root);slot_store::write_slot_state(&root,&slot_store::SlotState{clock_set:true,..Default::default()}).unwrap();
    for name in ["Aurora","Borealis","Celeste"]{std::fs::write(root.join("Games/GBA").join(format!("{name}.gba")),[0;256]).unwrap();}
    let surface=support::Egl::new(std::env::args().nth(2).as_deref()!=Some("gl"));let mut g=Compositor::new(&surface).unwrap();g.fit((width,480));
    // UI palette changes must never recolour ordinary artwork, and a theme
    // round trip must restore the original cached glyph pixels exactly.
    let pixels=[246,244,239,255];let ui=g.create_ui_texture(1,1,&pixels);let art=g.create_texture(1,1,&pixels);
    for light in [true,false,true,false] {g.set_light_ui(light);g.begin_frame();g.draw_list(&[Draw::Tex{x:10.0,y:10.0,w:20.0,h:20.0,tex:ui,alpha:1.0},Draw::Tex{x:40.0,y:10.0,w:20.0,h:20.0,tex:art,alpha:1.0}]);let frame=g.read_frame();let at=|x:u32|((15*width+x)*4)as usize;assert_eq!(&frame[at(45)..at(45)+3],&pixels[..3]);assert_eq!(&frame[at(15)..at(15)+3],if light{&slot_gfx::ui_palette::INK[..]}else{&pixels[..3]});}
    // Background textures have their own palette; ordinary art remains exact.
    let original=[15,40,85,255,80,20,100,128];let bg=g.create_background_texture(2,1,&original);
    for light in [true,false] {g.set_light_ui(light);g.begin_frame();g.draw_list(&[Draw::Tex{x:0.0,y:0.0,w:2.0,h:1.0,tex:bg,alpha:1.0}]);let frame=g.read_frame();let expected=if light {slot_gfx::ui_palette::light_background(&original)}else{original.to_vec()};assert_eq!(&frame[..3],&expected[..3]);}
    // Exercise both shader palettes and actual motion on both graphics APIs.
    let mut first=vec![];
    for (name,light,phase) in [("dark-flow-field",false,0.0),("light-flow-field",true,0.0),("light-flow-field-moved",true,1.0)] {
        g.set_light_ui(light);g.begin_frame();g.draw_list(&[Draw::Lava{w:width as f32,h:480.0,phase}]);let frame=g.read_frame();
        if light {assert!(frame.chunks_exact(4).all(|p|p[..3].iter().all(|&c|c>=140)),"light flow became too dark");
            assert!(frame.chunks_exact(4).filter(|p|p[..3].iter().max().unwrap()-p[..3].iter().min().unwrap()>20).count()>width as usize*40,"light gradient colours washed out");if first.is_empty(){first=frame.clone();}else{assert!(first.iter().zip(&frame).filter(|(a,b)|a.abs_diff(**b)>3).count()>width as usize*20,"flow stopped moving");}}
        let mut e=png::Encoder::new(std::fs::File::create(dir.join(format!("{name}.png"))).unwrap(),width,480);e.set_color(png::ColorType::Rgba);e.set_depth(png::BitDepth::Eight);e.write_header().unwrap().write_image_data(&frame).unwrap();
    }
    g.set_light_ui(false);
    let mut f=Frontend::boot(Box::new(slot_power::SimPlatform::at(root.clone())));f.upload_faces(&mut g);f.render(&mut g,(width,480));let mut a=Review{f,g,now:0,dir,width};
    a.save("dark-carousel");a.tap(Btn::Menu);a.save("dark-grid");a.nav(QuickRow::General);a.tap(Btn::A);a.nav(QuickRow::Theme);a.save("dark-theme-row");a.tap(Btn::Right);assert!(a.f.app().light());a.save("light-theme-row");
    assert!(slot_store::read_slot_state(&root).appearance.light());
    a.tap(Btn::B);a.save("light-grid");a.nav(QuickRow::Bluetooth);a.tap(Btn::A);
    for i in 0..42 {a.frame(vec![]);if [0,1,2,5,9,20,41].contains(&i){a.capture(&format!("sound-entry-{i:02}"));}let mut out=vec![];a.f.app().draw(&mut out);for d in out{if let Draw::RoundRect{x,y,w,h,colour,..}=d{if colour==slot_ui::expressive::SELECTED{assert!(x>=55.9&&x+w<=width as f32-55.9&&y>=55.9&&y+h<=366.1,"focus escaped card: {x},{y},{w},{h}");if (h-slot_ui::expressive::ROW_H).abs()<0.1 {assert!((x+w/2.0-width as f32/2.0).abs()<0.1&& (w-(width as f32-168.0)).abs()<0.1,"submenu focus slid in horizontally");}}}}}
    a.save("light-sound");a.exit_review("sound");a.nav(QuickRow::Kids);a.tap(Btn::A);a.save("light-parent-settings");a.exit_review("parent");a.nav(QuickRow::Kids);a.tap(Btn::A);a.settle();a.tap(Btn::Down);a.tap(Btn::A);a.tap(Btn::L1);a.tap(Btn::R1);a.tap(Btn::L2);a.save("light-parent-code");a.tap(Btn::Menu);a.tap(Btn::B);
    assert!(matches!(a.f.app().phase(),Phase::Shelf));a.save("light-carousel");a.tap(Btn::Menu);a.nav(QuickRow::General);a.tap(Btn::A);a.nav(QuickRow::Theme);a.tap(Btn::Left);assert!(!a.f.app().light());a.save("dark-restored");
    a.g.set_light_ui(true);let badge=slot_ui::kid_time_meter(3600,3600);let tex=a.g.create_ui_texture(badge.w,badge.h,&badge.rgba);a.g.begin_frame();a.g.draw_list(&[Draw::Rect{x:0.0,y:0.0,w:width as f32,h:480.0,colour:[0.90,0.93,0.93,1.0]},Draw::Tex{x:(width-badge.w)as f32/2.0,y:190.0,w:badge.w as f32,h:badge.h as f32,tex,alpha:1.0}]);a.capture("light-time-badge");a.g.set_light_ui(false);
    // Fractional positions exposed the cropped off-state thumb. Check the
    // actual GLES/GL raster at both saved endpoints and a transition midpoint.
    for (index,t) in [0.0,0.5,1.0].into_iter().enumerate(){let mut draws=vec![Draw::Rect{x:0.0,y:0.0,w:width as f32,h:480.0,colour:[0.09,0.115,0.125,1.0]}];slot_ui::draw_switch(&mut draws,100.0,100.0,t,1.0);a.g.begin_frame();a.g.draw_list(&draws);let pixels=a.g.read_frame();let mut ys=vec![];for y in 95..140 {for x in 95..170 {let p=&pixels[((y*width+x)*4)as usize..][..4];if p[0]>230&&p[1]>230&&p[2]>230{ys.push(y);}}}assert!(!ys.is_empty());let lo=*ys.iter().min().unwrap();let hi=*ys.iter().max().unwrap();assert!((lo+hi)as i32>=228&&(lo+hi)as i32<=230,"switch thumb is not vertically centered");for y in 100..130 {let count=|row:u32|(95..170).filter(|x|{let p=&pixels[((row*width+x)*4)as usize..][..3];p.iter().all(|&v|v>230)}).count();assert!(count(y).abs_diff(count(229-y))<=2,"asymmetric switch thumb at row {y}");}a.capture(&format!("switch-{index}"));}
    // Production shelf views for every background choice, including custom and
    // random image paths. The fixture is a colourful image, not UI artwork.
    for (name,wallpaper) in [("flowing",slot_store::WallpaperPick::Default),("static",slot_store::WallpaperPick::Static),("plain",slot_store::WallpaperPick::None),("custom",slot_store::WallpaperPick::File("review.png".into())),("random",slot_store::WallpaperPick::Random)] {
        let root=a.dir.join(format!("card-{name}"));slot::root::ensure(&root);std::fs::create_dir_all(root.join("Wallpapers")).unwrap();
        let mut image=vec![];for y in 0..32 {for x in 0..32 {image.extend_from_slice(&[(x*8)as u8,(y*8)as u8,((31-x)*8)as u8,255]);}}
        let mut e=png::Encoder::new(std::fs::File::create(root.join("Wallpapers/review.png")).unwrap(),32,32);e.set_color(png::ColorType::Rgba);e.set_depth(png::BitDepth::Eight);e.write_header().unwrap().write_image_data(&image).unwrap();
        slot_store::write_slot_state(&root,&slot_store::SlotState{clock_set:true,appearance:slot_store::Appearance::Light,wallpaper,..Default::default()}).unwrap();
        for name in ["Aurora","Borealis"]{std::fs::write(root.join("Games/GBA").join(format!("{name}.gba")),[0;256]).unwrap();}
        let mut f=Frontend::boot(Box::new(slot_power::SimPlatform::at(root)));f.upload_faces(&mut a.g);f.render(&mut a.g,(width,480));a.f=f;a.now=0;a.save(&format!("light-background-{name}"));
    }

    // A dark UI over a bright custom wallpaper needs dark header text even
    // though the floating card and its labels retain their dark theme.
    {
        let root=a.dir.join("card-custom-contrast");slot::root::ensure(&root);std::fs::create_dir_all(root.join("Wallpapers")).unwrap();
        let mut pixels=vec![];for _y in 0..32 {for x in 0..64 {pixels.extend_from_slice(if x<22 {&[240,232,218,255]}else{&[24,32,40,255]});}}
        let mut e=png::Encoder::new(std::fs::File::create(root.join("Wallpapers/contrast.png")).unwrap(),64,32);e.set_color(png::ColorType::Rgba);e.set_depth(png::BitDepth::Eight);e.write_header().unwrap().write_image_data(&pixels).unwrap();
        slot_store::write_slot_state(&root,&slot_store::SlotState{clock_set:true,wallpaper:slot_store::WallpaperPick::File("contrast.png".into()),..Default::default()}).unwrap();
        let mut f=Frontend::boot(Box::new(slot_power::SimPlatform::at(root)));f.upload_faces(&mut a.g);f.render(&mut a.g,(width,480));a.f=f;a.now=0;a.tap(Btn::Menu);a.save("custom-header-contrast");a.nav(QuickRow::Bluetooth);a.tap(Btn::A);a.save("custom-sound-hints");a.tap(Btn::B);a.nav(QuickRow::Kids);a.tap(Btn::A);a.save("custom-parent-hints");
    }
    // Keycaps follow each footer backdrop independently of the global theme.
    // Reuse the same uploaded textures through both palette/theme round trips.
    {
        let faces=slot_ui::quick_legend_faces(slot_ui::footer_ink(false));
        let ids:[slot_gfx::TexId;2]=[0,2].map(|i|{let f=&faces[i];let id=a.g.create_ui_texture(f.w,f.h,&f.rgba);a.g.mark_hint_texture(id,usize::from(i!=0));id});
        let mut baseline=std::collections::BTreeMap::new();
        for (frame,(theme,left)) in [(false,true),(true,false),(true,true),(false,false),(false,true)].into_iter().enumerate() {
            a.g.set_light_ui(theme);a.g.set_hint_light([left,!left]);
            let bright=slot_gfx::ui_palette::BACKDROP;let dark=[0.04,0.06,0.08,1.0];
            let mut out=vec![Draw::Rect{x:0.0,y:0.0,w:width as f32/2.0,h:480.0,colour:if left{bright}else{dark}},Draw::Rect{x:width as f32/2.0,y:0.0,w:width as f32/2.0,h:480.0,colour:if left{dark}else{bright}}];
            for (side,i) in [0,2].into_iter().enumerate(){let f=&faces[i];out.push(Draw::Tex{x:if side==0{16.0}else{width as f32-16.0-f.w as f32},y:200.0,w:f.w as f32,h:f.h as f32,tex:ids[side],alpha:1.0});}
            a.g.begin_frame();a.g.draw_list(&out);let pixels=a.g.read_frame();
            let cap_pixel=|side:usize| {let x=if side==0{20}else{width-24};let y=200+slot_ui::MENU_HINT_H/2;let i=((y*width+x)*4)as usize;pixels[i..i+3].to_vec()};
            assert_eq!(cap_pixel(0),if left{slot_gfx::ui_palette::INK.to_vec()}else{vec![224,227,232]});
            assert_eq!(cap_pixel(1),if left{vec![224,227,232]}else{slot_gfx::ui_palette::INK.to_vec()});
            if let Some(previous)=baseline.get(&left){assert_eq!(&pixels,previous,"global theme or palette round trip changed adaptive hint artwork");}else{baseline.insert(left,pixels);}
            a.capture(&format!("adaptive-hints-{frame}"));
        }
    }
    // Inspect native-sized battery glyphs, slider thumbs, charging and error
    // notices through the same compositor used on the handheld.
    for light in [false,true] {
        a.g.set_light_ui(light);let mut out=vec![Draw::Rect{x:0.0,y:0.0,w:width as f32,h:480.0,colour:if light {slot_gfx::ui_palette::BACKDROP}else{[0.04,0.06,0.08,1.0]}}];
        for (row,charge) in [slot_power::Charge::Discharging,slot_power::Charge::Charging].into_iter().enumerate() {
            for (i,percent) in [0,10,50,82,100].into_iter().enumerate() {
                let face=slot_ui::status_battery_face_on(slot_power::Battery{percent,charge},light);
                let tex=a.g.create_texture(face.w,face.h,&face.rgba);
                out.push(Draw::Tex{x:64.0+i as f32*102.0,y:60.0+row as f32*50.0,w:face.w as f32,h:face.h as f32,tex,alpha:1.0});
            }
        }
        for (i,value) in [0.0,0.5,1.0].into_iter().enumerate(){slot_ui::draw_slider(&mut out,80.0+i as f32*170.0,180.0,140.0,value,1.0);}
        let mut hud_out=vec![Draw::Rect{x:0.0,y:0.0,w:width as f32,h:480.0,colour:if light {slot_gfx::ui_palette::BACKDROP}else{[0.04,0.06,0.08,1.0]}}];
        let icons:Vec<_>=slot_ui::Icon::ALL.into_iter().map(|icon|{let f=slot_ui::control_icon_face(icon,24,slot_ui::HUD_INK);a.g.create_ui_texture(f.w,f.h,&f.rgba)}).collect();
        for (i,(kind,value,muted)) in [(slot_ui::HudKind::Volume,60,false),(slot_ui::HudKind::Volume,0,true),(slot_ui::HudKind::Brightness,6,false),(slot_ui::HudKind::BlueLight,5,false)].into_iter().enumerate(){
            let mut hud=slot_ui::Hud::new();hud.set_icons(icons.clone());hud.show(kind,value,muted,0);let start=hud_out.len();hud.draw(a.f.app().canvas(),300,&mut hud_out);
            for d in &mut hud_out[start..] {match d {Draw::Rect{y,..}|Draw::RoundRect{y,..}|Draw::Tex{y,..}|Draw::Part{y,..}=>*y+=70.0+i as f32*80.0,_=>{}}}
        }
        a.g.begin_frame();a.g.draw_list(&hud_out);a.capture(if light{"light-hud-controls"}else{"dark-hud-controls"});
        let message="Charging: 57%";let face=slot_ui::settings_notice_face_on(message,a.f.app().canvas(),light);let tex=a.g.create_texture(face.w,face.h,&face.rgba);
        let mut n=slot_ui::SettingsNotice::default();n.observe(Some((1,message.into())),0);n.draw_on(a.f.app().canvas(),Some((tex,face.w,face.h)),1200,light,&mut out);
        a.g.begin_frame();a.g.draw_list(&out);a.capture(if light{"light-controls-contrast"}else{"dark-controls-contrast"});
        let message=slot::bluetooth::error_notice("Error: Connection failed: org.bluez.Error.Failed br-connection-busy");
        let face=slot_ui::settings_notice_face_on(message,a.f.app().canvas(),light);let tex=a.g.create_texture(face.w,face.h,&face.rgba);let mut n=slot_ui::SettingsNotice::default();n.observe(Some((2,message.into())),0);
        let mut out=vec![Draw::Rect{x:0.0,y:0.0,w:width as f32,h:480.0,colour:if light {slot_gfx::ui_palette::BACKDROP}else{[0.04,0.06,0.08,1.0]}}];n.draw_on(a.f.app().canvas(),Some((tex,face.w,face.h)),1200,light,&mut out);
        a.g.begin_frame();a.g.draw_list(&out);a.capture(if light{"light-error-notice"}else{"dark-error-notice"});
    }
    // Open actual in-game settings over the mock core's bright test picture.
    for light in [false,true] {
        let root=a.dir.join(if light{"card-game-light"}else{"card-game-dark"});slot::root::ensure(&root);
        for name in ["Aurora","Borealis"]{std::fs::write(root.join("Games/GBA").join(format!("{name}.gba")),[0;256]).unwrap();}
        slot_store::write_slot_state(&root,&slot_store::SlotState{clock_set:true,cart:Some("Aurora".into()),appearance:if light{slot_store::Appearance::Light}else{slot_store::Appearance::Dark},..Default::default()}).unwrap();
        let mut f=Frontend::boot(Box::new(slot_power::SimPlatform::at(root)));f.upload_faces(&mut a.g);f.render(&mut a.g,(width,480));a.f=f;a.now=0;
        for _ in 0..180 {a.frame(vec![]);if matches!(a.f.app().phase(),Phase::Playing{..}){break;}}
        assert!(matches!(a.f.app().phase(),Phase::Playing{..}));a.tap(Btn::Menu);a.save(if light{"light-game-pause"}else{"dark-game-pause"});
        for _ in 0..12 {if a.f.app().in_game_menu()==Some(QuickRow::Settings){break;}a.tap(Btn::Right);}
        assert_eq!(a.f.app().in_game_menu(),Some(QuickRow::Settings));a.tap(Btn::A);a.save(if light{"light-game-settings"}else{"dark-game-settings"});
    }

    std::fs::write(a.dir.join("review-complete.txt"),b"Complete native contrast review\n").unwrap();
}
#[cfg(not(target_os="linux"))] fn main(){}
