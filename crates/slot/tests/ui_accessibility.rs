mod common;
use slot::{session::Session,app::App};
use slot_input::{Btn,RawEvent,Action};
use slot_power::{Power,SimPlatform,Battery,Charge};
use slot_store::{SlotState,Appearance,WallpaperPick,write_slot_state,read_slot_state};
use std::time::Duration;
#[test]
fn menus_and_charging_do_not_vibrate(){
    let d=common::tmp_root_with_carts(&["Approved","Another"]);
    let platform=SimPlatform::at(d.path().to_owned());let motor=platform.motor();
    let mut s=Session::boot(d.path().to_owned());
    s.app_mut().set_power(Power::new(Box::new(platform),Duration::from_secs(300)));
    s.feed([RawEvent::Down(Btn::Menu)],0);s.feed([RawEvent::Up(Btn::Menu)],20);
    s.app_mut().on_battery(Battery{percent:30,charge:Charge::Discharging});
    s.app_mut().on_battery(Battery{percent:30,charge:Charge::Charging});
    for _ in 0..20 {s.update(0.02);assert_eq!(motor.last(),0);}
}
#[test]
fn light_theme_survives_restart_and_old_vibration_preferences_are_ignored(){
    let d=common::tmp_root_with_carts(&["Approved","Another"]);
    let state=SlotState{clock_set:true,appearance:Appearance::Light,..Default::default()};
    write_slot_state(d.path(),&state).unwrap();
    let path=d.path().join("Config/slot.state");let mut saved=std::fs::read_to_string(&path).unwrap();
    saved.push_str("ui_vibration=1\ncharging_vibration=1\n");std::fs::write(path,saved).unwrap();
    let app=App::boot(d.path());assert!(app.light());assert_eq!(read_slot_state(d.path()),state);
}
#[test]
fn legacy_blur_is_migrated_and_cannot_be_selected_again(){
    let d=common::tmp_root_with_carts(&["Approved","Another"]);write_slot_state(d.path(),&SlotState{clock_set:true,wallpaper:WallpaperPick::Blur,..Default::default()}).unwrap();let mut a=App::boot(d.path());assert_eq!(a.wallpaper(),WallpaperPick::Default);
    a.apply(Action::QuickMenu);common::through(&mut a,Btn::A);
    for _ in 0..15 {assert_ne!(a.wallpaper(),WallpaperPick::Blur);common::through(&mut a,Btn::Right);}
}
