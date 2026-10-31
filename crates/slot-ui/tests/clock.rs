use slot_ui::{clock_label, Polaroids};

#[test]
fn recent_states_read_relative_and_old_ones_read_absolute() {
    let now = "2026-08-09_14-36-05";
    assert_eq!(
        Polaroids::relative_time("2026-08-09_14-32-05", now),
        "4 min ago"
    );
    assert_eq!(
        Polaroids::relative_time("2026-08-09_03-00-00", now),
        "11 hr ago"
    );
    assert_eq!(
        Polaroids::relative_time("2026-08-08_20-00-00", now),
        "2026-08-08 20:00"
    );
    assert_eq!(
        Polaroids::relative_time("2025-01-02_09-05-00", now),
        "2025-01-02 09:05"
    );
}

#[test]
fn the_clock_is_24_hour_and_shows_no_seconds() {
    assert_eq!(
        clock_label("2026-08-09_21-07-00"),
        "21:07",
        "not 24 hour, or showing seconds"
    );
}

#[test]
fn navigation_does_not_change_time_until_a_field_is_opened() {
    use slot_ui::{ClockPicker, Field};
    let mut picker=ClockPicker::from_ymd(2026,10,30);
    let before=picker.secs();
    picker.navigate(1,0); picker.navigate(0,1);
    assert_eq!(picker.cursor(),Field::Minute);
    assert_eq!(picker.secs(),before);
    picker.toggle_editing(); picker.navigate(0,-1);
    assert_eq!(picker.secs(),before+60);
    picker.toggle_editing(); picker.navigate(-1,0);
    assert_eq!(picker.cursor(),Field::Hour);
    assert_eq!(picker.secs(),before+60);
}

#[test]
fn twelve_hour_clock_handles_midnight_noon_and_local_day_wrap() {
    use slot_ui::clock_text;
    assert_eq!(clock_text(0,true),"12:00 AM");
    assert_eq!(clock_text(12*3600,true),"12:00 PM");
    assert_eq!(clock_text(13*3600+5*60,true),"1:05 PM");
    assert_eq!(clock_text(-60,true),"11:59 PM");
    assert_eq!(clock_text(24*3600,true),"12:00 AM");
    assert_eq!(clock_text(13*3600+5*60,false),"13:05");
}

#[test]
fn older_saves_and_screenshots_use_the_selected_clock_format(){
    assert_eq!(slot_ui::Polaroids::relative_time_format("2026-10-30_00-05-00","2026-10-31_14-00-00",true),"2026-10-30 12:05 AM");
    assert_eq!(slot_ui::Polaroids::relative_time_format("2026-10-30_12-05-00","2026-10-31_14-00-00",true),"2026-10-30 12:05 PM");
    assert_eq!(slot_ui::Polaroids::relative_time_format("2026-10-30_23-05-00","2026-10-31_14-00-00",false),"2026-10-30 23:05");
}
