mod common;

use std::sync::atomic::Ordering;

use common::{app_playing_with_jack, tmp_root_with_carts};
use slot_input::Action;
use slot_store::read_slot_state;
use slot_ui::Icon;

fn has_indicator(a: &slot::app::App, id: slot_ui::TexId) -> bool {
    let mut draws = vec![];
    a.draw(&mut draws);
    draws.iter().any(
        |d| matches!(*d, slot_ui::Draw::Tex { tex, y, .. } if tex == id && y < slot_ui::PLATE_H),
    )
}

#[test]
fn persistent_mute_tracks_the_active_audio_route_and_survives_hud_timeout() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let (mut a, jack) = app_playing_with_jack(d.path(), "Emerald");
    a.set_game_ready(true);
    let mute = slot_ui::TexId::from_raw(9001);
    let connection = slot_ui::TexId::from_raw(9002);
    a.hud_mute_indicator = Some(mute);
    a.hud_audio_indicator = Some(connection);
    a.apply(Action::QuickMenu);
    for _ in 0..60 {
        a.update(1.0 / 60.0);
    }
    assert!(!has_indicator(&a, mute));
    a.apply(Action::MuteToggle);
    a.tick_ms(a.now() + 10_000);
    assert!(has_indicator(&a, mute));
    assert!(has_indicator(&a, connection));

    // Wired, Bluetooth and USB each have their own mute state. A silent
    // speaker must not label an audible newly-selected output as muted.
    jack.store(true, Ordering::Relaxed);
    a.tick_ms(a.now() + 1_000);
    assert!(!has_indicator(&a, mute));
    a.apply(Action::MuteToggle);
    assert!(has_indicator(&a, mute));
    a.bluetooth_state(Some("AA:BB:CC:DD:EE:FF".into()), true);
    a.apply(Action::VolumeUp);
    assert!(!has_indicator(&a, mute));
    a.apply_at(Action::MuteToggle, a.now() + 1_000);
    assert!(has_indicator(&a, mute));
    a.usb_state(true, false);
    a.apply(Action::VolumeUp);
    assert!(!has_indicator(&a, mute));
    a.apply_at(Action::MuteToggle, a.now() + 1_000);
    assert!(has_indicator(&a, mute));
    a.apply(Action::VolumeUp);
    assert!(!has_indicator(&a, mute));
}

#[test]
fn zero_volume_and_mute_are_visible_on_the_carousel_and_in_settings() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let mut a = common::boot(d.path());
    let mute = slot_ui::TexId::from_raw(9001);
    a.mute_indicator = Some(mute);
    assert!(!has_indicator(&a, mute));
    for _ in 0..20 {
        a.apply(Action::VolumeDown);
    }
    assert!(!a.muted());
    assert!(has_indicator(&a, mute));
    a.apply(Action::QuickMenu);
    assert!(has_indicator(&a, mute));
    a.apply(Action::VolumeUp);
    assert!(!has_indicator(&a, mute));
    a.apply(Action::MuteToggle);
    assert!(has_indicator(&a, mute));
}

#[test]
fn unplug_uses_disconnect_volume_and_preserves_wired_volume() {
    let d = tmp_root_with_carts(&["Emerald"]);
    std::fs::write(d.path().join("Config/sound.conf"), "disconnect_volume=20\n").unwrap();
    let (mut a, jack) = app_playing_with_jack(d.path(), "Emerald");
    let speaker = a.volume();

    jack.store(true, Ordering::Relaxed);
    let at = a.now() + 1_000;
    a.tick_ms(at);
    assert_eq!(
        a.hud_icon(),
        Icon::Headphones,
        "plugging in did not show headphones"
    );
    for _ in 0..3 {
        a.apply(Action::VolumeDown);
    }
    let headphones = a.volume();
    assert!(
        headphones < speaker,
        "the buttons did not move the headphone level"
    );

    jack.store(false, Ordering::Relaxed);
    let at = a.now() + 1_000;
    a.tick_ms(at);
    assert_eq!(
        a.volume(),
        20,
        "the speaker did not use the disconnect setting"
    );
    assert_eq!(a.hud_icon(), Icon::Volume);

    jack.store(true, Ordering::Relaxed);
    let at = a.now() + 1_000;
    a.tick_ms(at);
    assert_eq!(
        a.volume(),
        headphones,
        "the headphone level was not remembered"
    );

    a.flush_state();
    let saved = read_slot_state(d.path());
    assert_eq!((saved.volume, saved.volume_hp), (20, headphones));
}

#[test]
fn unplug_defaults_to_silence_without_forgetting_wired_mute() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let (mut a, jack) = app_playing_with_jack(d.path(), "Emerald");

    jack.store(true, Ordering::Relaxed);
    let at = a.now() + 1_000;
    a.tick_ms(at);
    a.apply(Action::MuteToggle);
    assert_eq!(a.output_volume(), 0);
    assert_eq!(a.hud_icon(), Icon::VolumeMuted);

    jack.store(false, Ordering::Relaxed);
    let at = a.now() + 1_000;
    a.tick_ms(at);
    assert_eq!(
        a.output_volume(),
        0,
        "default disconnect volume must be silent"
    );

    jack.store(true, Ordering::Relaxed);
    let at = a.now() + 1_000;
    a.tick_ms(at);
    assert_eq!(
        a.output_volume(),
        0,
        "the headphones forgot they were muted"
    );

    a.flush_state();
    let saved = read_slot_state(d.path());
    assert_eq!((saved.muted, saved.muted_hp), (true, true));
}

fn enable_pause(root: &std::path::Path) {
    std::fs::write(root.join("Config/sound.conf"),"disconnect_volume=15\npause_on_disconnect=1\n").unwrap();
}
#[test]
fn optional_wired_pause_stays_paused_on_reconnect_and_defaults_off() {
    for enabled in [false,true] {
        let d=tmp_root_with_carts(&["Emerald"]);if enabled {enable_pause(d.path());}
        let (mut a,jack)=app_playing_with_jack(d.path(),"Emerald");a.set_game_ready(true);
        jack.store(true,Ordering::Relaxed);a.tick_ms(a.now()+1000);
        jack.store(false,Ordering::Relaxed);a.tick_ms(a.now()+1000);
        assert_eq!(a.in_game_menu().is_some(),enabled);
        jack.store(true,Ordering::Relaxed);a.tick_ms(a.now()+1000);
        assert_eq!(a.in_game_menu().is_some(),enabled);
        if enabled { a.apply(Action::QuickMenu);assert!(a.in_game_menu().is_none()); }
    }
}
#[test]
fn bluetooth_loss_pauses_but_intentional_route_change_and_link_play_do_not() {
    for (manual,linked) in [(false,false),(true,false),(false,true)] {
        let d=tmp_root_with_carts(&["Emerald"]);enable_pause(d.path());
        let mut a=common::app_playing_in(d.path(),"Emerald");a.set_game_ready(true);
        if linked {a.begin_link(1);}
        let address=Some("AA:BB:CC:DD:EE:FF".to_string());
        a.bluetooth_state(address.clone(),true);
        a.bluetooth_state(if manual {None} else {address.clone()},false);
        assert_eq!(a.in_game_menu().is_some(),!manual&&!linked);
        a.bluetooth_state(address,true);
        assert_eq!(a.in_game_menu().is_some(),!manual&&!linked);
    }
}
#[test]
fn only_physical_usb_unplug_pauses_and_does_not_open_menus_twice() {
    let d=tmp_root_with_carts(&["Emerald"]);enable_pause(d.path());
    let mut a=common::app_playing_in(d.path(),"Emerald");a.set_game_ready(true);
    a.usb_state(true,false);a.usb_state(false,false);assert!(a.in_game_menu().is_none());
    a.usb_state(true,false);a.usb_state(false,true);assert!(a.in_game_menu().is_some());
    a.usb_state(false,true);assert!(a.in_game_menu().is_some());
    a.usb_state(true,false);assert!(a.in_game_menu().is_some());
}
#[test]
fn disconnect_in_kid_mode_opens_only_the_child_pause_menu() {
    let d=tmp_root_with_carts(&["Emerald"]);enable_pause(d.path());
    let mut policy=slot::kid_policy::Policy::default();
    policy.approved=vec![(slot_store::Platform::Gba,"Emerald".into())];
    policy.begin(common::CLOCK_IS_SET).unwrap();
    let (mut a,jack)=app_playing_with_jack(d.path(),"Emerald");a.set_game_ready(true);
    policy.save(d.path()).unwrap();a.kids=policy;
    assert!(a.kid_mode());
    jack.store(true,Ordering::Relaxed);a.tick_ms(a.now()+1000);
    jack.store(false,Ordering::Relaxed);a.tick_ms(a.now()+1000);
    assert!(a.kid_menu.as_ref().is_some_and(|m|m.child_menu()));
    assert!(a.in_game_menu().is_none());
}
#[test]
fn sound_pause_setting_and_volume_save_without_clobbering_each_other() {
    use slot::bluetooth::{Manager,disconnect_volume,pause_on_disconnect};
    use slot_input::Btn;
    let d=tmp_root_with_carts(&[]);let mut m=Manager::new(d.path().into(),false);
    m.selected=m.rows().iter().position(|r|r.starts_with("PAUSE_DISCONNECT ")).unwrap();m.input(Action::GbaDown(Btn::A));assert!(pause_on_disconnect(d.path()));
    m.selected-=1;m.input(Action::GbaDown(Btn::Right));assert!(pause_on_disconnect(d.path()));assert_eq!(disconnect_volume(d.path()),5);
    m.selected+=1;m.input(Action::GbaDown(Btn::Left));assert!(!pause_on_disconnect(d.path()));assert_eq!(disconnect_volume(d.path()),5);
    m.input(Action::GbaDown(Btn::Right));
    if let Ok(path)=std::env::var("SLOT_QOL_PREVIEW") {
        std::fs::create_dir_all(&path).unwrap();let f=slot::bluetooth::screen_face(&m.lines());
        let mut e=png::Encoder::new(std::fs::File::create(format!("{path}/pause-disconnect.png")).unwrap(),f.w,f.h);
        e.set_color(png::ColorType::Rgba);e.set_depth(png::BitDepth::Eight);e.write_header().unwrap().write_image_data(&f.rgba).unwrap();
    }
}
