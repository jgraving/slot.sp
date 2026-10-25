use slot_gfx::{lava_centres, lava_phase, LAVA_PERIOD_MS};

#[test]
fn motion_is_periodic_and_bounded_even_after_long_uptime() {
    for now in [0, 10, 9000, 18000, u64::MAX - LAVA_PERIOD_MS] {
        let phase = lava_phase(now);
        assert!((0.0..std::f32::consts::TAU).contains(&phase));
        for pool in lava_centres(phase) {
            assert!(pool.iter().all(|value| value.is_finite()));
            assert!(pool[2] > 0.4 && pool[3] > 0.5);
        }
    }
    assert_eq!(lava_phase(0), lava_phase(LAVA_PERIOD_MS));
    assert_ne!(lava_centres(lava_phase(0)), lava_centres(lava_phase(9000)));
}

#[test]
fn pools_breathe_visibly_within_two_seconds() {
    let rest = lava_centres(lava_phase(0));
    let expanded = lava_centres(lava_phase(2000));
    assert!(expanded[0][2] / rest[0][2] > 1.20);
    assert!(expanded[0][3] / rest[0][3] > 1.20);
}
