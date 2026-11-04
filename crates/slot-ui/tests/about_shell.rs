use slot_ui::{AboutMotion, Canvas, Draw, TexId};

fn quad(draw: &Draw) -> [f32; 4] {
    match *draw {
        Draw::Tex { x, y, w, h, .. } | Draw::Part { x, y, w, h, .. }
        | Draw::RoundRect { x, y, w, h, .. } => [x, y, w, h],
        _ => panic!("not a quad"),
    }
}

#[test]
fn reversal_retains_pose_and_velocity_and_settles_without_overshoot() {
    let mut motion = AboutMotion::default();
    motion.update(true, 0.08);
    let at = motion.value();
    motion.update(false, 0.0);
    assert_eq!(motion.value(), at);
    // Physical inertia is retained on a mid-flight reversal.
    motion.update(false, 0.001);
    assert!(motion.value() > at);
    for i in 0..100 {
        motion.update(i < 10, 1.0 / 60.0);
        assert!((0.0..=1.0).contains(&motion.value()));
    }
    assert_eq!(motion.value(), 0.0);
    assert!(!motion.active());
}

#[test]
fn shell_sticker_and_slot_keep_one_rigid_transform_at_both_widths() {
    for w in [640.0, 720.0] {
        let canvas = Canvas { w, h: 480.0 };
        let mut source = vec![];
        slot_ui::draw_empty_slot(canvas, Some((TexId::from_raw(1), TexId::from_raw(2))), &mut source);
        slot_ui::draw_about_body(canvas, Some(TexId::from_raw(3)), &mut source);
        slot_ui::draw_about_sticker(canvas, Some(TexId::from_raw(4)), &mut source);
        let rest = source.iter().map(quad).collect::<Vec<_>>();
        let mut motion = AboutMotion::default();
        for _ in 0..45 {
            motion.update(true, 1.0 / 60.0);
            let mut drawn = source.clone();
            motion.transform(canvas, &mut drawn);
            let placed = drawn.iter().map(quad).collect::<Vec<_>>();
            let scale = placed[0][2] / rest[0][2];
            for (old, new) in rest.iter().zip(&placed) {
                assert!((new[2] / old[2] - scale).abs() < 0.0001);
                assert!((new[3] / old[3] - scale).abs() < 0.0001);
                assert!((new[1] - placed[0][1] - (old[1] - rest[0][1]) * scale).abs() < 0.001);
                assert!((new[0] + new[2] / 2.0 - w / 2.0).abs() < 0.001);
            }
        }
    }
}

#[test]
fn parent_moves_with_the_shell_but_its_chrome_stays_anchored() {
    let canvas = Canvas { w: 640.0, h: 480.0 };
    let mut draws = vec![
        Draw::RoundRect { x: 56.0, y: 56.0, w: 528.0, h: 310.0, radius: 28.0, colour: [1.0; 4] },
        Draw::Tex { x: 270.0, y: 0.0, w: 100.0, h: 56.0, tex: TexId::from_raw(1), alpha: 1.0 },
    ];
    let title = draws[1];
    let mut motion = AboutMotion::default();
    motion.update(true, 1.0);
    motion.transform_parent(canvas, &mut draws);
    assert_eq!(draws[1], title);
    let [x, y, w, _] = quad(&draws[0]);
    assert!((w - 528.0 * 0.82).abs() < 0.001);
    assert!((x + w / 2.0 - 320.0).abs() < 0.001);
    assert!(y > 300.0);
}

#[test]
fn tall_cartridge_is_cropped_at_the_header_without_changing_its_texture_scale() {
    let canvas = Canvas { w: 640.0, h: 480.0 };
    let mut motion = AboutMotion::default();
    motion.update(true, 1.0);
    let mut draws = vec![Draw::Tex { x: 80.0, y: -100.0, w: 400.0, h: 500.0, tex: TexId::from_raw(8), alpha: 1.0 }];
    motion.clip_hardware(canvas, &mut draws, 0);
    let Draw::Part { x, y, w, h, uv, .. } = draws[0] else { panic!("expected clipped cartridge"); };
    assert_eq!([x, y, w, h], [80.0, 56.0, 400.0, 344.0]);
    assert!((h / uv[3] - 500.0).abs() < 0.001);
    assert_eq!(uv[0], 0.0);
    assert_eq!(uv[2], 1.0);
}
