use slot_ui::{settings_notice_face, Canvas, Draw, SettingsNotice, TexId};

#[test]
fn unchanged_status_expires_and_another_attempt_rearms_it() {
    let mut notice = SettingsNotice::default();
    let message = Some((1, "Set date & time first".into()));
    notice.observe(message.clone(), 100);
    assert_eq!(notice.alpha(100), 0.0);
    assert!((notice.alpha(170) - 0.5).abs() < 0.001);
    assert_eq!(notice.alpha(300), 1.0);
    notice.observe(message, 2500);
    assert_eq!(notice.alpha(2900), 0.0);
    notice.observe(Some((2, "Set date & time first".into())), 3000);
    assert_eq!(notice.alpha(3200), 1.0);
    notice.observe(Some((3, "Could not save settings".into())), 3300);
    assert_eq!(notice.text(), Some("Could not save settings"));
    assert_eq!(notice.alpha(3300), 0.0);
    notice.observe(None, 3400);
    assert_eq!(notice.alpha(3500), 0.0);
}

#[test]
fn notices_fit_both_native_canvases_at_the_bottom_without_a_scrim() {
    for w in [640.0, 720.0] {
        let canvas = Canvas { w, h: 480.0 };
        for message in [
            "Error",
            "Device disconnected — choose output",
            "Day and night need different times",
            "Could not save parent settings",
            "Saved Bluetooth/latest-report.txt",
        ] {
            let face = settings_notice_face(message, canvas);
            assert!(face.w <= w as u32 - 64 && face.h <= 68);
            assert!(face.rgba.chunks_exact(4).any(|p| p[3] > 0));
            let mut notice = SettingsNotice::default();
            notice.observe(Some((1, message.into())), 0);
            let mut out = vec![];
            notice.draw(
                canvas,
                Some((TexId::from_raw(1), face.w, face.h)),
                200,
                &mut out,
            );
            assert_eq!(out.len(), 2);
            assert!(
                matches!(out[0],Draw::RoundRect{x,y,w:ww,h,..} if x>=16.0 && x+ww<=w-16.0 && y>300.0 && y+h<=414.0)
            );
        }
    }
}
