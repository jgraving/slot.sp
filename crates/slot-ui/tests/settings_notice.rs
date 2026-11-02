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
    assert_eq!(out.iter().filter(|d|matches!(d,Draw::RoundRect{colour,..} if colour[3]==1.0)).count(),2);
    assert!(out.iter().any(|d|matches!(d,Draw::RoundRect{colour,..}if colour[1]>colour[0]&&colour[3]==1.0)));
}

#[test]
fn an_arriving_notice_floats_whole_above_the_hardware_bay() {
    let canvas=Canvas{w:640.0,h:480.0};let face=settings_notice_face("Saved",canvas);
    let mut notice=SettingsNotice::default();notice.observe(Some((1,"Saved".into())),0);
    let mut out=vec![];notice.draw(canvas,Some((TexId::from_raw(1),face.w,face.h)),100,&mut out);
    assert!(out.iter().any(|d|matches!(d,Draw::RoundRect{y,h,..} if *h==face.h as f32&&*y+*h>422.0&&*y+*h<=480.0)),"the menu footer must not crop the floating capsule");
}

#[test]
fn charging_pulses_the_whole_pill_then_settles_without_restarting_on_a_battery_poll() {
    let canvas=Canvas::FRAMED;
    let face=settings_notice_face("Charging: 57%",canvas);
    let texture=Some((TexId::from_raw(1),face.w,face.h));
    let mut notice=SettingsNotice::default();notice.observe(Some((1,"Charging: 57%".into())),0);
    let bounds=|notice:&SettingsNotice,ms| {
        let mut out=vec![];notice.draw(canvas,texture,ms,&mut out);
        let chip=out.iter().find_map(|d|if let Draw::RoundRect{x,y,w,h,..}=*d{Some((x,y,w,h))}else{None}).unwrap();
        assert!(out.iter().any(|d|matches!(*d,Draw::Tex{x,y,w,h,alpha:1.0,..} if (x,y,w,h)==chip)),"ink must pulse with the surface");
        assert!((chip.0+chip.2/2.0-canvas.w/2.0).abs()<0.01);
        assert!((chip.2/face.w as f32-chip.3/face.h as f32).abs()<0.001);
        chip
    };
    let widths:Vec<_>=(200..1100).step_by(16).map(|ms|bounds(&notice,ms).2).collect();
    assert!(widths.iter().copied().fold(0.0,f32::max)>face.w as f32*1.02);
    assert!(widths.iter().all(|w|*w<=face.w as f32*1.036));
    assert_eq!(bounds(&notice,1200).2,face.w as f32);
    notice.observe(Some((1,"Charging: 58%".into())),1200);
    assert_eq!(bounds(&notice,1450).2,face.w as f32,"percentage changes must not restart the pulse");
    assert_eq!(notice.alpha(2900),0.0,"percentage changes must not extend the notice");
    notice.observe(None,3000);
    notice.observe(Some((2,"Charging: 58%".into())),3100);
    assert!(bounds(&notice,3450).2>face.w as f32*1.02,"a new connection should pulse again");
}

#[test]
fn charging_energy_flows_towards_the_bolt_then_settles_inside_the_pill() {
    let canvas=Canvas::FRAMED;
    for light in [false,true] {
        let face=slot_ui::settings_notice_face_on("Charging: 57%",canvas,light);
        let texture=Some((TexId::from_raw(1),face.w,face.h));
        let mut notice=SettingsNotice::default();notice.observe(Some((1,"Charging: 57%".into())),0);
        let mut previous=1.0;
        for now in [220,320,420,520] {
            let mut out=vec![];notice.draw_on(canvas,texture,now,light,&mut out);
            let Draw::RoundRect{x,y,w,h,..}=out[0] else {panic!("missing capsule")};
            let energy:Vec<_>=out.iter().filter_map(|d|match *d {Draw::RoundRect{x:xx,y:yy,w:ww,h:hh,colour,..}if colour[..3]==[1.0;3]=>Some((xx,yy,ww,hh,colour[3])),_=>None}).collect();
            assert!(!energy.is_empty());assert!(energy.len()<=8,"too many glow quads");
            let position=(energy[0].0+energy[0].2/2.0-x)/w;
            assert!(position<previous,"energy moved away from the bolt");previous=position;
            for (xx,yy,ww,hh,a) in energy {assert!(xx>=x&&yy>=y&&xx+ww<=x+w&&yy+hh<=y+h);assert!(a>0.0&&a<0.02);}
            assert_eq!(out.iter().filter(|d|matches!(d,Draw::Tex{alpha:1.0,..})).count(),1,"keep one clean text layer");
        }
        for now in [1200,1800] {
            let mut out=vec![];notice.draw_on(canvas,texture,now,light,&mut out);
            assert!(!out.iter().any(|d|matches!(d,Draw::RoundRect{colour,..}if colour[..3]==[1.0;3])),"energy should settle");
        }
    }
}
