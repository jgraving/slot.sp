mod common;
use slot::app::{App,Phase};
use slot_input::{Action,Btn};
use slot_ui::{Draw,QuickRow};
fn app()->(tempfile::TempDir,App) {
    let root=common::tmp_root_with_carts(&["Alpha","Beta"]);common::clocked(root.path());
    let app=App::boot(root.path());(root,app)
}
fn press(app:&mut App,b:Btn){app.apply(Action::GbaDown(b));app.apply(Action::GbaUp(b));}
fn settle(app:&mut App){for _ in 0..30{app.update(0.05);}}
#[test]
fn taps_toggle_without_launching_a_game_and_the_card_slides_out() {
    let (_root,mut app)=app();let selected=app.selected_stem().map(str::to_owned);
    app.apply(Action::QuickControls);settle(&mut app);
    assert!(app.quick_controls_open());assert_eq!(app.input_context(),slot_input::Context::Menu);
    let muted=app.muted();press(&mut app,Btn::A);assert_ne!(app.muted(),muted);
    assert!(matches!(app.phase(),Phase::Shelf));assert_eq!(app.selected_stem(),selected.as_deref());
    press(&mut app,Btn::B);app.update(0.008);assert!(!app.quick_controls_open());
    let mut out=vec![];app.draw(&mut out);
    assert!(out.iter().any(|d|matches!(d,Draw::RoundRect{y,h,..}if *h==104.0&&*y>266.0)));
    settle(&mut app);out.clear();app.draw(&mut out);
    assert!(!out.iter().any(|d|matches!(d,Draw::RoundRect{h,..}if *h==104.0)));
}
#[test]
fn holding_a_opens_settings_after_the_exit_motion_without_toggling() {
    let (_root,mut app)=app();app.apply(Action::QuickControls);settle(&mut app);
    let muted=app.muted();app.apply(Action::GbaDown(Btn::A));settle(&mut app);
    assert!(!app.quick_controls_open());assert!(app.bluetooth.is_some());
    assert_eq!(app.quick_menu(),Some(QuickRow::Bluetooth));
    app.apply(Action::GbaUp(Btn::A));assert_eq!(app.muted(),muted);
}
#[test]
fn night_toggle_and_hold_share_the_night_settings() {
    let (_root,mut app)=app();app.apply(Action::QuickControls);settle(&mut app);
    press(&mut app,Btn::Right);let night=app.night_mode();press(&mut app,Btn::A);
    assert_ne!(app.night_mode(),night);
    app.apply(Action::GbaDown(Btn::A));settle(&mut app);
    assert!(app.display_schedule.is_some());
}
#[test]
fn kids_only_get_safe_sound_controls_and_restricted_settings() {
    let (_root,mut app)=app();app.kids.active=true;
    app.apply(Action::QuickControls);settle(&mut app);let night=app.night_mode();
    for _ in 0..4{press(&mut app,Btn::Right);press(&mut app,Btn::A);}
    assert_eq!(app.night_mode(),night);assert!(app.kids.active);
    app.apply(Action::GbaDown(Btn::A));settle(&mut app);
    assert!(app.kid_menu.as_ref().is_some_and(|m|m.child_menu()));
    assert!(app.kids.active);assert!(app.bluetooth.is_none());
}

#[test]
fn sleeping_cancels_a_pending_settings_hold() {
    let (_root,mut app)=app();app.apply(Action::QuickControls);settle(&mut app);
    app.apply(Action::GbaDown(Btn::A));app.update(0.51);
    app.apply(Action::LidClose);settle(&mut app);app.apply(Action::LidOpen);settle(&mut app);
    assert!(app.bluetooth.is_none());assert!(!app.quick_controls_open());
}

#[test]
fn back_cancels_a_settings_hold_during_the_cards_exit() {
    let (_root,mut app)=app();app.apply(Action::QuickControls);settle(&mut app);
    app.apply(Action::GbaDown(Btn::A));app.update(0.51);
    press(&mut app,Btn::B);settle(&mut app);
    assert!(app.bluetooth.is_none());assert!(matches!(app.phase(),Phase::Shelf));
}
#[test]
fn enabling_kid_mode_while_the_card_is_open_blocks_night_toggles() {
    let (_root,mut app)=app();app.apply(Action::QuickControls);settle(&mut app);
    press(&mut app,Btn::Right);let night=app.night_mode();app.kids.active=true;
    press(&mut app,Btn::A);assert_eq!(app.night_mode(),night);
}
