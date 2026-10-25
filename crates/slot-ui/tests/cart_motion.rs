use slot_ui::{eject_depth, eject_progress_for_depth, insert_depth};

#[test]
fn travel_stays_in_the_slot_and_never_reverses() {
    assert_eq!(insert_depth(-1.0), 0.0);
    assert_eq!(insert_depth(2.0), 1.0);
    assert_eq!(eject_depth(-1.0), 1.0);
    assert_eq!(eject_depth(2.0), 0.0);
    let (mut last_in, mut last_out) = (0.0, 1.0);
    for i in 0..=1_000 {
        let u = i as f32 / 1_000.0;
        let (inside, outside) = (insert_depth(u), eject_depth(u));
        assert!((0.0..=1.0).contains(&inside));
        assert!((0.0..=1.0).contains(&outside));
        assert!(inside + 0.00001 >= last_in);
        assert!(outside <= last_out + 0.00001);
        (last_in, last_out) = (inside, outside);
    }
}

#[test]
fn the_insert_slides_then_snaps_the_last_bit_home() {
    assert!((insert_depth(0.84) - 0.86).abs() < 0.001);
    let hand = insert_depth(0.84) - insert_depth(0.79);
    let snap = insert_depth(1.0) - insert_depth(0.95);
    assert!(snap > hand * 2.0, "no final snap: {snap} against {hand}");
    // No mid-travel stop at the lip, and no teleport at the snap boundary.
    for i in 2..16 {
        let u = i as f32 * 0.05;
        assert!(insert_depth(u + 0.05) - insert_depth(u) > 0.02);
    }
    assert!((insert_depth(0.8401) - insert_depth(0.8399)).abs() < 0.001);
}

#[test]
fn the_eject_pops_partway_up_then_slides_out() {
    assert!((eject_depth(0.20) - 0.82).abs() < 0.001);
    let release = 1.0 - eject_depth(0.05);
    let settle = eject_depth(0.15) - eject_depth(0.20);
    assert!(release > settle * 10.0);
    assert!(eject_depth(0.25) > 0.80, "the pop pulled it completely out");
    assert!(eject_depth(0.75) > 0.0);
    let finish = eject_depth(0.95) - eject_depth(1.0);
    let pull = eject_depth(0.60) - eject_depth(0.65);
    assert!(finish < pull * 0.1, "the hand stops abruptly");
}

#[test]
fn interrupted_travel_starts_from_the_current_depth() {
    for i in 0..=100 {
        let depth = insert_depth(i as f32 / 100.0);
        let progress = eject_progress_for_depth(depth);
        assert!((eject_depth(progress) - depth).abs() < 0.00001);
    }
}
