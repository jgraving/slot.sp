use slot_ui::{quick_row_top_at, MenuMotion, QuickRow};

fn moving(hz: usize) -> (MenuMotion, f32) {
    let mut m = MenuMotion::default();
    m.update(Some(QuickRow::General), &[], None, 1.0);
    let target = quick_row_top_at(QuickRow::Screen, QuickRow::Screen, &[]);
    for _ in 0..hz / 5 {
        m.update(Some(QuickRow::Screen), &[], None, 1.0 / hz as f32);
    }
    (m, target)
}
#[test]
fn focus_matches_across_frame_rates_and_survives_dropped_frames() {
    let (reference, target) = moving(60);
    let y = reference.frame(QuickRow::Screen, &[]).focus_y;
    for hz in [30, 120] {
        let (m, _) = moving(hz);
        assert!((m.frame(QuickRow::Screen, &[]).focus_y - y).abs() < 0.02);
    }
    let mut m = reference;
    for dt in [0.25, 0.5, 1.0] {
        m.update(Some(QuickRow::Screen), &[], None, dt);
    }
    assert!((m.frame(QuickRow::Screen, &[]).focus_y - target).abs() < 0.01);
}
#[test]
fn reversals_keep_momentum_while_selection_is_immediately_actionable() {
    let (mut m, _) = moving(60);
    let before = m.frame(QuickRow::Screen, &[]).focus_y;
    m.update(Some(QuickRow::General), &[], None, 0.0);
    assert_eq!(
        m.frame(QuickRow::General, &[]).focus_y,
        before,
        "retargeting jumped the shape"
    );
    for _ in 0..60 {
        m.update(Some(QuickRow::General), &[], None, 1.0 / 60.0);
    }
    assert!(
        (m.frame(QuickRow::General, &[]).focus_y
            - quick_row_top_at(QuickRow::General, QuickRow::General, &[]))
        .abs()
            < 0.01
    );
}
#[test]
fn pages_arrive_with_bounded_opacity_and_closing_finishes_without_blocking_input() {
    let mut m = MenuMotion::default();
    m.update(Some(QuickRow::General), &[], None, 0.0);
    let mut last = 0.0;
    for _ in 0..60 {
        m.update(Some(QuickRow::General), &[], None, 1.0 / 60.0);
        let f = m.frame(QuickRow::General, &[]);
        assert!(f.opacity >= last && f.opacity <= 1.0);
        last = f.opacity;
    }
    m.update(None, &[], None, 1.0 / 60.0);
    assert_eq!(m.closing_row(), Some(QuickRow::General));
    for _ in 0..60 {
        m.update(None, &[], None, 1.0 / 60.0);
    }
    assert_eq!(m.closing_row(), None);
    m.update(Some(QuickRow::Brightness), &[], None, 1.0 / 60.0);
    let f = m.frame(QuickRow::Brightness, &[]);
    assert_eq!(
        f.focus_y,
        quick_row_top_at(QuickRow::Brightness, QuickRow::Brightness, &[])
    );
    assert!(!f.closing);
}
#[test]
fn switch_interruption_stays_in_its_track_and_settles_exactly() {
    let mut m = MenuMotion::default();
    m.update(Some(QuickRow::Rumble), &[], Some(false), 1.0);
    for on in [true, false, true, false, true] {
        for _ in 0..4 {
            m.update(Some(QuickRow::Rumble), &[], Some(on), 1.0 / 60.0);
            assert!((0.0..=1.0).contains(&m.frame(QuickRow::Rumble, &[]).switch.unwrap()));
        }
    }
    m.update(Some(QuickRow::Rumble), &[], Some(true), 1.0);
    assert_eq!(m.frame(QuickRow::Rumble, &[]).switch, Some(1.0));
}

#[test]
fn status_capsules_fit_both_canvases_and_do_not_leave_empty_chrome() {
    use slot_ui::{draw_status_surfaces,Canvas,Draw,Printed};
    use slot_power::{Battery,Charge};
    for width in [640.0,720.0] {
        for ink in [[246,244,239],[32,32,36]] {
            let canvas=Canvas{w:width,h:480.0};
            for connection in [false,true] {for mute in [false,true] {
                let mut out=vec![];
                draw_status_surfaces(canvas,Some(Battery{percent:100,charge:Charge::Charging}),
                    Printed{face:None,w:64},Printed{face:None,w:112},connection,mute,ink,&mut out);
                assert_eq!(out.len(),2);
                let boxes:Vec<_>=out.iter().map(|d|match d {
                    Draw::RoundRect{x,y,w,h,colour,..}=>{assert!(*y>=0.0 && y+h<=40.0 && colour[3]>0.5);(*x,*x+*w)},
                    _=>panic!("status surface should be a single quad"),
                }).collect();
                assert!(boxes[0].0>=0.0 && boxes[0].1<boxes[1].0 && boxes[1].1<=width);
            }}
            let mut out=vec![];
            draw_status_surfaces(canvas,None,Printed::default(),Printed::default(),false,false,ink,&mut out);
            assert!(out.is_empty());
        }
    }
}
#[test]
fn status_reveal_preserves_icon_centres_and_bounds_opacity() {
    use slot_ui::{Draw,TexId,expressive::StatusMotion};
    let mut m=StatusMotion::default();m.update(0,1.0);m.update(3,0.0);
    for _ in 0..60 {
        m.update(3,1.0/60.0);
        let mut draws=[Draw::Tex{x:90.0,y:10.0,w:20.0,h:20.0,tex:TexId::from_raw(0),alpha:1.0}];
        m.apply(&mut draws);
        if let Draw::Tex{x,y,w,h,alpha,..}=draws[0] {
            assert!((x+w/2.0-100.0).abs()<0.01 && (y+h/2.0-20.0).abs()<0.01);
            assert!(w<=21.0 && h<=21.0 && (0.0..=1.0).contains(&alpha));
        }
    }
}
