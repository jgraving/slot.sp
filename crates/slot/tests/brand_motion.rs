use slot::brand_motion::{frame_at,boot_opacity,FRAMES,REVEAL_MS,BOOT_MS};
#[test]
fn reveal_locks_to_final_frame_and_fade_ends_without_replaying() {
    assert_eq!(frame_at(0),0);
    assert_eq!(frame_at(REVEAL_MS),FRAMES-1);
    assert_eq!(frame_at(u64::MAX),FRAMES-1);
    let mut previous=0;
    for ms in 0..=BOOT_MS {
        let frame=frame_at(ms);
        assert!(frame>=previous);previous=frame;
        assert!((0.0..=1.0).contains(&boot_opacity(ms)));
    }
    assert_eq!(boot_opacity(REVEAL_MS),1.0);
    assert_eq!(boot_opacity(BOOT_MS),0.0);
}
