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
    if app.kid_menu.as_ref().unwrap().child_menu() {
        press(app, Btn::Up);
        press(app, Btn::A);
    }
    for b in [Btn::A, Btn::B, Btn::X, Btn::Y] {
        press(app, b);
    }
    assert!(
        app.kid_menu
            .as_ref()
            .unwrap()
            .lines()
            .iter()
            .any(|s| s.contains("Kid mode On") || s.contains("Kid mode Off"))
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
    // The top row changes kid mode immediately.
    press(&mut app, Btn::A);
    assert!(!app.kid_mode());
    assert!(app.parent_controls_open());
    press(&mut app, Btn::B);
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
    press(&mut app, Btn::Up);
    press(&mut app, Btn::A);
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
    app.update(0.4);
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
    if app.kid_removal_pending() {
        assert_eq!(Policy::read(d.path()).unwrap().approved, before);
        press(&mut app, Btn::X);
    }
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

#[test]
fn wake_checks_session_and_bedtime_before_any_update() {
    use slot::kid_policy::Rule;
    for rule in [Rule::Session, Rule::Until] {
        let d = card();
        let wall = now();
        let mut policy = Policy::read(d.path()).unwrap();
        policy.rule = rule;
        // Give bedtime enough room even if this test runs near midnight.
        let start = wall.div_euclid(86_400) * 86_400 + 12 * 3600;
        policy.bedtime = 12 * 60 + 5;
        policy.begin(start).unwrap();
        policy.save(d.path()).unwrap();
        let (mut app, clock) = common::app_booting_at(d.path(), start);
        app.apply(Action::Insert);
        app.set_snapshot(common::StubSnapshot::boxed());
        app.on_core_ready();
        for _ in 0..120 {
            app.update(1.0 / 60.0);
        }
        assert!(matches!(app.phase(), Phase::Playing { .. }));
        app.apply(Action::LidClose);
        assert!(matches!(app.phase(), Phase::Doze { .. }));
        clock.advance(301);
        app.apply(Action::LidOpen);
        assert!(matches!(app.phase(), Phase::Shelf), "{rule:?}");
        assert!(app.parent_controls_open());
        assert_eq!(app.output_volume(), 0);
    }
}

#[test]
fn daily_allowance_does_not_count_suspend_without_intervening_frames() {
    let d = card();
    let (mut app, clock) = common::app_booting_at(d.path(), now());
    app.apply(Action::Insert);
    app.set_snapshot(common::StubSnapshot::boxed());
    app.on_core_ready();
    for _ in 0..120 {
        app.update(1.0 / 60.0);
    }
    assert!(matches!(app.phase(), Phase::Playing { .. }));
    let before = app.kids.remaining(now());
    app.apply(Action::LidClose);
    clock.advance(600);
    // Advance monotonic time without running a frame while asleep.
    app.tick_ms(app.now() + 600_000);
    app.apply(Action::LidOpen);
    assert!(matches!(app.phase(), Phase::Playing { .. }));
    assert_eq!(app.kids.remaining(now() + 600), before);
}

// Exercise the production Session::feed path, with one persistent gesture state.
// Direct App::apply tests cannot catch swallowed/reordered physical button edges.
fn raw_tap(s: &mut slot::session::Session, time: &mut u64, button: Btn) {
    use slot_input::RawEvent;
    *time += 20;
    s.feed([RawEvent::Down(button)], *time);
    *time += 20;
    s.feed([RawEvent::Up(button)], *time);
}
fn preview_menu(s: &slot::session::Session, name: &str) {
    if let Ok(dir) = std::env::var("SLOT_KID_UI_PREVIEW") {
        write_preview(&dir, name, s.app().kid_menu.as_ref().unwrap().screen_face());
    }
}
#[test]
fn raw_parent_setup_shows_settings_records_buttons_and_requires_matching_confirmation() {
    use slot::session::Session;
    let d = card();
    let dir = Policy::directory(d.path());
    std::fs::remove_dir_all(&dir).unwrap();
    let mut s = Session::boot(d.path().to_owned());
    let mut time = 0;
    s.app_mut().apply(Action::QuickMenu);
    for _ in 0..12 {
        if s.app().quick_menu() == Some(slot_ui::QuickRow::Kids) { break; }
        raw_tap(&mut s, &mut time, Btn::Down);
    }
    assert_eq!(s.app().quick_menu(), Some(slot_ui::QuickRow::Kids));
    raw_tap(&mut s, &mut time, Btn::A);
    assert!(!s.app().kid_menu.as_ref().unwrap().entering_code());
    assert!(s.app().kid_menu.as_ref().unwrap().lines().iter().any(|l| l.contains("Volume limit")));
    preview_menu(&s, "parent-first-settings.png");
    // Select Parent code. Merely opening settings must not create a code.
    assert!(!dir.join("code").exists());
    for _ in 0..4 { raw_tap(&mut s, &mut time, Btn::Down); }
    raw_tap(&mut s, &mut time, Btn::A);
    assert!(s.app().kid_menu.as_ref().unwrap().entering_code());
    let empty = s.app().kid_menu.as_ref().unwrap().screen_face().rgba;
    for (i, b) in [Btn::A, Btn::B].into_iter().enumerate() {
        raw_tap(&mut s, &mut time, b);
        let menu = s.app().kid_menu.as_ref().unwrap();
        assert_eq!(menu.entered_buttons().len(), i + 1);
        assert!(menu.lines()[1].contains(&format!("{} of 4", i + 1)));
        assert_ne!(menu.screen_face().rgba, empty);
        let face = menu.screen_face();
        let x0 = (face.w - 320) / 2 + i as u32 * 80;
        let visible = (208..272).any(|y| (x0..x0 + 80).any(|x| {
            let at = ((y * face.w + x) * 4) as usize;
            face.rgba[at + 3] > 0 && face.rgba[at..at + 3].iter().any(|c| *c > 150)
        }));
        assert!(visible, "entered button slot {i} is transparent or invisible");
        preview_menu(&s, &format!("parent-code-{}.png", i + 1));
    }
    raw_tap(&mut s, &mut time, Btn::X);
    raw_tap(&mut s, &mut time, Btn::Y);
    assert_eq!(s.app().kid_menu.as_ref().unwrap().lines()[0], "Repeat your code");
    assert!(!dir.join("code").exists());
    preview_menu(&s, "parent-confirm-empty.png");
    for b in [Btn::A; 4] { raw_tap(&mut s, &mut time, b); }
    assert!(!dir.join("code").exists(), "mismatched confirmation created a code");
    assert_eq!(s.app().kid_menu.as_ref().unwrap().entered_buttons(), &[0; 4]);
    s.app_mut().update(0.4);
    assert!(s.app().kid_menu.as_ref().unwrap().entered_buttons().is_empty());
    assert_eq!(s.app().kid_menu.as_ref().unwrap().lines()[0], "Repeat your code");
    for b in [Btn::A, Btn::B, Btn::X, Btn::Y] { raw_tap(&mut s, &mut time, b); }
    assert!(dir.join("code").exists());
    assert!(!s.app().kid_mode());
    assert!(!s.app().kid_menu.as_ref().unwrap().entering_code());
    let mut gate = Gate::new(dir.join("code"));
    assert!(gate.verify(&[0, 1, 2, 3], std::time::Instant::now()).unwrap());
    assert!(std::fs::read_to_string(dir.join("parent-code.txt")).unwrap().contains("A B X Y"));
}

#[test]
fn raw_unlock_and_delete_accept_dpad_and_ignore_repeated_down_edges() {
    use slot::{session::Session, kid_menu::Menu};
    use slot_input::RawEvent;
    let d = card();
    let mut gate = Gate::new(Policy::directory(d.path()).join("code"));
    gate.verify(&[0, 1, 2, 3], std::time::Instant::now()).unwrap();
    gate.set(&[4, 5, 6, 7]).unwrap();
    let mut s = Session::boot(d.path().to_owned());
    let policy = s.app().kids.clone();
    s.app_mut().kid_menu = Some(Menu::new(d.path().to_owned(), policy));
    let mut time = 0;
    s.feed([RawEvent::Down(Btn::A), RawEvent::Down(Btn::A)], time);
    assert_eq!(s.app().kid_menu.as_ref().unwrap().entered_buttons(), &[0]);
    s.feed([RawEvent::Up(Btn::A)], time);
    raw_tap(&mut s, &mut time, Btn::Select);
    assert!(s.app().kid_menu.as_ref().unwrap().entered_buttons().is_empty());
    for b in [Btn::Up, Btn::Down, Btn::Left] { raw_tap(&mut s, &mut time, b); }
    assert_eq!(s.app().kid_menu.as_ref().unwrap().entered_buttons(), &[4, 5, 6]);
    preview_menu(&s, "parent-dpad-code.png");
    raw_tap(&mut s, &mut time, Btn::Right);
    assert!(!s.app().kid_menu.as_ref().unwrap().entering_code());
    assert!(s.app().kid_mode(), "unlock alone must not disable kid mode");
}

#[test]
fn parent_settings_start_with_kid_mode_and_leave_approvals_on_the_carousel() {
    let d = card();
    let mut app = App::boot(d.path());
    app.apply(Action::QuickMenu);
    unlock(&mut app);
    let lines = app.kid_menu.as_ref().unwrap().lines();
    assert_eq!(lines[3], "SELECTED Kid mode On");
    assert_eq!(lines.iter().filter(|l| l.starts_with("ROW ") || l.starts_with("SELECTED ")).count(), 5);
    assert!(!lines.iter().any(|l| l.starts_with("ROW Approved games")));
    press(&mut app, Btn::A);
    press(&mut app, Btn::B);
    let before = app.kids.approved.clone();
    app.apply(Action::GbaDown(Btn::X));
    app.apply(Action::GbaUp(Btn::X));
    if app.kids.approved == before { app.apply(Action::GbaDown(Btn::X)); }
    assert_ne!(app.kids.approved, before);
}

#[test]
fn approval_removal_requires_release_and_a_second_press_and_cancels_on_navigation() {
    let d = card();
    let mut policy = Policy::read(d.path()).unwrap();
    policy.leave();
    policy.approved.push((Platform::Gba, "Blocked".into()));
    policy.save(d.path()).unwrap();
    let mut app = App::boot(d.path());
    let before = Policy::read(d.path()).unwrap().approved;
    for _ in 0..10 { app.apply(Action::GbaDown(Btn::X)); }
    assert!(app.kid_removal_pending());
    assert_eq!(Policy::read(d.path()).unwrap().approved, before);
    app.apply(Action::GbaUp(Btn::X));
    app.apply(Action::ShelfRight);
    assert!(!app.kid_removal_pending());
    assert_eq!(Policy::read(d.path()).unwrap().approved, before);
}

#[test]
fn approval_removal_timeout_keeps_the_game_and_restarts_confirmation() {
    let d = card();
    let mut policy = Policy::read(d.path()).unwrap();
    policy.leave();
    policy.approved.push((Platform::Gba, "Blocked".into()));
    policy.save(d.path()).unwrap();
    let mut app = App::boot(d.path());
    let before = Policy::read(d.path()).unwrap().approved;
    press(&mut app, Btn::X);
    assert!(app.kid_removal_pending());
    app.update(3.1);
    assert!(!app.kid_removal_pending());
    press(&mut app, Btn::X);
    assert!(app.kid_removal_pending());
    assert_eq!(Policy::read(d.path()).unwrap().approved, before);
    press(&mut app, Btn::X);
    assert_eq!(Policy::read(d.path()).unwrap().approved.len(), before.len() - 1);
}

#[test]
fn kid_pause_menu_allows_volume_brightness_resume_and_exit_without_a_code() {
    use slot::session::Session;
    let d = card();
    let mut s = Session::boot(d.path().to_owned());
    s.app_mut().apply(Action::Insert);
    s.app_mut().set_snapshot(common::StubSnapshot::boxed());
    s.app_mut().on_core_ready();
    for _ in 0..120 { s.app_mut().update(1.0 / 60.0); }
    assert!(matches!(s.app().phase(), Phase::Playing { .. }));
    s.app_mut().apply(Action::QuickMenu);
    let menu = s.app().kid_menu.as_ref().unwrap();
    assert!(menu.child_menu());
    assert!(!menu.entering_code());
    assert!(menu.lines().iter().any(|l| l.contains("Exit game")));
    preview_menu(&s, "kid-pause-menu.png");
    let mut time = 0;
    raw_tap(&mut s, &mut time, Btn::Down);
    for _ in 0..30 { raw_tap(&mut s, &mut time, Btn::Right); }
    assert_eq!(s.app().volume(), s.app().kid_volume_limit());
    raw_tap(&mut s, &mut time, Btn::Left);
    assert!(s.app().volume() < s.app().kid_volume_limit());
    raw_tap(&mut s, &mut time, Btn::Down);
    let brightness = s.app().brightness();
    raw_tap(&mut s, &mut time, Btn::Left);
    assert_eq!(s.app().brightness(), brightness.saturating_sub(1));
    raw_tap(&mut s, &mut time, Btn::B);
    assert!(s.app().kid_menu.is_none());
    assert!(matches!(s.app().phase(), Phase::Playing { .. }));
    s.app_mut().apply(Action::QuickMenu);
    for _ in 0..3 { raw_tap(&mut s, &mut time, Btn::Down); }
    raw_tap(&mut s, &mut time, Btn::A);
    assert!(s.app().kid_menu.is_none());
    assert!(matches!(s.app().phase(), Phase::Ejecting { .. } | Phase::Shelf));
    assert!(s.app().kid_mode());
}

#[test]
fn parent_settings_apply_volume_immediately_and_use_transparent_menu_artwork() {
    let d = card();
    let mut app = App::boot(d.path());
    app.apply(Action::QuickMenu);
    unlock(&mut app);
    for _ in 0..3 { press(&mut app, Btn::Down); }
    press(&mut app, Btn::Left);
    assert_eq!(app.kid_volume_limit(), 40);
    assert_eq!(Policy::read(d.path()).unwrap().volume_limit, 40);
    let face = app.kid_menu.as_ref().unwrap().screen_face();
    assert_eq!(face.rgba[3], 0, "menu background must remain transparent");
    assert!(face.rgba.chunks_exact(4).filter(|p| p[3] == 0).count() > (face.w * face.h / 2) as usize);
    if let Ok(dir) = std::env::var("SLOT_KID_UI_PREVIEW") {
        write_preview(&dir, "parent-volume-arrows.png", face);
    }
    press(&mut app, Btn::B);
    assert!(app.kid_mode());
    assert_eq!(app.kid_volume_limit(), 40);
}

#[test]
fn kid_can_exit_when_only_one_game_is_approved() {
    let d = card();
    let mut policy = Policy::read(d.path()).unwrap();
    policy.approved = vec![(Platform::Gba, "Approved".into())];
    policy.save(d.path()).unwrap();
    let mut app = App::boot(d.path());
    assert_eq!(app.carts().count(), 1);
    app.apply(Action::Insert);
    app.set_snapshot(common::StubSnapshot::boxed());
    app.on_core_ready();
    for _ in 0..120 { app.update(1.0 / 60.0); }
    assert!(matches!(app.phase(), Phase::Playing { .. }));
    app.apply(Action::QuickMenu);
    for _ in 0..3 { press(&mut app, Btn::Down); }
    press(&mut app, Btn::A);
    assert!(matches!(app.phase(), Phase::Ejecting { .. }));
    for _ in 0..120 { app.update(1.0 / 60.0); }
    assert!(matches!(app.phase(), Phase::Shelf));
    assert!(app.kid_mode());
}

#[test]
fn parent_can_adjust_the_volume_cap_and_resume_the_same_game() {
    let d = card();
    let mut app = App::boot(d.path());
    app.apply(Action::Insert);
    app.set_snapshot(common::StubSnapshot::boxed());
    app.on_core_ready();
    for _ in 0..120 { app.update(1.0 / 60.0); }
    assert!(matches!(app.phase(), Phase::Playing { .. }));
    app.apply(Action::QuickMenu);
    unlock(&mut app);
    for _ in 0..3 { press(&mut app, Btn::Down); }
    press(&mut app, Btn::Left);
    assert_eq!(app.kid_volume_limit(), 40);
    assert!(matches!(app.phase(), Phase::Playing { .. }));
    press(&mut app, Btn::B);
    assert!(app.kid_menu.is_none());
    assert!(matches!(app.phase(), Phase::Playing { .. }));
    assert!(app.output_volume() <= 40);
}

fn warning_game(rule: slot::kid_policy::Rule, minutes: u32) -> (tempfile::TempDir, App, common::Clock) {
    let d = card();
    let start = now().div_euclid(86_400) * 86_400 + 12 * 3600;
    let mut policy = Policy::read(d.path()).unwrap();
    policy.rule = rule;
    policy.minutes = minutes;
    policy.bedtime = 12 * 60 + minutes;
    policy.begin(start).unwrap();
    policy.save(d.path()).unwrap();
    let (mut app, clock) = common::app_booting_at(d.path(), start);
    app.apply(Action::Insert);
    app.set_snapshot(common::StubSnapshot::boxed());
    app.on_core_ready();
    for _ in 0..120 { app.update(1.0 / 60.0); }
    assert!(matches!(app.phase(), Phase::Playing { .. }));
    (d, app, clock)
}

#[test]
fn time_warning_pauses_mutes_and_accepts_a_real_continue_press_without_a_code() {
    use slot::{kid_policy::Rule, session::Session};
    let (d, mut app, clock) = warning_game(Rule::Session, 120);
    clock.advance(3600);
    app.update(0.016);
    assert!(matches!(app.phase(), Phase::Playing { .. }));
    assert_eq!(app.output_volume(), 0);
    assert_eq!(app.input_context(), slot_input::Context::Menu);
    let menu = app.kid_menu.as_ref().unwrap();
    assert!(menu.time_warning());
    assert_eq!(menu.lines(), ["1 hour left"]);
    if let Ok(dir) = std::env::var("SLOT_KID_UI_PREVIEW") {
        write_preview(&dir, "kid-warning-hour.png", menu.screen_face());
    }
    let mut session = Session::boot(d.path().to_owned());
    *session.app_mut() = app;
    let mut time = 0;
    raw_tap(&mut session, &mut time, Btn::A);
    assert!(session.app().kid_menu.is_none());
    assert_eq!(session.app().input_context(), slot_input::Context::Game);
    assert!(session.app().output_volume() > 0);
    session.app_mut().update(0.016);
    assert!(session.app().kid_menu.is_none(), "same warning repeated after Continue");
    session.app_mut().apply(Action::QuickMenu);
    let menu = session.app().kid_menu.as_ref().unwrap();
    assert!(menu.child_menu());
    assert_eq!(menu.remaining_seconds(), Some(3600));
    preview_menu(&session, "kid-time-meter.png");
}

#[test]
fn daily_allowance_freezes_during_warnings_menus_and_sleep() {
    use slot::kid_policy::Rule;
    let (_d, mut app, clock) = warning_game(Rule::Daily, 5);
    app.tick_ms(app.now() + 240_000);
    assert!(app.kid_menu.as_ref().unwrap().time_warning());
    let before = app.kids.remaining(clock.get());
    clock.advance(120);
    app.tick_ms(app.now() + 120_000);
    assert_eq!(app.kids.remaining(clock.get()), before);
    press(&mut app, Btn::A);
    assert!(app.kid_menu.is_none());
    app.tick_ms(app.now());
    app.tick_ms(app.now() + 1000);
    assert_eq!(app.kids.remaining(clock.get()), before - 1);
    app.apply(Action::QuickMenu);
    clock.advance(180);
    app.tick_ms(app.now() + 180_000);
    assert_eq!(app.kids.remaining(clock.get()), before - 1);
    assert_eq!(app.kid_menu.as_ref().unwrap().remaining_seconds(), Some(before - 1));
    press(&mut app, Btn::B);
    app.apply(Action::LidClose);
    clock.advance(180);
    app.tick_ms(app.now() + 180_000);
    app.apply(Action::LidOpen);
    assert_eq!(app.kids.remaining(clock.get()), before - 1);
    assert!(app.kid_menu.is_none());
}

#[test]
fn wake_shows_one_current_warning_and_a_fixed_deadline_expires_while_it_is_open() {
    use slot::kid_policy::Rule;
    for rule in [Rule::Session, Rule::Until] {
        let (_d, mut app, clock) = warning_game(rule, 120);
        app.apply(Action::LidClose);
        clock.advance(6600);
        app.apply(Action::LidOpen);
        let menu = app.kid_menu.as_ref().unwrap();
        assert!(menu.time_warning());
        assert_eq!(menu.lines(), ["10 minutes left"]);
        clock.advance(601);
        app.update(0.016);
        assert!(matches!(app.phase(), Phase::Shelf));
        assert!(!app.kid_menu.as_ref().unwrap().time_warning());
        assert_eq!(app.kid_menu.as_ref().unwrap().lines()[0], "Time for a break");
        app.apply(Action::Insert);
        assert!(matches!(app.phase(), Phase::Shelf));
    }
}

#[test]
fn warning_allows_saving_and_exiting_without_a_parent_code() {
    use slot::kid_policy::Rule;
    let (_d, mut app, clock) = warning_game(Rule::Session, 120);
    clock.advance(6300);
    app.update(0.016);
    assert!(app.kid_menu.as_ref().unwrap().time_warning());
    if let Ok(dir) = std::env::var("SLOT_KID_UI_PREVIEW") {
        write_preview(&dir, "kid-warning-15min.png", app.kid_menu.as_ref().unwrap().screen_face());
    }
    press(&mut app, Btn::B);
    assert!(app.kid_menu.is_none());
    assert!(matches!(app.phase(), Phase::Ejecting { .. }));
    assert!(app.kid_mode());
}

#[test]
fn settings_meter_is_present_only_while_kid_mode_is_on() {
    let d = card();
    let mut app = App::boot(d.path());
    app.apply(Action::QuickMenu);
    unlock(&mut app);
    assert!(app.kid_menu.as_ref().unwrap().remaining_seconds().is_some());
    let active = app.kid_menu.as_ref().unwrap().screen_face();
    let band = |face: &slot_ui::UndoFace| (342..366).any(|y|
        (0..face.w).any(|x| face.rgba[((y * face.w + x) * 4 + 3) as usize] != 0));
    assert!(band(&active));
    if let Ok(dir) = std::env::var("SLOT_KID_UI_PREVIEW") {
        write_preview(&dir, "parent-time-meter.png", active);
    }
    press(&mut app, Btn::A);
    assert!(!app.kid_mode());
    if let Ok(dir) = std::env::var("SLOT_KID_UI_PREVIEW") {
        write_preview(&dir, "parent-mode-off.png", app.kid_menu.as_ref().unwrap().screen_face());
    }
    assert_eq!(app.kid_menu.as_ref().unwrap().remaining_seconds(), None);
    assert!(!band(&app.kid_menu.as_ref().unwrap().screen_face()));
}

#[test]
fn a_wrong_code_shakes_only_the_icons_then_resets_for_an_automatic_retry() {
    let (_d, mut app, clock) = warning_game(slot::kid_policy::Rule::Daily, 5);
    app.apply(Action::QuickMenu);
    press(&mut app, Btn::Up);
    press(&mut app, Btn::A);
    let remaining = app.kids.remaining(now());
    for b in [Btn::A; 4] { press(&mut app, b); }
    let menu = app.kid_menu.as_ref().unwrap();
    assert_eq!(menu.entered_buttons(), &[0; 4]);
    assert!(menu.entering_code());
    let still = menu.screen_face();
    press(&mut app, Btn::X);
    assert_eq!(app.kid_menu.as_ref().unwrap().entered_buttons(), &[0; 4]);
    let mut offsets = Vec::new();
    for i in 0..23 {
        let menu = app.kid_menu.as_ref().unwrap();
        offsets.push(menu.code_offset());
        if let Ok(dir) = std::env::var("SLOT_KID_UI_PREVIEW") {
            write_preview(&dir, &format!("parent-code-error-{i:02}.png"), menu.screen_face());
        }
        if i < 20 {
            let moving = menu.screen_face();
            for y in (0..208).chain(272..moving.h) {
                let row = (y * moving.w * 4) as usize;
                assert_eq!(&moving.rgba[row..row + moving.w as usize * 4],
                    &still.rgba[row..row + still.w as usize * 4]);
            }
        }
        clock.set(clock.get() + 17);
        app.update(1.0 / 60.0);
    }
    assert!(offsets.iter().any(|x| *x > 0) && offsets.iter().any(|x| *x < 0));
    assert!(app.kid_menu.as_ref().unwrap().entered_buttons().is_empty());
    assert!(app.kid_menu.as_ref().unwrap().entering_code());
    assert_eq!(app.kids.remaining(now()), remaining);
    for b in [Btn::A, Btn::B, Btn::X] { press(&mut app, b); }
    assert!(app.kid_menu.as_ref().unwrap().entering_code());
    press(&mut app, Btn::Y);
    assert!(!app.kid_menu.as_ref().unwrap().entering_code());
    assert!(app.kid_menu.as_ref().unwrap().entered_buttons().is_empty());
    assert_eq!(app.kid_menu.as_ref().unwrap().lines()[3], "SELECTED Kid mode On");
    assert!(app.kid_mode());
    assert!(matches!(app.phase(), Phase::Playing { .. }));
}

#[test]
fn time_controls_allow_one_minute_steps_and_separate_hours_and_minutes() {
    let d = card();
    let mut app = App::boot(d.path());
    app.apply(Action::QuickMenu);
    unlock(&mut app);
    for _ in 0..2 { press(&mut app, Btn::Down); }
    press(&mut app, Btn::Right);
    assert_eq!(app.kids.minutes, 6);
    press(&mut app, Btn::A);
    assert_eq!(app.kid_menu.as_ref().unwrap().lines()[1], "Hours 0");
    press(&mut app, Btn::Right);
    assert_eq!(app.kids.minutes, 66);
    press(&mut app, Btn::Down);
    press(&mut app, Btn::Right);
    assert_eq!(app.kids.minutes, 67);
    assert_eq!(Policy::read(d.path()).unwrap().minutes, 67);
    if let Ok(dir) = std::env::var("SLOT_KID_UI_PREVIEW") {
        write_preview(&dir, "parent-hours-minutes.png", app.kid_menu.as_ref().unwrap().screen_face());
    }
    press(&mut app, Btn::B);
    assert!(app.kid_menu.as_ref().unwrap().lines().iter().any(|l| l == "SELECTED 1 h 7 min"));
    if let Ok(dir) = std::env::var("SLOT_KID_UI_PREVIEW") {
        write_preview(&dir, "parent-duration.png", app.kid_menu.as_ref().unwrap().screen_face());
    }
    press(&mut app, Btn::A);
    for _ in 0..8 { press(&mut app, Btn::Right); }
    assert_eq!(app.kids.minutes, 240);
    press(&mut app, Btn::Down);
    press(&mut app, Btn::Left);
    assert_eq!(app.kids.minutes, 239);
}

#[test]
fn bedtime_editor_wraps_at_midnight_in_one_minute_steps() {
    use slot::kid_menu::Menu;
    let d = card();
    let mut policy = Policy::read(d.path()).unwrap();
    policy.leave();
    policy.rule = slot::kid_policy::Rule::Until;
    policy.bedtime = 1439;
    let mut menu = Menu::new(d.path().to_owned(), policy);
    let tap = |m: &mut Menu, b| m.input(Action::GbaDown(b), now());
    for b in [Btn::A, Btn::B, Btn::X, Btn::Y, Btn::Down, Btn::Down, Btn::A] { tap(&mut menu, b); }
    tap(&mut menu, Btn::Down);
    tap(&mut menu, Btn::Right);
    assert_eq!(menu.draft.bedtime, 0);
    tap(&mut menu, Btn::Left);
    assert_eq!(menu.draft.bedtime, 1439);
    tap(&mut menu, Btn::Up);
    tap(&mut menu, Btn::Right);
    assert_eq!(menu.draft.bedtime, 59);
}
