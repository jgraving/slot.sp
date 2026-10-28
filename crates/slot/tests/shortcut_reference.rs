mod common;
use slot::{app::App, kid_menu::Menu, kid_policy::Policy, reference};
use slot_input::{Action,Btn};
use slot_store::{Binding,CarouselKeys,Controls,Device,Platform,Shortcut,ShortcutKeys};
use slot_ui::QuickRow;
fn press(a:&mut App,b:Btn) { a.apply(Action::GbaDown(b));a.apply(Action::GbaUp(b)); }
#[test]
fn reference_uses_saved_bindings_and_child_pages_never_advertise_blocked_shortcuts() {
    let mut keys = ShortcutKeys::default();
    keys.set(Shortcut::SaveState, Binding::plain(Device::Y));
    let adult = reference::pages(
        false,
        true,
        Platform::Gba,
        Controls::default(),
        keys,
        CarouselKeys::default(),
    );
    assert!(
        adult
            .iter()
            .flat_map(|p| &p.1)
            .any(|r| r.0 == "Save State" && r.1 == "Y")
    );
    let kid = reference::pages(
        true,
        true,
        Platform::Gba,
        Controls::default(),
        keys,
        CarouselKeys::default(),
    );
    let text = format!("{kid:?}");
    for forbidden in [
        "Save State",
        "Load State",
        "Rewind",
        "Fast Forward",
        "Approve for kids",
        "Choose GBA emulator",
        "Turbo",
    ] {
        assert!(!text.contains(forbidden), "{forbidden}");
    }
    assert!(text.contains("Parent controls"));
    let d = common::tmp_root_with_carts(&["Alpha"]);
    let mut menu = Menu::child(d.path().into(), Policy::default(), true, 50, 50);
    menu.set_reference(kid);
    // Up wraps to Parent controls; one more Up reaches the reference without unlocking anything.
    menu.input(Action::GbaDown(Btn::Up), 0);
    menu.input(Action::GbaDown(Btn::Up), 0);
    menu.input(Action::GbaDown(Btn::A), 0);
    assert!(menu.reference_open());
    menu.input(Action::GbaDown(Btn::Right), 0);
    assert!(menu.lines().iter().any(|l| l == "Kid menu"));
    assert!(!menu.entering_code());
    assert!(menu.changed.is_none());
    let face = menu.screen_face();
    assert!(face.rgba.iter().any(|c| *c != 0));
    if let Ok(dir) = std::env::var("SLOT_FEATURE_PREVIEW") {
        std::fs::create_dir_all(&dir).unwrap();
        for (name, pages) in [
            (
                "kid",
                reference::pages(
                    true,
                    true,
                    Platform::Gba,
                    Controls::default(),
                    keys,
                    CarouselKeys::default(),
                ),
            ),
            ("adult", adult),
        ] {
            let mut m = Menu::reference(d.path().into(), Policy::default(), pages);
            for i in 0..7 {
                let f = m.screen_face();
                let file = std::fs::File::create(format!("{dir}/{name}-{i}.png")).unwrap();
                let mut e = png::Encoder::new(file, f.w, f.h);
                e.set_color(png::ColorType::Rgba);
                e.set_depth(png::BitDepth::Eight);
                e.write_header().unwrap().write_image_data(&f.rgba).unwrap();
                m.input(Action::GbaDown(Btn::Right), 0);
            }
        }
    }
    menu.input(Action::GbaDown(Btn::B), 0);
    assert!(menu.child_menu());
    assert!(menu.changed.is_none());
}
#[test]
fn adult_reference_is_reachable_from_both_menus_and_returns_without_changing_settings() {
    let d = common::tmp_root_with_carts(&["Alpha", "Beta"]);
    common::clocked(d.path());
    let mut a = App::boot(d.path());
    a.apply(Action::QuickMenu);
    for _ in 0..12 {
        if a.quick_menu() == Some(QuickRow::Controls) {
            break;
        }
        press(&mut a, Btn::Down);
    }
    press(&mut a, Btn::A);
    press(&mut a, Btn::Up);
    press(&mut a, Btn::Up);
    assert_eq!(a.quick_menu(), Some(QuickRow::Reference));
    press(&mut a, Btn::A);
    assert!(a.kid_menu.as_ref().unwrap().reference_open());
    press(&mut a, Btn::B);
    assert!(a.kid_menu.is_none());
    assert_eq!(a.quick_menu(), Some(QuickRow::Reference));
    let mut a = common::app_playing_in(d.path(), "Alpha");
    a.apply(Action::QuickMenu);
    for _ in 0..12 {
        if a.in_game_menu() == Some(QuickRow::Settings) {
            break;
        }
        press(&mut a, Btn::Down);
    }
    press(&mut a, Btn::A);
    for _ in 0..QuickRow::SETTINGS.len() {
        if a.in_game_menu() == Some(QuickRow::PlayReference) { break; }
        press(&mut a, Btn::Up);
    }
    assert_eq!(a.in_game_menu(), Some(QuickRow::PlayReference));
    press(&mut a, Btn::A);
    assert!(a.kid_menu.as_ref().unwrap().reference_open());
    press(&mut a, Btn::B);
    assert_eq!(a.in_game_menu(), Some(QuickRow::PlayReference));
}
