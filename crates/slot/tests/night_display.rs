mod common;
use slot::app::App;
use slot_input::{Action, Btn, Gestures, RawEvent};
#[test]
fn day_and_night_keep_separate_adjustments_and_survive_reboot() {
    let d = common::tmp_root_with_carts(&["Alpha", "Beta"]);
    common::clocked(d.path());
    let mut a = App::boot(d.path());
    let day = (a.brightness(), a.blue_light());
    a.apply(Action::NightToggle);
    assert!(a.night_mode());
    assert_eq!((a.brightness(), a.blue_light()), (2, 9));
    a.apply(Action::BrightnessUp);
    a.apply(Action::BlueLightDown);
    a.apply(Action::NightToggle);
    assert_eq!((a.brightness(), a.blue_light()), day);
    a.apply(Action::NightToggle);
    assert_eq!((a.brightness(), a.blue_light()), (3, 8));
    a.settle_saves();
    a.flush_state();
    let mut b = App::boot(d.path());
    assert!(b.night_mode());
    assert_eq!((b.brightness(), b.blue_light()), (3, 8));
    b.apply(Action::NightToggle);
    assert_eq!((b.brightness(), b.blue_light()), day);
}
#[test]
fn night_chord_does_not_emit_select_or_x_to_the_game() {
    let mut g = Gestures::new();
    g.set_entries(slot::input::Table::new(
        &slot_store::Controls::default(), &slot_store::ShortcutKeys::default(),
        &slot_store::CarouselKeys::default(), false,
    ).entries());
    assert!(g.feed(RawEvent::Down(Btn::Select), 0).is_empty());
    assert_eq!(
        g.feed(RawEvent::Down(Btn::X), 50),
        vec![Action::NightToggle]
    );
    assert!(g.feed(RawEvent::Up(Btn::X), 100).is_empty());
    assert!(g.feed(RawEvent::Up(Btn::Select), 150).is_empty());
}

#[test]
fn migrates_only_display_fields_and_leaves_old_preferences_untouched() {
    let d = common::tmp_root_with_carts(&["Alpha", "Beta"]);
    common::clocked(d.path());
    let old = "night=1\nday=7,2\nbedtime=1,8\nquiet=1\nlid_close=2\n";
    std::fs::write(d.path().join("Config/comfort.txt"), old).unwrap();
    let mut a = App::boot(d.path());
    assert!(a.night_mode());
    assert_eq!((a.brightness(), a.blue_light()), (1, 8));
    a.apply(Action::NightToggle);
    assert_eq!((a.brightness(), a.blue_light()), (7, 2));
    assert_eq!(
        std::fs::read_to_string(d.path().join("Config/comfort.txt")).unwrap(),
        old
    );
}
#[test]
fn invalid_display_values_leave_safe_defaults() {
    let d = common::tmp_root_with_carts(&["Alpha", "Beta"]);
    common::clocked(d.path());
    std::fs::write(
        d.path().join("Config/display.txt"),
        "night=1\nbedtime=255,255\n",
    )
    .unwrap();
    let a = App::boot(d.path());
    assert_eq!((a.brightness(), a.blue_light()), (2, 9));
}

#[test]
fn schedule_handles_boundaries_overnight_and_equal_times() {
    use slot::display_settings::DisplaySettings;
    let mut s = DisplaySettings { schedule: true, ..Default::default() };
    assert_eq!(s.scheduled_night(7*3600-1),Some(true));
    assert_eq!(s.scheduled_night(7*3600),Some(false));
    assert_eq!(s.scheduled_night(20*3600),Some(true));
    assert_eq!(s.next_change(20*3600),86_400+7*3600);
    s.day_start=20*60;s.night_start=7*60;
    assert_eq!(s.scheduled_night(23*3600),Some(false));
    assert_eq!(s.scheduled_night(6*3600),Some(false));
    assert_eq!(s.scheduled_night(12*3600),Some(true));
    s.day_start=s.night_start;
    assert_eq!(s.scheduled_night(12*3600),None);
}

#[test]
fn scheduled_presets_and_manual_override_survive_restart_and_sleep() {
    use slot::display_settings::DisplaySettings;
    let d=common::tmp_root_with_carts(&["Alpha","Beta"]);common::clocked(d.path());
    let midnight=1_800_000_000i64.div_euclid(86_400)*86_400;
    DisplaySettings { schedule:true, day:(8,1), bedtime:(2,8), ..Default::default() }.save(d.path()).unwrap();
    let (mut a,clock)=common::app_booting_at(d.path(),midnight+19*3600);
    assert!(!a.night_mode());
    clock.advance(3600);a.update(0.1);assert!(a.night_mode());
    assert_eq!((a.brightness(),a.blue_light()),(2,8));
    a.apply(Action::NightToggle);assert!(!a.night_mode());
    clock.advance(1800);a.update(0.1);assert!(!a.night_mode());
    a.settle_saves();drop(a);
    let (mut b,clock)=common::app_booting_at(d.path(),midnight+20*3600+1800);
    assert!(!b.night_mode(),"manual override must survive reboot");
    b.apply(Action::LidClose);clock.advance(12*3600);b.apply(Action::LidOpen);
    assert!(!b.night_mode());
    assert_eq!((b.brightness(),b.blue_light()),(8,1));
    clock.advance(12*3600);b.update(0.1);assert!(b.night_mode());
    assert!(DisplaySettings::read(d.path(),&Default::default()).override_until.is_none());
}

#[test]
fn schedule_uses_local_timezone_and_requires_a_valid_clock() {
    use slot::display_settings::DisplaySettings;
    let d=common::tmp_root_with_carts(&["Alpha","Beta"]);
    slot_store::write_slot_state(d.path(),&slot_store::SlotState{clock_set:true,utc_offset_min:120,..Default::default()}).unwrap();
    DisplaySettings {schedule:true,..Default::default()}.save(d.path()).unwrap();
    let midnight=1_800_000_000i64.div_euclid(86_400)*86_400;
    let (a,_)=common::app_booting_at(d.path(),midnight+18*3600);assert!(a.night_mode());
    let d=common::tmp_root_with_carts(&["Alpha","Beta"]);common::clocked(d.path());
    DisplaySettings {schedule:true,..Default::default()}.save(d.path()).unwrap();
    let (mut a,_)=common::app_booting_at(d.path(),0);a.update(0.1);assert!(!a.night_mode());
}

#[test]
fn schedule_editor_is_a_persisted_setting_and_returns_to_screen_menu() {
    use slot_ui::QuickRow;
    let d=common::tmp_root_with_carts(&["Alpha","Beta"]);common::clocked(d.path());
    let (mut a,_)=common::app_booting_at(d.path(),1_800_000_000);
    let press=|a:&mut App,b| {a.apply(Action::GbaDown(b));a.apply(Action::GbaUp(b));};
    a.apply(Action::QuickMenu);press(&mut a,Btn::Right);press(&mut a,Btn::A);press(&mut a,Btn::A);
    press(&mut a,Btn::Down);press(&mut a,Btn::Down);
    assert_eq!(a.quick_menu(),Some(QuickRow::DisplaySchedule));press(&mut a,Btn::A);
    assert!(a.display_schedule.is_some());press(&mut a,Btn::Right);
    press(&mut a,Btn::Down);press(&mut a,Btn::A);press(&mut a,Btn::Down);press(&mut a,Btn::Right);
    assert_eq!(slot::display_settings::DisplaySettings::read(d.path(),&Default::default()).day_start,421);
    press(&mut a,Btn::B);
    if let Ok(path)=std::env::var("SLOT_QOL_PREVIEW") {
        let f=a.display_schedule.as_ref().unwrap().screen_face();std::fs::create_dir_all(&path).unwrap();
        let mut e=png::Encoder::new(std::fs::File::create(format!("{path}/display-schedule.png")).unwrap(),f.w,f.h);
        e.set_color(png::ColorType::Rgba);e.set_depth(png::BitDepth::Eight);e.write_header().unwrap().write_image_data(&f.rgba).unwrap();
    }
    press(&mut a,Btn::B);assert!(a.display_schedule.is_none());assert_eq!(a.quick_menu(),Some(QuickRow::DisplaySchedule));
}

#[test]
fn edited_night_warmth_is_shared_by_schedule_shortcut_and_reboot() {
    use slot::{display_settings::DisplaySettings,display_schedule};
    let d=common::tmp_root_with_carts(&["Alpha","Beta"]);common::clocked(d.path());
    let midnight=1_800_000_000i64.div_euclid(86_400)*86_400;
    DisplaySettings{schedule:true,day:(8,1),bedtime:(2,8),..Default::default()}.save(d.path()).unwrap();
    let (mut a,clock)=common::app_booting_at(d.path(),midnight+19*3600);
    a.display_schedule=Some(display_schedule::Menu::new(DisplaySettings::read(d.path(),&Default::default())));
    let press=|a:&mut App,b|{a.apply(Action::GbaDown(b));a.apply(Action::GbaUp(b));};
    for _ in 0..3 {press(&mut a,Btn::Down);}
    press(&mut a,Btn::Left);assert_eq!(a.blue_light(),1,"editing the night preset must leave day warmth alone");
    assert_eq!(DisplaySettings::read(d.path(),&Default::default()).bedtime.1,7);
    press(&mut a,Btn::B);
    clock.advance(3600);a.update(0.1);assert!(a.night_mode());assert_eq!(a.blue_light(),7);
    a.display_schedule=Some(display_schedule::Menu::new(DisplaySettings::read(d.path(),&Default::default())));
    for _ in 0..3 {press(&mut a,Btn::Down);}press(&mut a,Btn::Left);
    assert_eq!(a.blue_light(),6,"editing at night applies immediately");press(&mut a,Btn::B);
    a.apply(Action::NightToggle);assert_eq!(a.blue_light(),1);
    a.apply(Action::NightToggle);assert_eq!(a.blue_light(),6);
    a.settle_saves();a.flush_state();drop(a);
    let (b,_)=common::app_booting_at(d.path(),midnight+20*3600);
    assert!(b.night_mode());assert_eq!(b.blue_light(),6);
}

#[test]
fn night_schedule_clock_format_changes_labels_without_changing_schedule(){
    let mut menu=slot::display_schedule::Menu::new(slot::display_settings::DisplaySettings::default());let before=menu.settings;let a=menu.screen_face();menu.twelve=true;let b=menu.screen_face();assert_ne!(a.rgba,b.rgba);assert_eq!(menu.settings.day_start,before.day_start);assert_eq!(menu.settings.night_start,before.night_start);
}
