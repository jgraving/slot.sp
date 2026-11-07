mod common;
use slot::{app::{App,Phase}, studio::{LabelKind,Screen}, studio_menu::Menu};
use slot_input::{Action,Btn};
use slot_store::{SlotState,write_slot_state};
use std::time::{Duration,Instant};

fn app() -> (tempfile::TempDir,App) {
    let root=common::tmp_root_with_carts(&["Aurora","Borealis"]);
    write_slot_state(root.path(),&SlotState{clock_set:true,..Default::default()}).unwrap();
    let mut app=App::boot(root.path());
    for _ in 0..60 {app.update(1.0/60.0);}
    (root,app)
}
#[test]
fn select_opens_the_highlighted_cart_without_a_quick_controls_page() {
    let (_root,mut app)=app();
    assert!(matches!(app.phase(),Phase::Shelf));
    app.apply(Action::CartStudio);
    assert_eq!(app.studio.as_ref().unwrap().model.cart.stem,"Aurora");
    assert!(app.quick_menu().is_none());
    app.apply(Action::CartStudio);
    assert_eq!(app.studio.as_ref().unwrap().model.cart.stem,"Aurora");
}
#[test]
fn a_held_play_button_cannot_launch_behind_studio() {
    let (_root,mut app)=app();
    app.apply(Action::GbaDown(Btn::A));
    app.apply(Action::CartStudio);
    app.apply(Action::GbaUp(Btn::A));
    for _ in 0..60 {app.update(1.0/60.0);}
    assert!(app.studio.is_some());
    assert!(app.seated_cart().is_none());
}
#[test]
fn studio_shortcut_does_not_replace_an_open_settings_page() {
    let (_root,mut app)=app();
    app.apply(Action::QuickMenu);
    app.apply(Action::CartStudio);
    assert!(app.studio.is_none());
    assert!(app.quick_menu().is_some());
}
fn ready(menu:&mut Menu) {
    let deadline=Instant::now()+Duration::from_secs(15);
    while !menu.preview_ready() {
        assert!(Instant::now()<deadline,"Studio worker stalled");
        menu.update();std::thread::sleep(Duration::from_millis(5));
    }
}
#[test]
fn power_save_completes_the_chosen_label_even_during_its_animation() {
    let (root,_)=app();
    let cart=slot_store::scan(root.path()).unwrap().remove(0);
    let mut menu=Menu::new(root.path().into(),cart,false);
    ready(&mut menu);
    menu.update_at(1000);
    menu.input(Action::GbaDown(Btn::A));
    assert!(matches!(menu.model.screen,Screen::Sheet{..}));
    menu.input(Action::GbaDown(Btn::A));
    assert!(menu.applying());
    menu.finish();
    assert!(!menu.applying());
    assert_eq!(menu.model.wearing,LabelKind::Title);
    let deadline=Instant::now()+Duration::from_secs(15);
    while menu.busy() {
        assert!(Instant::now()<deadline);
        menu.update();std::thread::sleep(Duration::from_millis(5));
    }
    assert!(menu.take_saved().is_some());
}
#[test]
fn fetched_art_suppresses_matching_pixels_but_keeps_the_logo_wheel() {
    fn png(colour:[u8;3],fast:bool)->Vec<u8> {
        let mut out=vec![];
        {let mut encoder=png::Encoder::new(&mut out,32,16);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_compression(if fast {png::Compression::Fast}else{png::Compression::Best});
        encoder.write_header().unwrap().write_image_data(&colour.repeat(32*16)).unwrap();}
        out
    }
    let original=png([80,120,160],true);
    let mut art=slot::art::Artwork{real:Some(png([80,120,160],false)),
        logo:Some(png([200,20,30],true)),wheel:Some(vec![1,2,3]),..Default::default()};
    assert!(art.without_duplicates(Some(&original)));
    assert!(art.real.is_none());assert!(art.logo.is_some());assert!(art.wheel.is_some());
    let mut both=slot::art::Artwork{real:Some(original.clone()),logo:Some(original),..Default::default()};
    assert!(both.without_duplicates(None));assert!(both.real.is_some());assert!(both.logo.is_none());
}

fn change_finish(menu:&mut Menu) {
    let row=menu.model.rows().iter().position(|r|*r==slot::studio::Row::Finish).unwrap();
    menu.model.screen=Screen::List{row};
    menu.input(Action::GbaDown(Btn::Right));
}
fn wait_for_save(menu:&mut Menu) {
    let deadline=Instant::now()+Duration::from_secs(15);
    while menu.busy() {menu.update();assert!(Instant::now()<deadline);std::thread::sleep(Duration::from_millis(5));}
}
#[test]
fn save_stays_open_and_b_closes_without_reprompting_once_saved() {
    let (root,_)=app();let cart=slot_store::scan(root.path()).unwrap().remove(0);
    let mut menu=Menu::new(root.path().into(),cart,false);
    change_finish(&mut menu);assert!(menu.has_unsaved_changes());
    let row=menu.model.rows().iter().position(|r|*r==slot::studio::Row::Save).unwrap();
    menu.model.screen=Screen::List{row};menu.input(Action::GbaDown(Btn::A));wait_for_save(&mut menu);
    assert!(menu.visible);assert!(menu.take_saved().is_some());assert!(!menu.has_unsaved_changes());
    menu.input(Action::GbaDown(Btn::B));assert!(!menu.visible);assert!(!menu.confirming_close());
}
#[test]
fn unsaved_close_can_resume_editing_or_discard_without_touching_the_card() {
    let (root,_)=app();let cart=slot_store::scan(root.path()).unwrap().remove(0);
    let stem=cart.stem.clone();let original=cart.shell;
    let mut menu=Menu::new(root.path().into(),cart,false);change_finish(&mut menu);
    menu.input(Action::GbaDown(Btn::B));assert!(menu.confirming_close());assert!(!menu.busy());
    menu.input(Action::GbaDown(Btn::B));assert!(!menu.confirming_close());assert!(menu.visible);
    menu.input(Action::GbaDown(Btn::B));menu.input(Action::GbaDown(Btn::Down));menu.input(Action::GbaDown(Btn::A));
    assert!(!menu.visible);assert!(menu.take_saved().is_none());
    assert_eq!(slot_store::scan(root.path()).unwrap().into_iter().find(|c|c.stem==stem).unwrap().shell,original);
}
#[test]
fn unchanged_close_needs_neither_save_nor_confirmation() {
    let (root,_)=app();let cart=slot_store::scan(root.path()).unwrap().remove(0);
    let mut menu=Menu::new(root.path().into(),cart,false);
    menu.input(Action::GbaDown(Btn::B));assert!(!menu.visible);assert!(!menu.busy());
    assert!(!menu.confirming_close());assert!(menu.take_saved().is_none());
}
#[test]
fn submenus_keep_the_cartridge_pose_after_the_shared_arrival() {
    let (root,_)=app();let cart=slot_store::scan(root.path()).unwrap().remove(0);
    let mut menu=Menu::new(root.path().into(),cart,false);ready(&mut menu);
    menu.source_rect=Some([122.0,90.0,396.0,228.0]);menu.update_at(1000);
    let canvas=slot_ui::Canvas{w:640.0,h:480.0};assert_eq!(menu.shown_cart_rect(canvas),menu.source_rect.unwrap());
    menu.update_at(2000);let rest=menu.shown_cart_rect(canvas);
    assert!(rest[2]<396.0 && rest[0]<122.0);
    menu.input(Action::GbaDown(Btn::Right));assert!(matches!(menu.model.screen,Screen::Sheet{..}));
    assert_eq!(menu.shown_cart_rect(canvas),rest);
    menu.input(Action::GbaDown(Btn::B));assert_eq!(menu.shown_cart_rect(canvas),rest);
}
#[test]
fn right_opens_disclosures_but_finish_adjusts_and_save_requires_a() {
    let (root,_)=app();let cart=slot_store::scan(root.path()).unwrap().remove(0);
    let mut menu=Menu::new(root.path().into(),cart,false);
    let color=menu.model.rows().iter().position(|r|*r==slot::studio::Row::Color).unwrap();
    menu.model.screen=Screen::List{row:color};menu.input(Action::GbaDown(Btn::Right));
    assert!(matches!(menu.model.screen,Screen::Chips{..}));menu.input(Action::GbaDown(Btn::B));
    let old=menu.model.current_shell();change_finish(&mut menu);
    assert_ne!(menu.model.current_shell(),old);assert!(matches!(menu.model.screen,Screen::List{..}));
    let save=menu.model.rows().iter().position(|r|*r==slot::studio::Row::Save).unwrap();
    menu.model.screen=Screen::List{row:save};menu.input(Action::GbaDown(Btn::Right));
    assert!(!menu.busy());assert!(menu.visible);
}
