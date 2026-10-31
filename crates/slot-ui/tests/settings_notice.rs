use slot_ui::{settings_notice_face, Canvas, Draw, SettingsNotice, TexId};

#[test]
fn unchanged_status_expires_and_another_attempt_rearms_it() {
    let mut notice = SettingsNotice::default();
    let message = Some((1, "Set date & time first".into()));
    notice.observe(message.clone(), 100);
    assert_eq!(notice.alpha(100), 0.0);
    assert!(notice.alpha(170)>0.5 && notice.alpha(170)<1.0);
    assert!(notice.alpha(300)>0.95);
    notice.observe(message, 2500);
    assert_eq!(notice.alpha(2900), 0.0);
    notice.observe(Some((2, "Set date & time first".into())), 3000);
    assert!(notice.alpha(3200)>0.95);
    notice.observe(Some((3, "Could not save settings".into())), 3300);
    assert_eq!(notice.text(), Some("Could not save settings"));
    assert_eq!(notice.alpha(3300), 1.0,"rearming a visible pill preserves its position");
    notice.observe(None, 3400);
    assert!(notice.alpha(3500)>0.0 && notice.alpha(3500)<1.0);
    assert_eq!(notice.alpha(3900),0.0);
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
                500,
                &mut out,
            );
            assert_eq!(out.len(), 2);
            assert!(
                matches!(out[0],Draw::RoundRect{x,y,w:ww,h,..} if x>=16.0 && x+ww<=w-16.0 && y>300.0 && y+h<=414.0)
            );
        }
    }
}

#[test]
fn feedback_moves_as_an_opaque_surface_and_retargets_without_a_jump() {
    let canvas=Canvas::FRAMED;
    let face=settings_notice_face("Saved",canvas);
    let texture=Some((TexId::from_raw(1),face.w,face.h));
    let mut n=SettingsNotice::default();n.observe(Some((1,"Saved".into())),0);
    let mut before=vec![];n.draw(canvas,texture,120,&mut before);
    assert!(before.iter().all(|d|match d {Draw::Tex{alpha,..}=>*alpha==1.0,Draw::RoundRect{colour,..}=>colour[3]==1.0,_=>true}));
    n.observe(Some((2,"Updated".into())),120);
    let mut after=vec![];n.draw(canvas,texture,120,&mut after);
    assert_eq!(before,after);
    n.observe(None,120);
    let mut closing=vec![];n.draw(canvas,texture,160,&mut closing);
    assert!(closing.iter().all(|d|match d {Draw::Tex{alpha,..}=>*alpha==1.0,_=>true}));
    assert!(n.alpha(160)>0.0);
    assert_eq!(n.alpha(1000),0.0);
}

#[test]
fn charging_chip_uses_one_filled_surface_and_readable_native_ink() {
    use slot_ui::{Canvas,Draw,SettingsNotice,TexId,settings_notice_face};
    let canvas=Canvas{w:640.0,h:480.0};
    let face=settings_notice_face("Charging: 100%",canvas);
    assert_eq!(face.h,52);assert!(face.w<400);
    assert!(face.rgba.chunks_exact(4).any(|p|p[3]>200));
    let mut notice=SettingsNotice::default();
    notice.observe(Some((1,"Charging: 100%".into())),0);
    let mut out=vec![];notice.draw(canvas,Some((TexId::from_raw(1),face.w,face.h)),500,&mut out);
    assert_eq!(out.iter().filter(|d|matches!(d,Draw::RoundRect{..})).count(),1);
    assert!(out.iter().any(|d|matches!(d,Draw::RoundRect{colour,..}if colour[1]>colour[0]&&colour[3]==1.0)));
}
