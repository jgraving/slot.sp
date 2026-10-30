use slot_ui::{Canvas, ClockPicker, Draw, Field, MenuFrame, MenuMotion, QuickRow, TexId};

#[test]
fn clock_navigation_is_spatial_and_separate_from_editing() {
    let mut picker = ClockPicker::from_ymd(2026, 10, 26);
    let original = picker.secs();
    for (dx, dy, field) in [
        (1, 0, Field::Month),
        (0, 1, Field::Minute),
        (1, 0, Field::Offset),
        (0, -1, Field::Day),
        (-1, 0, Field::Month),
    ] {
        picker.move_selection(dx, dy);
        assert_eq!(picker.cursor(), field);
        assert_eq!(picker.secs(), original);
    }
    assert_eq!(picker.month_name(), "October");
    picker.toggle_edit();
    picker.up();
    assert_eq!(picker.month_name(), "November");
    assert_eq!(picker.cursor(), Field::Month);
    picker.finish_edit();
    assert!(!picker.editing());
    for w in [640, 720] {
        let face = picker.face_on(Canvas {
            w: w as f32,
            h: 480.0,
        });
        for y in 310..355 {
            assert!(
                face.rgba[(y * w * 4) as usize..((y + 1) * w * 4) as usize]
                    .chunks_exact(4)
                    .all(|p| p[3] == 0),
                "no instruction line below the fields"
            );
        }
    }
}

#[test]
fn settings_stays_centered_and_the_pause_surface_does_not_follow_selection() {
    for hidden in [
        vec![QuickRow::UndoSave, QuickRow::UndoLoad],
        vec![
            QuickRow::UndoSave,
            QuickRow::UndoLoad,
            QuickRow::States,
            QuickRow::Screenshots,
        ],
    ] {
        let y = slot_ui::quick_row_top_at(QuickRow::Settings, QuickRow::Resume, &hidden);
        assert_eq!(y + 22.0, 240.0);
        let mut motion = MenuMotion::default();
        motion.update(Some(QuickRow::Resume), &hidden, None, 1.0);
        let surface = |frame| {
            let mut out = vec![];
            slot_ui::expressive::draw_game_menu_surface(Canvas::FRAMED, frame, &mut out);
            out
        };
        let before = surface(motion.frame(QuickRow::Resume, &hidden));
        motion.update(Some(QuickRow::Settings), &hidden, None, 0.05);
        assert_eq!(
            surface(motion.frame(QuickRow::Settings, &hidden)),
            before,
            "background must not jump with focus"
        );
        motion.update(Some(QuickRow::PlayScreen), &hidden, None, 0.06);
        let middle = motion.frame(QuickRow::PlayScreen, &hidden);
        let Draw::RoundRect { w, .. } = surface(middle).last().unwrap().clone() else {
            panic!()
        };
        assert!(w > 352.0 && w < 720.0);
        motion.update(Some(QuickRow::Settings), &hidden, None, 0.0);
        assert_eq!(
            surface(motion.frame(QuickRow::Settings, &hidden)),
            surface(middle),
            "interrupted surface expansion stays continuous"
        );
    }
}

#[test]
fn full_percentages_have_a_clear_lane_after_the_slider() {
    for width in [640, 720] {
        let canvas = Canvas {
            w: width as f32,
            h: 480.0,
        };
        let face = slot_ui::parent_settings_face_animated_on(
            canvas,
            &[("Night warmth".into(), "100%".into(), true)],
            0,
            0,
            "",
            0.0,
        );
        let mut draws = vec![];
        slot_ui::draw_menu_sliders(
            canvas,
            &[slot_ui::MenuSlider {
                id: 1,
                fraction: 1.0,
                y: 94.0,
            }],
            &Default::default(),
            MenuFrame::settled(QuickRow::DisplaySchedule, &[]),
            &mut draws,
        );
        let end = draws
            .iter()
            .find_map(|d| match d {
                Draw::RoundRect { x, w, h, .. } if *h == 6.0 => Some(x + w),
                _ => None,
            })
            .unwrap();
        let text = (end.ceil() as u32..width - 80)
            .find(|x| {
                (89..129).any(|y| {
                    let i = ((y * width + x) * 4) as usize;
                    face.rgba[i] > 220 && face.rgba[i + 1] > 220 && face.rgba[i + 3] > 180
                })
            })
            .unwrap();
        assert!(
            text as f32 >= end + 16.0,
            "100% needs breathing room after the thumb"
        );
    }
}

#[test]
fn arbitrary_settings_notices_fade_in_a_bottom_pill_then_expire() {
    let mut hud = slot_ui::Hud::new();
    hud.notice("Bluetooth could not connect. Try again.", 0);
    hud.set_notice_face((TexId::from_raw(90), 340, 26));
    let draw = |hud: &slot_ui::Hud, now| {
        let mut out = vec![];
        hud.draw(Canvas::FRAMED, now, &mut out);
        out
    };
    let alpha = |out: Vec<Draw>| {
        out.into_iter()
            .find_map(|d| match d {
                Draw::Tex { alpha, .. } => Some(alpha),
                _ => None,
            })
            .unwrap()
    };
    assert!(alpha(draw(&hud, 80)) < alpha(draw(&hud, 200)));
    let out = draw(&hud, 200);
    let (x, y, w, h) = out
        .iter()
        .find_map(|d| match d {
            Draw::RoundRect { x, y, w, h, .. } => Some((*x, *y, *w, *h)),
            _ => None,
        })
        .unwrap();
    assert_eq!(x + w / 2.0, 360.0);
    assert!(y > 250.0 && y + h < 370.0);
    assert!(!out.iter().any(|d| matches!(d,Draw::Rect{w,..}if *w==720.0)));
    assert!(draw(&hud, 4000).is_empty());
}

#[test]
fn bigger_keycaps_keep_white_space_and_long_hint_rows_fit() {
    assert_eq!(slot_ui::MENU_HINT_H, 40);
    let face = slot_ui::button_code_face(&[0]);
    let columns = (0..80)
        .filter(|x| (0..64).any(|y| face.rgba[((y * 320 + x) * 4 + 3) as usize] > 128))
        .count();
    assert!(columns >= 40 && columns <= 56);
    let hints = slot_ui::quick_legend_faces([246, 244, 239]);
    let total = [0, 7, 6, 5]
        .into_iter()
        .map(|i| hints[i].w - slot_ui::HINT_EDGE)
        .sum::<u32>();
    assert!(
        total + 3 * 24 <= 640 - 32,
        "long binding footer retains at least 24px gaps"
    );
}
