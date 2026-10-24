use slot_ui::{draw_power_switch, power_switch_faces, Canvas, Draw, TexId};

#[test]
fn detents_and_fade_stay_aligned_on_each_canvas() {
    for width in [480.0, 640.0, 720.0, 854.0] {
        let canvas = Canvas { w: width, h: 480.0 };
        let scale = width / 300.0;
        for (ms, mark, alpha) in [
            (0.0, 525.0, 1.0),
            (150.0, 525.0, 1.0),
            (240.0, 501.0, 1.0),
            (330.0, 477.0, 1.0),
            (500.0, 477.0, 1.0),
            (550.0, 477.0, 0.5),
            (600.0, 477.0, 0.0),
        ] {
            let mut draws = Vec::new();
            draw_power_switch(
                canvas,
                ms / slot_ui::POWER_SWITCH_MS as f32,
                [TexId::from_raw(0), TexId::from_raw(1)],
                &mut draws,
            );
            assert_eq!(draws.len(), 2);
            let Draw::Tex { x, w, alpha: a, .. } = draws[0] else {
                panic!()
            };
            assert!((x + w / 2.0 - width / 2.0).abs() < 0.001);
            assert!(x < 0.0 && x + w > width);
            assert!((a - alpha).abs() < 0.001);
            let Draw::Tex {
                x: sx,
                w: sw,
                alpha: sa,
                ..
            } = draws[1]
            else {
                panic!()
            };
            assert!((sx + sw * 0.5 - (x + mark * scale)).abs() < 0.001);
            assert_eq!(sa, a);
            assert!((sw / width - 0.3).abs() < 0.001, "slider must stay prominent");
        }
    }
}

#[test]
fn carousel_finishes_share_geometry_and_a_solid_neutral_slider() {
    let [dark, slider] = power_switch_faces(false);
    let [light, light_slider] = power_switch_faces(true);
    assert_eq!((dark.w, dark.h), (1000, 400));
    assert_eq!(slider.rgba, light_slider.rgba);
    assert_eq!(dark.rgba[3], 0);
    assert!(dark
        .rgba
        .chunks_exact(4)
        .zip(light.rgba.chunks_exact(4))
        .all(|(a, b)| a[3] == b[3]));
    let at = ((210 * dark.w + 100) * 4) as usize;
    assert!(light.rgba[at] > dark.rgba[at] + 60);
    assert_eq!(dark.rgba[at + 3], 255);
    let at = ((40 * slider.w + 40) * 4) as usize;
    assert_eq!(slider.rgba[at + 3], 255);
}

#[test]
fn every_selected_finish_tints_the_side_with_the_carousel_palette() {
    let mut seen = Vec::new();
    let mut neutral_slider = None;
    for colour in slot_store::ShellColour::FINISHES {
        let [housing, slider] = slot_ui::power_switch_faces_for(colour);
        let at = ((210 * housing.w + 100) * 4) as usize;
        let pixel = &housing.rgba[at..at + 4];
        assert_eq!(pixel[3], 255);
        assert!(!seen.contains(&pixel[..3].to_vec()), "finish did not change the housing");
        seen.push(pixel[..3].to_vec());
        if let Some(ref expected) = neutral_slider {
            assert_eq!(expected, &slider.rgba);
        } else {
            neutral_slider = Some(slider.rgba);
        }
    }
}
