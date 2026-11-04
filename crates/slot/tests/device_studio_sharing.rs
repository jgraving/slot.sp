mod common;
use slot::{
    art::Artwork,
    sharing,
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
fn imports_are_isolated_atomic_and_never_overwrite_existing_games() {
    let root = common::tmp_root_with_carts(&["Fusion"]);
    for p in Platform::ALL {
        fs::create_dir_all(root.path().join("Inbox").join(p.dir_name())).unwrap();
    }
    let inbox = root.path().join("Inbox/GBA");
    fs::write(inbox.join("New.gba"), [1u8; 1024]).unwrap();
    fs::write(inbox.join("Fusion.gba"), [2u8; 1024]).unwrap();
    fs::write(inbox.join("evil.sh"), [3u8; 1024]).unwrap();
    fs::write(inbox.join(".partial.gba"), [4u8; 1024]).unwrap();
    fs::write(inbox.join("broken.gba"), [4u8; 10]).unwrap();
    let original = fs::read(root.path().join("Games/GBA/Fusion.gba")).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(
        root.path().join("Games/GBA/Fusion.gba"),
        inbox.join("link.gba"),
    )
    .unwrap();
    let result = sharing::import(root.path()).unwrap();
    assert_eq!(result.added.len(), 1);
    assert_eq!(result.added[0].stem, "New");
    assert_eq!(
        fs::read(root.path().join("Games/GBA/New.gba")).unwrap(),
        [1u8; 1024]
    );
    assert!(!inbox.join("New.gba").exists());
    assert_eq!(
        fs::read(root.path().join("Games/GBA/Fusion.gba")).unwrap(),
        original
    );
    assert!(inbox.join("Fusion.gba").exists());
    assert!(!root.path().join("Games/GBA/evil.sh").exists());
    assert!(!root.path().join("Games/GBA/link.gba").exists());
}

fn wallpaper_png(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(&vec![120; width as usize * height as usize * 4])
            .unwrap();
    }
    bytes
}

#[test]
fn wallpaper_imports_are_visible_without_restart_and_never_overwrite() {
    let root = common::tmp_root_with_carts(&["Fusion"]);
    let (app, _clock) = common::app_booting_at(root.path(), common::CLOCK_IS_SET);
    let inbox = root.path().join("Inbox/Wallpapers");
    fs::create_dir_all(&inbox).unwrap();
    fs::create_dir_all(root.path().join("Wallpapers")).unwrap();
    let png = wallpaper_png(8, 6);
    fs::write(root.path().join("Wallpapers/Keep.png"), b"original").unwrap();
    for name in [
        "New.PNG",
        "Keep.png",
        ".partial.png",
        "wrong.jpg",
        "bad\\name.png",
        "bad\nname.png",
    ] {
        fs::write(inbox.join(name), &png).unwrap();
    }
    fs::create_dir(inbox.join("folder.png")).unwrap();
    let result = sharing::import(root.path()).unwrap();
    assert!(result.added.is_empty());
    assert_eq!(result.wallpapers, 1);
    assert_eq!(result.skipped, 6);
    assert_eq!(
        fs::read(root.path().join("Wallpapers/New.PNG")).unwrap(),
        png
    );
    assert!(!inbox.join("New.PNG").exists());
    assert_eq!(
        fs::read(root.path().join("Wallpapers/Keep.png")).unwrap(),
        b"original"
    );
    assert!(inbox.join("Keep.png").exists());
    assert_eq!(app.wallpaper_files(), ["Keep.png", "New.PNG"]);
    assert!(!root
        .path()
        .join("Wallpapers")
        .join(format!(".slot-upload-{}", std::process::id()))
        .exists());
}

#[test]
fn wallpapers_require_full_bounded_png_decode() {
    let root = tempfile::tempdir().unwrap();
    let inbox = root.path().join("Inbox/Wallpapers");
    fs::create_dir_all(&inbox).unwrap();
    let valid = wallpaper_png(8, 6);
    fs::write(inbox.join("Good.png"), &valid).unwrap();
    fs::write(inbox.join("Signature.png"), b"\x89PNG\r\n\x1a\nnot a PNG").unwrap();
    fs::write(inbox.join("Truncated.png"), &valid[..valid.len() - 12]).unwrap();
    let mut corrupt = valid.clone();
    let idat = corrupt.windows(4).position(|s| s == b"IDAT").unwrap();
    corrupt[idat + 5] ^= 0xff;
    fs::write(inbox.join("Corrupt.png"), corrupt).unwrap();
    fs::write(inbox.join("TooWide.png"), wallpaper_png(4097, 1)).unwrap();
    // Reject decompression bombs by dimensions even when their encoded bytes are tiny.
    fs::write(inbox.join("TooManyPixels.png"), wallpaper_png(4096, 2049)).unwrap();
    let large = fs::File::create(inbox.join("TooLarge.png")).unwrap();
    large.set_len(16 * 1024 * 1024 + 1).unwrap();
    let result = sharing::import(root.path()).unwrap();
    assert_eq!(result.wallpapers, 1);
    assert_eq!(result.skipped, 6);
    assert_eq!(slot::wallpaper::list(root.path()), ["Good.png"]);
    for name in [
        "Signature.png",
        "Truncated.png",
        "Corrupt.png",
        "TooWide.png",
        "TooManyPixels.png",
        "TooLarge.png",
    ] {
        assert!(inbox.join(name).exists(), "{name} must remain in Inbox");
    }
}

#[test]
fn interrupted_staging_files_do_not_block_later_game_or_wallpaper_imports() {
    let root = common::tmp_root_with_carts(&["Fusion"]);
    let old = format!(".slot-upload-{}", std::process::id());
    for folder in ["GBA", "Wallpapers"] {
        fs::create_dir_all(root.path().join("Inbox").join(folder)).unwrap();
    }
    fs::create_dir_all(root.path().join("Wallpapers")).unwrap();
    for folder in ["Games/GBA", "Wallpapers"] {
        fs::write(root.path().join(folder).join(&old), b"old partial upload").unwrap();
    }
    fs::write(root.path().join("Inbox/GBA/Recovered.gba"), [1; 1024]).unwrap();
    fs::write(
        root.path().join("Inbox/Wallpapers/Recovered.png"),
        wallpaper_png(4, 4),
    )
    .unwrap();
    fs::write(
        root.path().join("Inbox/Wallpapers/Corrupt.png"),
        b"invalid PNG",
    )
    .unwrap();
    let imported = sharing::import(root.path()).unwrap();
    assert_eq!(imported.added.len(), 1);
    assert_eq!(imported.wallpapers, 1);
    assert_eq!(imported.skipped, 1);
    for folder in ["Games/GBA", "Wallpapers"] {
        let dir = root.path().join(folder);
        assert_eq!(fs::read(dir.join(&old)).unwrap(), b"old partial upload");
        let hidden: Vec<_> = fs::read_dir(dir)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|name| name.starts_with(".slot-upload-"))
            .collect();
        assert_eq!(
            hidden,
            [old.clone()],
            "successful and rejected imports must clean up their own staging files"
        );
    }
}

#[cfg(unix)]
#[test]
fn wallpaper_imports_reject_source_target_and_directory_links() {
    use std::os::unix::fs::symlink;
    let root = tempfile::tempdir().unwrap();
    let inbox = root.path().join("Inbox/Wallpapers");
    let target = root.path().join("Wallpapers");
    fs::create_dir_all(&inbox).unwrap();
    fs::create_dir(&target).unwrap();
    fs::write(root.path().join("private.png"), wallpaper_png(4, 4)).unwrap();
    symlink(root.path().join("private.png"), inbox.join("source.png")).unwrap();
    fs::write(inbox.join("existing.png"), wallpaper_png(4, 4)).unwrap();
    symlink(root.path().join("missing.png"), target.join("existing.png")).unwrap();
    let result = sharing::import(root.path()).unwrap();
    assert_eq!(result.wallpapers, 0);
    assert_eq!(result.skipped, 2);
    assert!(target
        .join("existing.png")
        .symlink_metadata()
        .unwrap()
        .is_symlink());
    assert!(!target.join("source.png").exists());
    assert!(!root.path().join("missing.png").exists());
    fs::remove_file(target.join("existing.png")).unwrap();
    fs::remove_dir(&target).unwrap();
    fs::create_dir(root.path().join("private")).unwrap();
    symlink(root.path().join("private"), &target).unwrap();
    assert_eq!(sharing::import(root.path()).unwrap().wallpapers, 0);
    assert_eq!(
        fs::read_dir(root.path().join("private")).unwrap().count(),
        0
    );
    fs::remove_file(&target).unwrap();
    fs::remove_file(inbox.join("source.png")).unwrap();
    fs::remove_file(inbox.join("existing.png")).unwrap();
    fs::remove_dir(&inbox).unwrap();
    symlink(root.path().join("private"), &inbox).unwrap();
    fs::write(root.path().join("private/escape.png"), wallpaper_png(4, 4)).unwrap();
    assert_eq!(sharing::import(root.path()).unwrap().wallpapers, 0);
    assert!(!target.exists());
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
