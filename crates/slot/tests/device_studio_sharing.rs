mod common;
use slot::{
    art::Artwork,
    studio::{LabelKind, Screen, Studio, Target},
};
use slot_input::{Action, Btn};
use slot_store::{Outline, Platform, ShellChoice, ShellFinish};
use std::{
    fs,
    time::{Duration, Instant},
};

#[test]
fn editor_saves_label_and_shell_without_replacing_other_choices() {
    let root = common::tmp_root_with_carts(&["Fusion", "Emerald"]);
    let carts = slot_store::scan(root.path()).unwrap();
    let cart = carts.iter().find(|c| c.stem == "Fusion").unwrap().clone();
    let other = ShellChoice {
        outline: Outline::Auto,
        colour: [3, 4, 5],
        finish: ShellFinish::Clear,
    };
    slot_store::cart_shell::write_choice(root.path(), "Emerald", Some(&other)).unwrap();
    let mut model = Studio::new(cart, false);
    model.applied(LabelKind::Title);
    model.ground = Some([40, 80, 120]);
    model.shell = Some(ShellChoice {
        colour: [200, 30, 40],
        finish: ShellFinish::Glitter,
        ..other
    });
    let saved = slot::studio_menu::save(root.path(), &model, &Artwork::default(), None).unwrap();
    let label = fs::read(saved.label.as_ref().unwrap()).unwrap();
    assert!(label.starts_with(b"\x89PNG"));
    assert_eq!(
        slot_ui::house_label::label_tag(&label),
        Some(slot_ui::house_label::LabelTag::Text([40, 80, 120]))
    );
    let scan = slot_store::scan(root.path()).unwrap();
    assert_eq!(
        scan.iter().find(|c| c.stem == "Fusion").unwrap().shell,
        model.shell
    );
    assert_eq!(
        scan.iter().find(|c| c.stem == "Emerald").unwrap().shell,
        Some(other)
    );
}
#[test]
fn failed_shell_save_restores_the_original_label() {
    let root = common::tmp_root_with_carts(&["Fusion"]);
    let mut cart = slot_store::scan(root.path()).unwrap().remove(0);
    let path = root.path().join("Labels/GBA/Fusion.png");
    fs::write(&path, b"original").unwrap();
    cart.label = Some(path.clone());
    fs::create_dir(root.path().join("Labels/cart_shell.txt")).unwrap();
    let mut model = Studio::new(cart, false);
    model.applied(LabelKind::Title);
    model.ground = Some([20, 30, 40]);
    assert!(
        slot::studio_menu::save(root.path(), &model, &Artwork::default(), Some(b"original"))
            .is_err()
    );
    assert_eq!(fs::read(path).unwrap(), b"original");
}
#[test]
fn missing_original_snapshot_is_recovered_before_save_or_rollback() {
    let root = common::tmp_root_with_carts(&["Fusion"]);
    let path = root.path().join("Labels/GBA/Fusion.png");
    fs::write(&path, b"original label").unwrap();
    let cart = slot_store::scan(root.path()).unwrap().remove(0);
    let mut model = Studio::new(cart, false);
    let saved = slot::studio_menu::save(root.path(), &model, &Artwork::default(), None).unwrap();
    assert_eq!(saved.label.as_ref(), Some(&path));
    assert_eq!(fs::read(&path).unwrap(), b"original label");

    fs::remove_file(root.path().join("Labels/cart_shell.txt")).unwrap();
    fs::create_dir(root.path().join("Labels/cart_shell.txt")).unwrap();
    model.applied(LabelKind::Title);
    model.ground = Some([20, 30, 40]);
    assert!(slot::studio_menu::save(root.path(), &model, &Artwork::default(), None).is_err());
    assert_eq!(fs::read(path).unwrap(), b"original label");
}

#[test]
fn unreadable_original_aborts_before_changing_label_or_shell() {
    let root = common::tmp_root_with_carts(&["Fusion"]);
    let path = root.path().join("Labels/GBA/Fusion.png");
    let mut cart = slot_store::scan(root.path()).unwrap().remove(0);
    cart.label = Some(path.clone());
    let mut model = Studio::new(cart, false);
    model.applied(LabelKind::Title);
    model.ground = Some([20, 30, 40]);
    // No snapshot and a missing source simulate a failed original-label read.
    assert_eq!(
        slot::studio_menu::save(root.path(), &model, &Artwork::default(), None)
            .unwrap_err().kind(),
        std::io::ErrorKind::NotFound
    );
    assert!(!path.exists());
    assert!(!root.path().join("Labels/cart_shell.txt").exists());
}

#[test]
fn studio_replaces_and_removes_the_scanned_label_filename() {
    let root = common::tmp_root_with_carts(&["Fusion"]);
    let path = root.path().join("Labels/GBA/FUSION.PNG");
    fs::write(&path, b"original label").unwrap();
    let cart = slot_store::scan(root.path()).unwrap().remove(0);
    assert_eq!(cart.label.as_ref(), Some(&path));
    let mut model = Studio::new(cart, false);
    model.applied(LabelKind::Title);
    model.ground = Some([20, 30, 40]);
    let saved = slot::studio_menu::save(root.path(), &model, &Artwork::default(), Some(b"original label")).unwrap();
    assert_eq!(saved.label.as_ref(), Some(&path));
    let png = fs::read(&path).unwrap();
    assert_eq!(slot_ui::house_label::label_tag(&png), Some(slot_ui::house_label::LabelTag::Text([20, 30, 40])));
    assert_eq!(fs::read_dir(path.parent().unwrap()).unwrap().count(), 1);
    let mut reset = Studio::new(saved, false);
    reset.adopt_text([20, 30, 40]);
    reset.applied(LabelKind::Title);
    let saved = slot::studio_menu::save(root.path(), &reset, &Artwork::default(), None).unwrap();
    assert!(saved.label.is_none());
    assert!(!path.exists());
    assert!(slot_store::scan(root.path()).unwrap()[0].label.is_none());
}

#[test]
fn changing_shape_reflows_a_saved_title_with_its_original_color() {
    let root = common::tmp_root_with_gb_carts(&["Pocket"]);
    let cart = slot_store::scan(root.path()).unwrap().remove(0);
    let original = slot::art::make::text_label(&cart, [30, 80, 120]);
    let path = root.path().join("Labels/GB/Pocket.png");
    fs::write(&path, &original).unwrap();
    let cart = slot_store::scan(root.path()).unwrap().remove(0);
    let old_size = slot_ui::label_size(&cart);
    let mut menu = slot::studio_menu::Menu::new(root.path().to_path_buf(), cart, false);
    menu.model.shell = Some(ShellChoice {
        outline: Outline::Rounded,
        colour: [70, 80, 90],
        finish: ShellFinish::Clear,
    });
    let saved = slot::studio_menu::save(root.path(), &menu.model, &Artwork::default(), Some(&original)).unwrap();
    let (_, w, h) = slot_ui::decode_png(&path).unwrap();
    assert_ne!(old_size, (w, h));
    assert_eq!((w, h), slot_ui::label_size(&saved));
    assert_eq!(slot_ui::house_label::label_tag(&fs::read(path).unwrap()), Some(slot_ui::house_label::LabelTag::Text([30, 80, 120])));
}

#[test]
fn studio_is_modal_saves_on_exit_and_is_not_opened_in_kid_mode() {
    let root = common::tmp_root_with_carts(&["Fusion", "Emerald"]);
    slot_store::write_slot_state(
        root.path(),
        &slot_store::SlotState {
            clock_set: true,
            ..Default::default()
        },
    )
    .unwrap();
    let (mut app, _clock) = common::app_booting_at(root.path(), common::CLOCK_IS_SET);
    app.open_studio();
    assert!(app.studio.is_some());
    assert_eq!(app.input_context(), slot_input::Context::Menu);
    let editor = app.studio.as_mut().unwrap();
    editor.model.screen = Screen::Custom {
        target: Target::Shell,
        rgb: [10, 20, 30],
        start: [10, 20, 30],
        channel: 0,
    };
    app.apply(Action::GbaDown(Btn::Right));
    assert_eq!(
        app.studio.as_ref().unwrap().model.current_shell().colour,
        [11, 20, 30]
    );
    app.studio.as_mut().unwrap().model.screen = Screen::List { row: 0 };
    app.apply(Action::GbaDown(Btn::B));
    let until = Instant::now() + Duration::from_secs(10);
    while app.studio.is_some() {
        app.tick_ms(app.now() + 17);
        assert!(Instant::now() < until);
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(slot_store::scan(root.path())
        .unwrap()
        .iter()
        .any(|c| c.shell.is_some_and(|s| s.colour == [11, 20, 30])));
    app.kids.active = true;
    app.open_studio();
    assert!(app.studio.is_none());
}
#[test]
fn artwork_catalog_cannot_inject_paths_or_hosts() {
    for path in [
        "../secret.png",
        "/Config/password",
        "https://evil/x",
        "GB/%2e%2e/x",
        "a\\b.png",
        "x\n.png",
    ] {
        assert!(!slot::art::safe_asset(path), "{path}");
    }
    assert!(slot::art::safe_asset("GBA/Labels/Metroid Fusion.png"));
}

#[test]
fn custom_game_boy_title_is_sized_for_the_edited_cartridge_shape() {
    let root=common::tmp_root_with_gb_carts(&["Pocket"]);
    let cart=slot_store::scan(root.path()).unwrap().remove(0);
    let mut model=Studio::new(cart,false);model.applied(LabelKind::Title);model.ground=Some([30,80,120]);
    model.shell=Some(ShellChoice{outline:Outline::Rounded,colour:[70,80,90],finish:ShellFinish::Clear});
    let saved=slot::studio_menu::save(root.path(),&model,&Artwork::default(),None).unwrap();
    let (_,w,h)=slot_ui::decode_png(saved.label.as_ref().unwrap()).unwrap();
    assert_eq!((w,h),slot_ui::label_size(&saved));
    assert_eq!(slot_store::scan(root.path()).unwrap()[0].shell,model.shell);
}

#[test]
fn scanned_label_conversion_preserves_transparent_corners_instead_of_flattening_black() {
    let (w,h)=(64,32);let mut pixels=vec![0;w*h*4];
    for y in 0..h {for x in 0..w {let i=(y*w+x)*4;pixels[i..i+4].copy_from_slice(if x<4&&y<4 {&[255,255,255,0]}else{&[80,160,120,255]});}}
    let mut png=Vec::new();{let mut encoder=png::Encoder::new(&mut png,w as u32,h as u32);encoder.set_color(png::ColorType::Rgba);encoder.set_depth(png::BitDepth::Eight);encoder.write_header().unwrap().write_image_data(&pixels).unwrap();}
    let converted=slot::art::make::real_label(&png,Platform::Gba).unwrap();
    let root=tempfile::tempdir().unwrap();let path=root.path().join("label.png");fs::write(&path,converted).unwrap();
    let (pixels,w,h)=slot_ui::decode_png(&path).unwrap();assert_eq!(pixels[3],0);
    assert_eq!(pixels[((h/2*w+w/2)*4+3)as usize],255);
    assert_eq!(slot_ui::cover_label(&path,240,124),slot_ui::cover_label_bytes(&fs::read(path).unwrap(),240,124));
}
