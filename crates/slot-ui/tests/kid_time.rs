use slot_ui::{kid_time_meter, kid_time_text};

#[test]
fn time_labels_are_readable_at_each_warning_size() {
    for (seconds, expected) in [(3600, "1 hour left"), (7200, "2 hours left"),
        (3900, "1 h 5 min left"), (900, "15 minutes left"), (300, "5 minutes left"),
        (60, "1 minute left"), (30, "Less than a minute left"), (0, "Time is up")] {
        assert_eq!(kid_time_text(seconds), expected);
    }
}

#[test]
fn remaining_time_is_a_compact_antialiased_pill() {
    for seconds in [600,300,30,0] {
        let face=kid_time_meter(seconds,600);
        assert_eq!(face.h,36);
        assert!(face.w<520);
        assert_eq!(face.rgba[3],0,"rounded corner stays transparent");
        assert!(face.rgba.chunks_exact(4).any(|p|p[3]>0));
    }
}

#[test]
fn remaining_time_meter_tracks_fraction_even_when_the_text_is_identical() {
    let a=kid_time_meter(30,60);
    let b=kid_time_meter(30,600);
    assert_eq!((a.w,a.h),(b.w,b.h));
    assert_ne!(a.rgba,b.rgba);
}
