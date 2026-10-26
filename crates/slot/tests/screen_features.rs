mod common;
use slot::{
    app::{App, Phase},
    frame_blending,
    kid_menu::Menu,
    kid_policy::Policy,
    reference,
    video_mode::{self, VideoMode},
};
use slot_input::{Action, Btn};
use slot_store::{Binding, CarouselKeys, Controls, Core, Device, Platform, Shortcut, ShortcutKeys};
use slot_ui::{QuickRow, QuickValue};

fn press(a: &mut App, b: Btn) {
    a.apply(Action::GbaDown(b));
    a.apply(Action::GbaUp(b));
}
fn screen_row(a: &mut App, want: QuickRow) {
    a.apply(Action::QuickMenu);
    for target in [QuickRow::Settings, QuickRow::PlayScreen, want] {
        for _ in 0..16 {
            if a.in_game_menu() == Some(target) {
                break;
            }
            press(a, Btn::Down);
        }
        assert_eq!(a.in_game_menu(), Some(target));
        if target != want {
            press(a, Btn::A);
        }
    }
}
#[test]
fn fit_preserves_aspect_and_integer_uses_whole_physical_pixels() {
    for panel in [
        (720, 480),
        (640, 480),
        (480, 320),
        (1280, 720),
        (1920, 1080),
    ] {
        for platform in Platform::ALL {
            let (sw, sh) = platform.picture();
            for mode in [VideoMode::Fit, VideoMode::Integer] {
                let [x, y, w, h] = mode.destination(platform, panel).unwrap();
                let (x, y, w, h) = (
                    x / 720.0 * panel.0 as f32,
                    y / 480.0 * panel.1 as f32,
                    w / 720.0 * panel.0 as f32,
                    h / 480.0 * panel.1 as f32,
                );
                assert!((w / h - sw as f32 / sh as f32).abs() < 0.0001);
                assert!(
                    x >= 0.0
                        && y >= 0.0
                        && x + w <= panel.0 as f32 + 0.001
                        && y + h <= panel.1 as f32 + 0.001
                );
                assert!(((panel.0 as f32 - w) / 2.0 - x).abs() < 1.001);
                if mode == VideoMode::Integer {
                    assert!((w / sw as f32 - (w / sw as f32).round()).abs() < 0.001);
                    assert!((h / sh as f32 - (h / sh as f32).round()).abs() < 0.001);
                }
            }
        }
    }
    assert_eq!(
        VideoMode::Integer.destination(Platform::Gba, (640, 480)),
        Some([90.0, 80.0, 540.0, 320.0])
    );
}
#[test]
fn modes_round_trip_and_crop_only_the_game_pixels() {
    let d = common::tmp_root_with_gb_carts(&["Tetris"]);
    for mode in [
        VideoMode::Actual,
        VideoMode::Stretch,
        VideoMode::Fit,
        VideoMode::Integer,
    ] {
        video_mode::write_video_mode(d.path(), "Tetris", mode).unwrap();
        assert_eq!(video_mode::video_mode_for(d.path(), "Tetris"), mode);
    }
    let crop = video_mode::source_rect(Platform::Gb, VideoMode::Fit);
    assert_eq!(
        crop,
        [40.0 / 240.0, 8.0 / 160.0, 160.0 / 240.0, 144.0 / 160.0]
    );
    assert_eq!(VideoMode::Actual.step(true).step(false), VideoMode::Actual);
}
#[test]
fn scaling_menu_saves_per_game_and_keeps_the_selected_rectangle_while_paused() {
    let d = common::tmp_root_with_carts(&["Alpha", "Beta"]);
    let mut a = common::app_playing_in(d.path(), "Alpha");
    a.set_panel((640, 480));
    screen_row(&mut a, QuickRow::Scaling);
    press(&mut a, Btn::Right);
    press(&mut a, Btn::Right);
    assert_eq!(a.quick_value(QuickRow::Scaling), Some(QuickValue::Fit));
    assert_eq!(
        video_mode::video_mode_for(d.path(), "Alpha"),
        VideoMode::Fit
    );
    assert_eq!(
        video_mode::video_mode_for(d.path(), "Beta"),
        VideoMode::Actual
    );
    let rect = a.game_destination();
    press(&mut a, Btn::B);
    assert_eq!(a.game_destination(), rect);
    assert!(matches!(a.phase(), Phase::Playing { .. }));
}
#[test]
fn blending_is_opt_in_persisted_per_system_and_uses_declared_core_options() {
    let d = common::tmp_root_with_carts(&["Alpha"]);
    for p in Platform::ALL {
        assert!(!frame_blending::read(d.path(), p));
    }
    let mut a = common::app_playing_in(d.path(), "Alpha");
    screen_row(&mut a, QuickRow::FrameBlending);
    press(&mut a, Btn::Right);
    assert_eq!(a.take_frame_blending(), Some(true));
    assert!(frame_blending::read(d.path(), Platform::Gba));
    assert!(!frame_blending::read(d.path(), Platform::Gb));
    assert_eq!(
        frame_blending::option(Core::Mgba, true),
        ("mgba_interframe_blending", "mix")
    );
    assert_eq!(
        frame_blending::option(Core::Gpsp, true),
        ("gpsp_frame_mixing", "enabled")
    );
    assert_eq!(frame_blending::option(Core::Mgba, false).1, "OFF");
    assert_eq!(frame_blending::option(Core::Gpsp, false).1, "disabled");
}
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
    press(&mut a, Btn::Up);
    assert_eq!(a.in_game_menu(), Some(QuickRow::PlayReference));
    press(&mut a, Btn::A);
    assert!(a.kid_menu.as_ref().unwrap().reference_open());
    press(&mut a, Btn::B);
    assert_eq!(a.in_game_menu(), Some(QuickRow::PlayReference));
}
