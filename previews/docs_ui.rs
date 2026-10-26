//! Render documentation examples through the production menu and compositor code.
#[path = "egl_surface.rs"]
mod egl_surface;
use slot::{kid_menu::Menu, kid_policy::Policy, parent_code::Gate};
use slot_gfx::{Canvas, Compositor, Draw};
use slot_input::{Action, Btn};
use slot_ui::{UndoFace, StickerFields};
use std::{fs::File, path::Path};

const WALL: i64 = 1_790_784_000;
fn save(path: &Path, w: u32, h: u32, pixels: &[u8]) {
    let mut e=png::Encoder::new(File::create(path).unwrap(),w,h);
    e.set_color(png::ColorType::Rgba);e.set_depth(png::BitDepth::Eight);
    e.write_header().unwrap().write_image_data(pixels).unwrap();
}
fn press(menu:&mut Menu,button:Btn){
    menu.input(Action::GbaDown(button),WALL);
    menu.input(Action::GbaUp(button),WALL);
}
fn main(){
    let args:Vec<_>=std::env::args().skip(1).collect();
    let [root,out]=args.as_slice() else {panic!("ROOT OUT")};
    let root=Path::new(root).join("documentation-only-parent-code");
    std::fs::create_dir_all(Policy::directory(&root)).unwrap();
    let mut gate=Gate::new(Policy::directory(&root).join("code"));
    gate.set(&[0,1,2,3]).unwrap();
    let out=Path::new(out);std::fs::create_dir_all(out.join("pin-frames")).unwrap();
    let surface=egl_surface::HeadlessSurface::new().unwrap();
    let mut c=Compositor::new(&surface).unwrap();
    let bg=c.create_texture(720,480,slot_ui::default_background().unwrap());
    let [back,front]=slot_ui::shell_slot_faces(slot_store::ShellColour::Platinum);
    let slot=Some((c.create_texture(720,slot_ui::SLOT_H,back),c.create_texture(720,slot_ui::SLOT_H,front)));
    let ui=c.create_texture(720,480,&vec![0;720*480*4]);
    let mut render=|face:UndoFace,path:&Path|{
        assert_eq!((face.w,face.h),(720,480));
        c.update_texture(ui,720,480,&face.rgba);
        let mut draws=vec![Draw::Tex{x:0.,y:0.,w:720.,h:480.,tex:bg,alpha:1.}];
        slot_ui::draw_empty_slot(Canvas::FRAMED,slot,&mut draws);
        draws.push(Draw::Tex{x:0.,y:0.,w:720.,h:480.,tex:ui,alpha:1.});
        c.begin_frame();c.draw_list(&draws);save(path,720,480,&c.read_frame());
    };
    let mut policy=Policy::default();policy.minutes=90;policy.approved.push((slot_store::Platform::Gba,"Emerald".into()));policy.begin(WALL).unwrap();
    let mut home=Menu::new(root.clone(),policy.clone());
    for b in [Btn::A,Btn::B,Btn::X,Btn::Y]{press(&mut home,b);}
    home.set_remaining(Some((75*60,90*60)));
    render(home.screen_face(),&out.join("parent-settings.png"));
    press(&mut home,Btn::Down);press(&mut home,Btn::Down);press(&mut home,Btn::A);
    press(&mut home,Btn::Down);
    render(home.screen_face(),&out.join("parent-time.png"));
    let mut child=Menu::child(root.clone(),policy.clone(),true,50,55);
    child.set_remaining(Some((75*60,90*60)));
    render(child.screen_face(),&out.join("kid-menu.png"));
    let kid_pages=slot::reference::pages(true,true,slot_store::Platform::Gba,slot_store::Controls::default(),slot_store::ShortcutKeys::default(),slot_store::CarouselKeys::default());
    let mut kid_help=Menu::reference(root.clone(),policy.clone(),kid_pages);
    press(&mut kid_help,Btn::Right);
    render(kid_help.screen_face(),&out.join("kid-controls.png"));
    let adult_pages=slot::reference::pages(false,true,slot_store::Platform::Gba,slot_store::Controls::default(),slot_store::ShortcutKeys::default(),slot_store::CarouselKeys::default());
    let mut help=Menu::reference(root.clone(),policy.clone(),adult_pages);
    for _ in 0..3 { press(&mut help,Btn::Right); }
    render(help.screen_face(),&out.join("buttons-shortcuts.png"));

    render(Menu::warning(root.clone(),policy.clone(),15*60).screen_face(),&out.join("kid-warning.png"));
    let mut code=Menu::new(root.clone(),policy.clone());
    for b in [Btn::A,Btn::B,Btn::X]{press(&mut code,b);}
    render(code.screen_face(),&out.join("parent-code.png"));
    let mut demo=Menu::new(root.clone(),policy);
    let mut frame=0;
    // Wrong code: enter four visible buttons, shake, clear. Then the correct code
    // proceeds directly to settings.
    for b in [Btn::A,Btn::A,Btn::A,Btn::A]{
        press(&mut demo,b);
        for _ in 0..4 {render(demo.screen_face(),&out.join(format!("pin-frames/{frame:04}.png")));frame+=1;demo.update(0.05);}
    }
    for _ in 0..12 {render(demo.screen_face(),&out.join(format!("pin-frames/{frame:04}.png")));frame+=1;demo.update(0.05);}
    for b in [Btn::A,Btn::B,Btn::X,Btn::Y]{
        press(&mut demo,b);
        for _ in 0..4 {render(demo.screen_face(),&out.join(format!("pin-frames/{frame:04}.png")));frame+=1;}
    }
    for _ in 0..16 {render(demo.screen_face(),&out.join(format!("pin-frames/{frame:04}.png")));frame+=1;}
    let lines=["","","COMPACT","ROW BLUETOOTH On","ROW SPEAKER_DEVICE Internal speaker", "SELECTED ACTIVE DEVICE Headphones","ROW USB_DEVICE USB headphones","ROW Pair audio device","ROW DISCONNECT 0%","B Back · A Select · Y Forget"].map(str::to_owned);
    render(slot::bluetooth::screen_face_on(&lines,slot_ui::footer_ink(false),Canvas::FRAMED),&out.join("sound.png"));
    let sticker=slot_ui::sticker_face(&StickerFields{battery:Some(87),serial:"04738857",version:"0.1.0",model:"RG SP"});
    save(&out.join("about.png"),sticker.w,sticker.h,&sticker.rgba);
}
