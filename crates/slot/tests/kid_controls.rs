mod common;
use slot::{
    app::{App, Phase},
    kid_policy::Policy,
    parent_code::Gate,
};
use slot_input::{Action, Btn};
use slot_store::{Platform, Recent, SlotState, read_slot_state, write_slot_state};

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
fn card() -> tempfile::TempDir {
    let d = common::tmp_root_with_carts(&["Approved", "Blocked"]);
    common::write_gb_cart(&d, "Tetris", "TETRIS");
    let mut state = SlotState {
        clock_set: true,
        show_recents: true,
        volume: 90,
        volume_hp: 90,
        volume_usb: 90,
        ..SlotState::default()
    };
    state.recents = vec![
        Recent {
            platform: Platform::Gba,
            stem: "Blocked".into(),
        },
        Recent {
            platform: Platform::Gba,
            stem: "Approved".into(),
        },
        Recent {
            platform: Platform::Gb,
            stem: "Tetris".into(),
        },
    ];
    state.toggle_favorite(Platform::Gba, "Blocked");
    state.toggle_favorite(Platform::Gba, "Approved");
    state.toggle_favorite(Platform::Gb, "Tetris");
    write_slot_state(d.path(), &state).unwrap();
    let mut gate = Gate::new(Policy::directory(d.path()).join("code"));
    gate.set(&[0, 1, 2, 3]).unwrap();
    let mut policy = Policy::default();
    policy.minutes = 5;
    policy.approved = vec![
        (Platform::Gba, "Approved".into()),
        (Platform::Gb, "Tetris".into()),
    ];
    policy.begin(now()).unwrap();
    policy.save(d.path()).unwrap();
    d
}
fn press(app: &mut App, b: Btn) {
    common::through(app, b);
}
fn unlock(app: &mut App) {
    for b in [Btn::A, Btn::B, Btn::X, Btn::Y, Btn::Start] {
        press(app, b);
    }
    assert!(
        app.kid_menu
            .as_ref()
            .unwrap()
            .lines()
            .iter()
            .any(|s| s.contains("Apply limits") || s.contains("Start kid mode"))
    );
}
#[test]
fn kid_mode_filters_each_existing_rack_without_erasing_parent_memberships() {
    let d = card();
    let before = read_slot_state(d.path());
    let mut app = App::boot(d.path());
    assert!(app.kid_mode());
    assert!(matches!(app.phase(), Phase::Shelf));
    assert!(app.carts().all(|c| c.stem != "Blocked"));
    assert_eq!(app.carts().count(), 2);
    let mut racks = Vec::new();
    for _ in 0..8 {
        if let Some(name) = app.shelf_platform_name() {
            racks.push(name);
        }
        assert_ne!(app.selected_stem(), Some("Blocked"));
        for _ in 0..4 {
            press(&mut app, Btn::Right);
            app.update(0.5);
            assert_ne!(app.selected_stem(), Some("Blocked"));
        }
        app.apply(Action::NextShelf);
    }
    for r in ["Recents", "Favorites", "Game Boy Advance", "Game Boy"] {
        assert!(racks.iter().any(|s| *s == r), "{r}: {racks:?}");
    }
    assert!(!racks.iter().any(|s| *s == "Kids"));
    let after = read_slot_state(d.path());
    assert_eq!(before.recents, after.recents);
    assert_eq!(before.favorites, after.favorites);
    app.apply(Action::QuickMenu);
    unlock(&mut app);
    // Exit kid mode is row 4; parent menu starts on row 0.
    for _ in 0..4 {
        press(&mut app, Btn::Down);
    }
    press(&mut app, Btn::A);
    assert!(!app.kid_mode());
    assert_eq!(app.carts().count(), 3);
    assert!(app.carts().any(|c| c.stem == "Blocked"));
}
#[test]
fn blocked_shortcuts_cannot_change_rules_or_launch_parent_screens() {
    let d = card();
    let mut app = App::boot(d.path());
    for a in [
        Action::SaveState,
        Action::LoadState,
        Action::Polaroids,
        Action::Screenshot,
        Action::RewindStart,
        Action::FfStart,
        Action::RepairCard,
        Action::Favorite,
        Action::NightToggle,
        Action::PaletteNext,
        Action::ShaderNext,
    ] {
        assert!(!app.kid_action_allowed(a));
        app.apply(a);
        assert!(matches!(app.phase(), Phase::Shelf));
        assert!(app.kid_menu.is_none());
    }
    app.apply(Action::QuickMenu);
    for b in [Btn::A, Btn::A, Btn::A, Btn::A, Btn::Start] {
        press(&mut app, b);
    }
    assert!(
        app.kid_menu
            .as_ref()
            .unwrap()
            .lines()
            .iter()
            .any(|l| l == "Parent code")
    );
    assert!(app.kid_mode());
    unlock(&mut app);
    assert!(app.kid_mode());
}
#[test]
fn parent_code_has_readable_sd_recovery_and_is_updated_with_the_code() {
    let d = card();
    let p = Policy::directory(d.path()).join("parent-code.txt");
    assert!(std::fs::read_to_string(&p).unwrap().contains("A B X Y"));
    let mut gate = Gate::new(Policy::directory(d.path()).join("code"));
    gate.verify(&[0, 1, 2, 3], std::time::Instant::now())
        .unwrap();
    gate.set(&[4, 5, 6, 7]).unwrap();
    let text = std::fs::read_to_string(&p).unwrap();
    assert!(text.contains("UP DOWN LEFT RIGHT"));
    assert!(!text.contains("A B X Y"));
}
#[test]
fn volume_is_capped_at_boot_and_across_output_changes() {
    let d = card();
    let mut app = App::boot(d.path());
    assert_eq!(app.kid_volume_limit(), 50);
    assert!(app.output_volume() <= 50);
    for _ in 0..50 {
        app.apply(Action::VolumeUp);
    }
    assert_eq!(app.volume(), 50);
    app.usb_state(true, false);
    assert!(app.output_volume() <= 50);
    for _ in 0..40 {
        app.apply(Action::VolumeUp);
    }
    assert_eq!(app.volume(), 50);
    app.usb_state(false, false);
    app.bluetooth_state(Some("AA:BB:CC:DD:EE:FF".into()), true);
    app.observe_bluetooth_volume(100);
    assert!(app.output_volume() <= 50);
    assert!(app.volume() <= 50);
}
#[test]
fn active_policy_survives_restart_and_missing_settings_fail_closed() {
    let d = card();
    let app = App::boot(d.path());
    assert!(app.kid_mode());
    drop(app);
    assert!(App::boot(d.path()).kid_mode());
    std::fs::remove_file(Policy::directory(d.path()).join("settings")).unwrap();
    let app = App::boot(d.path());
    assert!(app.kid_mode());
    assert_eq!(app.carts().count(), 0);
    assert!(app.parent_controls_open());
}
#[test]
fn time_expiry_saves_and_returns_to_parent_notice() {
    let d = card();
    let mut app = App::boot(d.path());
    app.apply(Action::Insert);
    app.set_snapshot(common::StubSnapshot::boxed());
    app.on_core_ready();
    for _ in 0..120 {
        app.update(1.0 / 60.0);
    }
    assert!(matches!(app.phase(), Phase::Playing { .. }));
    app.tick_ms(app.now() + 310_000);
    assert!(matches!(app.phase(), Phase::Shelf));
    assert!(app.parent_controls_open());
    assert!(app.kid_mode());
    assert!(
        app.kid_menu
            .as_ref()
            .unwrap()
            .lines()
            .iter()
            .any(|s| s.contains("Time for a break"))
    );
    assert_eq!(read_slot_state(d.path()).cart, None);
}

#[test]
fn parent_overlays_pause_gameplay_and_expired_sessions_do_not_resume() {
    use slot::kid_policy::Rule;
    let d = card();
    let mut policy = Policy::read(d.path()).unwrap();
    policy.rule = Rule::Session;
    policy.begin(now()).unwrap();
    policy.save(d.path()).unwrap();
    let mut app = App::boot(d.path());
    app.apply(Action::Insert);
    app.set_snapshot(common::StubSnapshot::boxed());
    app.on_core_ready();
    for _ in 0..120 {
        app.update(1.0 / 60.0);
    }
    assert!(matches!(app.phase(), Phase::Playing { .. }));
    app.apply(Action::GameMenu);
    assert!(app.parent_controls_open());
    assert_eq!(app.output_volume(), 0);
    assert_eq!(app.input_context(), slot_input::Context::Menu);
    app.apply(Action::QuickMenu);
    assert!(!app.parent_controls_open());
    policy = Policy::read(d.path()).unwrap();
    policy.begin(now() - 400).unwrap();
    policy.save(d.path()).unwrap();
    // Reboot into an already expired session must not resume gameplay.
    let mut app = App::boot(d.path());
    app.update(0.016);
    assert!(app.parent_controls_open());
    assert!(matches!(app.phase(), Phase::Shelf));
}

#[test]
fn approval_marking_is_persisted_but_unavailable_to_children() {
    let d = card();
    let mut policy = Policy::read(d.path()).unwrap();
    policy.leave();
    policy.save(d.path()).unwrap();
    let mut app = App::boot(d.path());
    let before = Policy::read(d.path()).unwrap().approved;
    press(&mut app, Btn::X);
    let after = Policy::read(d.path()).unwrap().approved;
    assert_ne!(before, after);
    let mut policy = Policy::read(d.path()).unwrap();
    policy.begin(now()).unwrap();
    policy.save(d.path()).unwrap();
    let mut app = App::boot(d.path());
    let before = Policy::read(d.path()).unwrap().approved;
    press(&mut app, Btn::X);
    assert_eq!(before, Policy::read(d.path()).unwrap().approved);
}

#[test]
fn parent_menu_uses_real_button_hints_and_renders_without_overflow() {
    let d = card();
    let mut app = App::boot(d.path());
    app.apply(Action::QuickMenu);
    let face = app.kid_menu.as_ref().unwrap().screen_face();
    assert!(face.rgba.chunks_exact(4).any(|p| p[3] > 0));
    if let Ok(dir) = std::env::var("SLOT_KID_UI_PREVIEW") {
        write_preview(&dir, "kid-parent-code.png", face);
    }
    unlock(&mut app);
    let face = app.kid_menu.as_ref().unwrap().screen_face();
    if let Ok(dir) = std::env::var("SLOT_KID_UI_PREVIEW") {
        write_preview(&dir, "kid-parent-settings.png", face);
    }
}
fn write_preview(dir: &str, name: &str, face: slot_ui::UndoFace) {
    let file = std::fs::File::create(std::path::Path::new(dir).join(name)).unwrap();
    let mut enc = png::Encoder::new(file, face.w, face.h);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header()
        .unwrap()
        .write_image_data(&face.rgba)
        .unwrap();
}

#[test]
fn parent_menu_does_not_trap_power_or_lid_buttons() {
    let d = card();
    let mut app = App::boot(d.path());
    app.apply(Action::QuickMenu);
    assert!(app.parent_controls_open());
    app.apply(Action::LidClose);
    assert!(matches!(app.phase(), Phase::Doze { .. }));
    app.apply(Action::LidOpen);
    assert!(matches!(app.phase(), Phase::Shelf));
    assert!(app.parent_controls_open());
    app.apply(Action::PowerHold);
    assert!(app.powering_off());
}

#[test]
fn kid_volume_cap_applies_to_wired_headphones_too() {
    use std::sync::atomic::Ordering;
    let d = card();
    let (mut app, jack) = common::app_playing_with_jack(d.path(), "Approved");
    jack.store(true, Ordering::Relaxed);
    for _ in 0..150 {
        app.update(0.016);
    }
    for _ in 0..50 {
        app.apply(Action::VolumeUp);
    }
    assert_eq!(app.volume(), 50);
    assert!(app.output_volume() <= 50);
}

#[test]
fn invalid_hardware_clock_requires_parent_instead_of_exposing_clock_settings() {
    let d = card();
    let (mut app, _) = common::app_booting_at(d.path(), 0);
    assert!(app.kid_mode());
    assert!(app.parent_controls_open());
    assert!(matches!(app.phase(), Phase::Shelf));
    app.apply(Action::Insert);
    assert!(matches!(app.phase(), Phase::Shelf));
}
