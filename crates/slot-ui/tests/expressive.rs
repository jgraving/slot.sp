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
fn opaque_pages_travel_offscreen_before_retiring_without_blocking_input() {
    let mut m = MenuMotion::default();
    m.update(Some(QuickRow::General), &[], None, 0.0);
    for _ in 0..60 {
        m.update(Some(QuickRow::General), &[], None, 1.0 / 60.0);
        let f = m.frame(QuickRow::General, &[]);
        assert_eq!(f.opacity,1.0);
        assert_eq!(f.content_opacity,1.0);
        assert!(m.displacement(slot_ui::Canvas::FRAMED)>=0.0);
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
    assert!((m.frame(QuickRow::General, &[]).page_shift-leaving.page_shift).abs()<0.001);
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
    let old=m.outgoing_frame(QuickRow::Bluetooth,&[]).page_shift;
    m.update_page(Some(QuickRow::Bluetooth), &[], None, Some(1000), 0.0);
    assert!((m.frame_page(QuickRow::Bluetooth,&[],Some(1000)).page_shift-old).abs()<0.001);
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

#[test]
fn pause_selector_expands_and_reverses_without_jumping() {
    let mut motion = MenuMotion::default();
    motion.update(Some(QuickRow::Settings), &[], None, 1.0);
    let start = motion.frame(QuickRow::Settings, &[]);
    assert_eq!(start.focus_compact, 1.0);
    motion.update(Some(QuickRow::PlayScreen), &[], None, 0.0);
    let first = motion.frame(QuickRow::PlayScreen, &[]);
    assert_eq!(first.focus_y, start.focus_y);
    assert_eq!(first.focus_compact, 1.0);
    motion.update(Some(QuickRow::PlayScreen), &[], None, 0.05);
    let middle = motion.frame(QuickRow::PlayScreen, &[]);
    assert!(middle.focus_compact > 0.0 && middle.focus_compact < 1.0);
    assert!(middle.page_shift>0.0 && middle.page_shift<1.0);
    assert_eq!(middle.content_opacity,1.0);
    assert!(
        motion
            .outgoing_frame(QuickRow::Settings, &[])
            .suppress_focus
    );
    motion.update(Some(QuickRow::Settings), &[], None, 0.0);
    let reversed = motion.frame(QuickRow::Settings, &[]);
    assert!((reversed.focus_compact - middle.focus_compact).abs() < 0.001);
    assert!((reversed.focus_y - middle.focus_y).abs() < 0.001);
    motion.update(Some(QuickRow::Settings), &[], None, 1.0);
    assert_eq!(motion.frame(QuickRow::Settings, &[]).focus_compact, 1.0);
    assert!(!QuickRow::Resume.disclosure());
    assert!(!QuickRow::ExitGame.disclosure());
    assert!(QuickRow::Settings.disclosure());
}

#[test]
fn independent_cached_switches_and_sliders_animate_continuously() {
    use slot_ui::{
        expressive::{SliderMotion, SwitchMotion},
        MenuSlider, MenuSwitch,
    };
    let mut switches = SwitchMotion::default();
    let mut sliders = SliderMotion::default();
    let controls = [
        MenuSwitch {
            id: 100,
            on: false,
            y: 94.0,
        },
        MenuSwitch {
            id: 200,
            on: true,
            y: 146.0,
        },
    ];
    switches.update(&controls, 0.0);
    let changed = [
        MenuSwitch {
            on: true,
            ..controls[0]
        },
        controls[1],
    ];
    switches.update(&changed, 0.04);
    let before = switches.value(100, true);
    assert!(before > 0.0 && before < 1.0);
    assert_eq!(switches.value(200, true), 1.0);
    switches.update(&controls, 0.0);
    assert_eq!(switches.value(100, false), before);
    switches.update(&controls, 1.0);
    assert_eq!(switches.value(100, false), 0.0);
    sliders.update(
        &[MenuSlider { width:slot_ui::MENU_SLIDER_W,span:slot_ui::MENU_SLIDER_SPAN,
            id: 1,
            fraction: 0.0,
            y: 0.0,
        }],
        0.0,
    );
    sliders.update(
        &[MenuSlider { width:slot_ui::MENU_SLIDER_W,span:slot_ui::MENU_SLIDER_SPAN,
            id: 1,
            fraction: 1.0,
            y: 0.0,
        }],
        0.04,
    );
    let before = sliders.value(1, 1.0);
    assert!(before > 0.0 && before < 1.0);
    sliders.update(
        &[MenuSlider { width:slot_ui::MENU_SLIDER_W,span:slot_ui::MENU_SLIDER_SPAN,
            id: 1,
            fraction: 0.0,
            y: 0.0,
        }],
        0.0,
    );
    assert_eq!(sliders.value(1, 0.0), before);
    sliders.update(&[], 1.0);
    assert_eq!(sliders.value(1, 0.0), 0.0);
}

#[test]
fn details_and_clock_content_fit_both_native_canvases() {
    use slot_ui::{reference_face, Canvas, ClockPicker};
    for w in [640, 720] {
        let canvas = Canvas {
            w: w as f32,
            h: 480.0,
        };
        let rows = [
            ("Last played", "Oct 30, 2026 23:59"),
            (
                "Game name",
                "A very long game title with additional release information",
            ),
            ("Play time", "123 hours 59 minutes"),
        ]
        .map(|(a, b)| (a.into(), b.into(), false));
        let details = reference_face(canvas, "Game details", &rows, 0, 1);
        let clock = ClockPicker::local(1_800_000_000, 840).face_on(canvas);
        for face in [&details, &clock] {
            for y in 40..370 {
                for x in 0..w {
                    if x < 80 || x >= w - 80 {
                        assert_eq!(
                            face.rgba[((y * w + x) * 4 + 3) as usize],
                            0,
                            "content outside safe ink lane at {x},{y}, width {w}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn pause_settings_is_centered_and_the_card_expands_to_the_whole_canvas() {
    use slot_ui::*;
    let hidden = [QuickRow::UndoSave, QuickRow::UndoLoad];
    for w in [640.0,720.0] {
        let canvas=Canvas {w,h:480.0};
        let frame=MenuFrame::settled(QuickRow::Settings,&hidden);
        assert_eq!(frame.focus_y + expressive::ROW_H / 2.0,240.0);
        let mut out=vec![];
        expressive::draw_pause_surface(canvas,frame,&hidden,&mut out);
        assert!(matches!(out.last(),Some(Draw::RoundRect {w,h,..}) if *w==384.0 && *h<480.0));
        assert!(out.iter().all(|d| matches!(d,Draw::RoundRect {y,..} if *y >= 40.0)), "pause card and shadow must clear the status bar");
        let mut full=vec![];
        expressive::draw_pause_surface(canvas,MenuFrame::settled(QuickRow::PlayScreen,&hidden),&hidden,&mut full);
        assert!(matches!(full.last(),Some(Draw::RoundRect {x,y,w:ww,h,radius,..}) if *x==0.0 && *y==0.0 && *ww==w && *h==480.0 && *radius==0.0));
    }
}

#[test]
fn kid_card_expansion_uses_the_shared_interruptible_spatial_spring() {
    use slot_ui::expressive::SurfaceMotion;
    let mut reference=0.0;
    for hz in [30,60,120] {
        let mut motion=SurfaceMotion::default(); motion.snap(true);
        for _ in 0..hz/5 {motion.update(false,1.0/hz as f32);}
        if hz==30 {reference=motion.value();} else {assert!((motion.value()-reference).abs()<0.01);}
        let before=motion.value(); motion.update(true,0.0); assert_eq!(motion.value(),before);
        for _ in 0..hz {motion.update(true,1.0/hz as f32);}
        assert!((motion.value()-1.0).abs()<0.001);
    }
}

#[test]
fn color_temperature_has_one_neutral_to_warm_vocabulary() {
    use slot_ui::{QuickValue, setting_slider};
    assert_eq!(QuickValue::temperature(0).text(),"Neutral");
    assert_eq!(QuickValue::temperature(9).text(),"Warm 100%");
    for n in 0..=9 {
        let text=QuickValue::temperature(n).text();
        let fraction=setting_slider("Color temp",text).unwrap();
        assert!((fraction-n as f32/9.0).abs()<0.01);
    }
}

#[test]
fn opening_and_closing_reverse_at_the_same_position_without_losing_velocity() {
    let canvas=slot_ui::Canvas::FRAMED;
    let mut m=MenuMotion::default();
    m.update(Some(QuickRow::General),&[],None,0.0);
    assert!(m.displacement(canvas)>canvas.h);
    m.update(Some(QuickRow::General),&[],None,0.07);
    let y=m.displacement(canvas);
    m.update(None,&[],None,0.0);
    assert_eq!(m.displacement(canvas),y);
    m.update(None,&[],None,0.04);
    let closing=m.displacement(canvas);
    m.update(Some(QuickRow::General),&[],None,0.0);
    assert_eq!(m.displacement(canvas),closing);
    m.update(Some(QuickRow::General),&[],None,1.0);
    assert_eq!(m.displacement(canvas),0.0);
    m.update(None,&[],None,0.1);
    assert!(m.closing_row().is_some());
    m.update(None,&[],None,1.0);
    assert!(m.displacement(canvas)>canvas.h);
    assert!(m.closing_row().is_none());
}

#[test]
fn back_during_page_travel_reuses_both_pages_at_their_current_positions() {
    let mut motion=MenuMotion::default();
    motion.update(Some(QuickRow::General),&[],None,1.0);
    motion.update(Some(QuickRow::Wallpaper),&[],None,0.06);
    let incoming=motion.frame(QuickRow::Wallpaper,&[]).page_shift;
    let outgoing=motion.outgoing_frame(QuickRow::General,&[]).page_shift;
    motion.update(Some(QuickRow::General),&[],None,0.0);
    assert!((motion.frame(QuickRow::General,&[]).page_shift-outgoing).abs()<0.001);
    assert!((motion.outgoing_frame(QuickRow::Wallpaper,&[]).page_shift-incoming).abs()<0.001);
    assert_eq!(motion.frame(QuickRow::General,&[]).opacity,1.0);
}

#[test]
fn kid_pause_card_and_settings_row_share_the_screen_centre() {
    use slot_ui::*;
    for w in [640.0,720.0] {
        let canvas=Canvas{w,h:480.0};
        let mut motion=MenuMotion::default();
        motion.update_page_at(Some(QuickRow::Kids),&[],None,Some(2001),Some((1,0)),1.0);
        let mut frame=motion.frame_page(QuickRow::Kids,&[],Some(2001));frame.focus_compact=1.0;
        assert_eq!(frame.focus_y+expressive::ROW_H/2.0,240.0);
        let mut out=vec![];expressive::draw_kid_pause_surface(canvas,frame,&mut out);
        assert!(matches!(out.last(),Some(Draw::RoundRect{x,y,w:ww,h,..}) if *x+*ww/2.0==w/2.0 && *y+*h/2.0==240.0));
    }
}

#[test]
fn full_kid_menu_focus_uses_its_own_row_origin() {
    use slot_ui::*;
    for selected in 0..3 {
        let mut motion=MenuMotion::default();
        motion.update_page_at(Some(QuickRow::Kids),&[],None,Some(2010),Some((selected,0)),1.0);
        let frame=motion.frame_page(QuickRow::Kids,&[],Some(2010));
        assert_eq!(frame.focus_y,88.0+selected as f32*QUICK_PITCH);
    }
}
