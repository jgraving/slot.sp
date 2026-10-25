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
fn meter_fill_tracks_the_remaining_budget_and_has_a_transparent_background() {
    for (seconds, fill) in [(600, 160), (300, 80), (0, 0)] {
        let face = kid_time_meter(seconds, 600);
        let filled = (face.w - 160..face.w).filter(|&x| {
            face.rgba[((face.h / 2 * face.w + x) * 4 + 3) as usize] == 255
        }).count();
        assert_eq!(filled, fill);
        assert_eq!(face.rgba[((face.w - 1) * 4 + 3) as usize], 0);
    }
}
