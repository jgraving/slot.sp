mod common;
use slot::{session::Session,app::App};
use slot_input::{Btn,RawEvent,Action};
use slot_power::{Platform,Power,SimPlatform,Battery,Charge,LedState};
use slot_store::{SlotState,Appearance,WallpaperPick,write_slot_state,read_slot_state};
use std::{time::Duration,path::Path,sync::{Arc,Mutex}};
struct Probe{inner:SimPlatform,pulses:Arc<Mutex<Vec<(u16,Duration)>>>}
impl Platform for Probe {
    fn set_backlight(&mut self,v:u8){self.inner.set_backlight(v)}
    fn battery(&self)->Option<Battery>{self.inner.battery()}
    fn charge(&self)->Charge{self.inner.charge()}
    fn set_led(&mut self,v:LedState){self.inner.set_led(v)}
    fn poweroff(&mut self)->!{self.inner.poweroff()}
    fn restart(&mut self)->!{self.inner.restart()}
    fn root(&self)->&Path{self.inner.root()}
    fn now(&self)->i64{self.inner.now()}
    fn set_clock(&mut self,v:i64){self.inner.set_clock(v)}
    fn set_rumble(&mut self,v:u16){self.inner.set_rumble(v)}
    fn supports_rumble(&self)->bool{self.inner.supports_rumble()}
    fn set_rumble_for(&mut self,v:u16,d:Duration){self.pulses.lock().unwrap().push((v,d));self.inner.set_rumble(v)}
}
fn rig(root:&Path,supported:bool)->(Session,slot_power::Motor,Arc<Mutex<Vec<(u16,Duration)>>>) {
    let mut state=read_slot_state(root);state.clock_set=true;write_slot_state(root,&state).unwrap();
    let inner=SimPlatform::at(root.to_owned());let inner=if supported{inner}else{inner.without_rumble()};
    let motor=inner.motor();let pulses=Arc::new(Mutex::new(vec![]));let mut s=Session::boot(root.to_owned());s.app_mut().set_power(Power::new(Box::new(Probe{inner,pulses:pulses.clone()}),Duration::from_secs(300)));(s,motor,pulses)
}
#[test]
fn menu_feedback_requests_one_low_strength_eight_millisecond_tap(){
    let d=common::tmp_root_with_carts(&["Approved","Another"]);let (mut s,motor,pulses)=rig(d.path(),true);
    s.feed([RawEvent::Down(Btn::Menu)],0);s.feed([RawEvent::Up(Btn::Menu)],20);
    let pulses=pulses.lock().unwrap().clone();assert_eq!(pulses.len(),1);let(strength,duration)=pulses[0];assert!(strength>0&&strength<=u16::MAX/8);assert_eq!(duration,Duration::from_millis(8));assert_eq!(motor.last(),strength);
    s.update(0.009);assert_eq!(motor.last(),0);
}
#[test]
fn disabled_feedback_and_devices_without_a_motor_remain_silent(){
    for supported in [false,true] {
        let d=common::tmp_root_with_carts(&["Approved","Another"]);write_slot_state(d.path(),&SlotState{clock_set:true,ui_vibration:false,charging_vibration:false,..Default::default()}).unwrap();
        let(mut s,motor,pulses)=rig(d.path(),supported);s.feed([RawEvent::Down(Btn::Menu)],0);s.feed([RawEvent::Up(Btn::Menu)],20);s.update(0.02);assert_eq!(motor.last(),0);assert!(pulses.lock().unwrap().is_empty());
        assert_eq!(s.app().hidden_rows().contains(&slot_ui::QuickRow::UiVibration),!supported);
    }
}
#[test]
fn charging_feedback_fires_only_on_a_new_connection_and_stops_during_sleep(){
    let d=common::tmp_root_with_carts(&["Approved","Another"]);let(mut s,motor,pulses)=rig(d.path(),true);
    s.app_mut().on_battery(Battery{percent:30,charge:Charge::Discharging});s.app_mut().on_battery(Battery{percent:30,charge:Charge::Charging});s.update(0.0);
    assert!(motor.last()>0);assert_eq!(pulses.lock().unwrap().len(),1);s.update(0.04);assert_eq!(motor.last(),0);s.update(0.07);assert!(motor.last()>0);assert_eq!(pulses.lock().unwrap().len(),2);
    s.app_mut().on_battery(Battery{percent:31,charge:Charge::Charging});s.update(0.05);assert_eq!(motor.last(),0);assert_eq!(pulses.lock().unwrap().len(),2);
    s.feed([RawEvent::Down(Btn::Lid)],200);assert_eq!(motor.last(),0);
}
#[test]
fn light_theme_and_independent_vibration_switches_survive_restart(){
    let d=common::tmp_root_with_carts(&["Approved","Another"]);let state=SlotState{clock_set:true,appearance:Appearance::Light,ui_vibration:false,charging_vibration:true,..Default::default()};write_slot_state(d.path(),&state).unwrap();let app=App::boot(d.path());assert!(app.light());assert!(!app.ui_vibration_enabled());assert!(app.charging_vibration_enabled());assert_eq!(read_slot_state(d.path()),state);
}
#[test]
fn legacy_blur_is_migrated_and_cannot_be_selected_again(){
    let d=common::tmp_root_with_carts(&["Approved","Another"]);write_slot_state(d.path(),&SlotState{clock_set:true,wallpaper:WallpaperPick::Blur,..Default::default()}).unwrap();let mut a=App::boot(d.path());assert_eq!(a.wallpaper(),WallpaperPick::Default);
    a.apply(Action::QuickMenu);common::through(&mut a,Btn::A);
    for _ in 0..15 {assert_ne!(a.wallpaper(),WallpaperPick::Blur);common::through(&mut a,Btn::Right);}
}
