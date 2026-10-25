use slot_gfx::{lava_centres, lava_phase, LAVA_PERIOD_MS};

#[test]
fn motion_is_periodic_and_bounded_even_after_long_uptime() {
    for now in [0, 10, 9000, 18000, u64::MAX - LAVA_PERIOD_MS] {
        let phase = lava_phase(now);
        assert!((0.0..std::f32::consts::TAU).contains(&phase));
        for pool in lava_centres(phase) {
            assert!(pool.iter().all(|value| value.is_finite()));
            assert!(pool[2] > 0.08 && pool[3] > 0.08);
        }
    }
    assert_eq!(lava_phase(0), lava_phase(LAVA_PERIOD_MS));
    assert_ne!(lava_centres(lava_phase(0)), lava_centres(lava_phase(9000)));
}

#[test]
fn deforming_blobs_conserve_area_and_have_both_rising_and_falling_motion() {
    let a = lava_centres(lava_phase(0));
    let b = lava_centres(lava_phase(2000));
    assert!(a.iter().zip(b).any(|(a,b)| b[1] < a[1] - 0.02));
    assert!(a.iter().zip(b).any(|(a,b)| b[1] > a[1] + 0.02));
    for ms in (0..LAVA_PERIOD_MS).step_by(200) {
        for (rest, blob) in a.iter().zip(lava_centres(lava_phase(ms))) {
            assert!((rest[2] * rest[3] - blob[2] * blob[3]).abs() < 0.00001);
        }
    }
}
