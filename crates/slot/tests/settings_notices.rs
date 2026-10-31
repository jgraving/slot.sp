use slot::{app::App, display_schedule, display_settings::DisplaySettings};
use slot_input::{Action, Btn};

#[test]
fn another_invalid_attempt_rearms_the_notice_and_leaving_dismisses_it() {
    let mut app = App::new(vec![]);
    app.display_schedule = Some(display_schedule::Menu::new(DisplaySettings {
        day_start: 420,
        night_start: 435,
        ..Default::default()
    }));
    app.apply(Action::GbaDown(Btn::Down));
    app.apply(Action::GbaDown(Btn::Right));
    let first = app
        .settings_notice()
        .expect("the conflicting schedule needs feedback");
    assert_eq!(first.1, "Day and night need different times");
    app.apply(Action::GbaDown(Btn::Right));
    let repeated = app.settings_notice().unwrap();
    assert_ne!(first.0, repeated.0);
    assert_eq!(first.1, repeated.1);
    app.apply(Action::GbaDown(Btn::B));
    assert!(app.settings_notice().is_none());
}

#[test]
fn dynamic_notices_are_not_baked_into_the_cached_sound_page() {
    for w in [640.0, 720.0] {
        let canvas = slot_ui::Canvas { w, h: 480.0 };
        let lines = [
            "",
            "",
            "COMPACT",
            "SELECTED BLUETOOTH On",
            "ROW Pair audio device",
            "B Back · A Select",
        ]
        .map(str::to_owned);
        let clean = slot::bluetooth::screen_face_without_switches(&lines, slot_ui::HUD_INK, canvas);
        let mut warning = lines;
        warning[1] = "Error: Could not connect".into();
        let face =
            slot::bluetooth::screen_face_without_switches(&warning, slot_ui::HUD_INK, canvas);
        assert_eq!(
            clean.rgba, face.rgba,
            "message belongs to the transient overlay"
        );
        warning[1] = "CONFIRM_FORGET".into();
        let confirmation =
            slot::bluetooth::screen_face_without_switches(&warning, slot_ui::HUD_INK, canvas);
        assert_ne!(
            clean.rgba, confirmation.rgba,
            "confirmation must stay visible"
        );
    }
}

#[test]
fn charger_notice_appears_once_per_connection_and_clears_on_unplug() {
    use slot_power::{Battery,Charge};
    let mut app=App::new(vec![]);
    app.on_battery(Battery{percent:57,charge:Charge::Discharging});
    assert!(app.settings_notice().is_none());
    app.on_battery(Battery{percent:57,charge:Charge::Charging});
    assert_eq!(app.take_sfx(),Some(slot::audio::Sfx::Charging));
    let first=app.settings_notice().unwrap();
    assert_eq!(first.1,"Charging: 57%");
    app.on_battery(Battery{percent:57,charge:Charge::Charging});
    assert_eq!(app.take_sfx(),None);
    assert_eq!(app.settings_notice(),Some(first));
    app.update(3.0);
    assert!(app.settings_notice().is_none());
    app.on_battery(Battery{percent:57,charge:Charge::Charging});
    assert!(app.settings_notice().is_none());
    app.on_battery(Battery{percent:57,charge:Charge::Discharging});
    app.on_battery(Battery{percent:58,charge:Charge::Charging});
    assert_eq!(app.take_sfx(),Some(slot::audio::Sfx::Charging));
    assert_eq!(app.settings_notice().unwrap().1,"Charging: 58%");
    app.on_battery(Battery{percent:58,charge:Charge::Discharging});
    assert!(app.settings_notice().is_none());
}

#[test]
fn discovering_a_connected_charger_at_boot_is_silent() {
    use slot_power::{Battery,Charge};
    let mut app=App::new(vec![]);
    app.on_battery(Battery{percent:99,charge:Charge::Charging});
    assert_eq!(app.take_sfx(),None);
    app.on_battery(Battery{percent:100,charge:Charge::Full});
    assert_eq!(app.take_sfx(),None,"finishing a charge is not another connection");
    app.on_battery(Battery{percent:99,charge:Charge::Discharging});
    app.on_battery(Battery{percent:99,charge:Charge::Charging});
    assert_eq!(app.take_sfx(),Some(slot::audio::Sfx::Charging));
}
