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
fn status_indicators_have_no_background_surfaces() {
    use slot_ui::{draw_status, Canvas, Draw, Printed, TexId};
    let mut out = vec![];
    draw_status(
        Canvas::FRAMED,
        None,
        Printed::default(),
        None,
        Printed::new(TexId::from_raw(0), 100),
        [255; 3],
        &mut out,
    );
    assert!(out.iter().all(|d| !matches!(d, Draw::RoundRect { .. })));
}
#[test]
fn status_reveal_preserves_icon_centres_and_bounds_opacity() {
    use slot_ui::{expressive::StatusMotion, Draw, TexId};
    let mut m = StatusMotion::default();
    m.update(0, 1.0);
    m.update(3, 0.0);
    for _ in 0..60 {
        m.update(3, 1.0 / 60.0);
        let mut draws = [Draw::Tex {
            x: 90.0,
            y: 10.0,
            w: 20.0,
            h: 20.0,
            tex: TexId::from_raw(0),
            alpha: 1.0,
        }];
        m.apply(&mut draws);
        if let Draw::Tex {
            x, y, w, h, alpha, ..
        } = draws[0]
        {
            assert!((x + w / 2.0 - 100.0).abs() < 0.01 && (y + h / 2.0 - 20.0).abs() < 0.01);
            assert!(w <= 21.0 && h <= 21.0 && (0.0..=1.0).contains(&alpha));
        }
    }
}

#[test]
fn child_pages_slide_forward_and_back_without_fading() {
    let mut m = MenuMotion::default();
    m.update(Some(QuickRow::General), &[], None, 1.0);
    m.update(Some(QuickRow::Wallpaper), &[], None, 0.0);
    assert_eq!(m.outgoing_row(), Some(QuickRow::General));
    assert_eq!(m.frame(QuickRow::Wallpaper, &[]).page_shift, 1.0);
    assert_eq!(m.frame(QuickRow::Wallpaper, &[]).opacity, 1.0);
    m.update(Some(QuickRow::Wallpaper), &[], None, 0.08);
    let entering = m.frame(QuickRow::Wallpaper, &[]);
    let leaving = m.outgoing_frame(QuickRow::General, &[]);
    assert!(entering.page_shift > 0.0 && entering.page_shift < 1.0 && leaving.page_shift < 0.0);
    assert!((entering.page_shift - leaving.page_shift - 1.0).abs() < 0.001);
    m.update(Some(QuickRow::General), &[], None, 0.0);
    assert_eq!(m.frame(QuickRow::General, &[]).page_shift, -1.0);
    m.update(Some(QuickRow::General), &[], None, 1.0);
    assert_eq!(m.outgoing_row(), None);
}
#[test]
fn cached_submenus_reverse_and_retarget_the_same_page_motion() {
    let mut m = MenuMotion::default();
    m.update(Some(QuickRow::Bluetooth), &[], None, 1.0);
    m.update_page(Some(QuickRow::Bluetooth), &[], None, Some(1000), 0.0);
    assert!(!m.outgoing_is_custom());
    assert_eq!(
        m.frame_page(QuickRow::Bluetooth, &[], Some(1000))
            .page_shift,
        1.0
    );
    m.update_page(Some(QuickRow::Bluetooth), &[], None, Some(1001), 0.05);
    assert!(m.outgoing_is_custom());
    m.update_page(Some(QuickRow::Bluetooth), &[], None, Some(1000), 0.0);
    assert_eq!(
        m.frame_page(QuickRow::Bluetooth, &[], Some(1000))
            .page_shift,
        -1.0
    );
    m.update_page(Some(QuickRow::Bluetooth), &[], None, None, 0.0);
    assert_eq!(m.frame(QuickRow::Bluetooth, &[]).page_shift, -1.0);
}
#[test]
fn selection_scrollbar_and_window_share_continuous_spring_positions() {
    let mut m = MenuMotion::default();
    m.update(Some(QuickRow::General), &[], None, 1.0);
    m.update(Some(QuickRow::Power), &[], None, 0.03);
    let f = m.frame(QuickRow::Power, &[]);
    assert!(f.scroll_position > 0.0 && f.scroll_position < QuickRow::Power.position(&[]) as f32);
    assert!(
        f.window_start > 0.0
            && f.window_start < slot_ui::quick_window(QuickRow::Power, &[]).0 as f32
    );
    m.update(Some(QuickRow::Power), &[], None, 1.0);
    assert_eq!(
        m.frame(QuickRow::Power, &[]).scroll_position,
        QuickRow::Power.position(&[]) as f32
    );
}
#[test]
fn interface_copy_preserves_case_and_uses_sentence_case_for_authored_labels() {
    use slot_ui::{menu_sentence_case, quick_label_face, quick_value_face, text};
    for (from, to) in [
        ("Fast Forward Sound", "Fast forward sound"),
        ("Kid Mode", "Kid mode"),
        ("USB Audio", "USB audio"),
        ("Left Stick as D-Pad", "Left stick as D-Pad"),
    ] {
        assert_eq!(menu_sentence_case(from), to);
    }
    let font = text::font(text::Role::Menu).unwrap();
    assert_eq!(
        text::fit_preserving_case(font, "Internal speaker", 400.0, 1, 26.0, 26.0).lines,
        vec!["Internal speaker"]
    );
    assert_eq!(
        quick_label_face(QuickRow::FastForwardSound, false).rgba,
        quick_value_face("Fast forward sound", false).rgba
    );
}

#[test]
fn cached_settings_share_the_same_native_grid_and_only_one_selection_surface() {
    use slot_ui::{parent_settings_face_on, Canvas};
    for width in [640, 720] {
        let face = parent_settings_face_on(
            Canvas {
                w: width as f32,
                h: 480.0,
            },
            &[
                ("Kid mode".into(), "On".into(), true),
                ("Play limit".into(), "Daily".into(), true),
            ],
            0,
            0,
            "",
            0.0,
        );
        assert_eq!(face.w, width);
        let alpha = |x, y| face.rgba[((y * width + x) * 4 + 3) as usize];
        assert_eq!(alpha(60, 110), 255);
        assert_eq!(alpha(60, 162), 0, "unselected rows have no backing card");
        for y in 88..184 {
            for x in (0..56).chain(width - 56..width) {
                assert_eq!(alpha(x, y), 0, "body edge padding");
            }
        }
    }
}
