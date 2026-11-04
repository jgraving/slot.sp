use slot_store::ShellColour;
use slot_ui::{
    power_switch_faces_for, shell_about_body, shell_rgb, shell_slot_faces, ABOUT_BODY_H,
    ABOUT_BODY_TEXTURE_H, ABOUT_BODY_TEXTURE_W, ABOUT_BODY_W, SLOT_H, SLOT_TEXTURE_H,
    SLOT_TEXTURE_W, SLOT_W,
};

// Probe in logical display coordinates even though the raster is supersampled.
fn at(x: usize, y: usize) -> usize {
    ((y * (SLOT_TEXTURE_H / SLOT_H) as usize * SLOT_TEXTURE_W as usize
        + x * (SLOT_TEXTURE_W / SLOT_W) as usize)
        * 4) as usize
}

fn sample(rgba: &[u8], x: usize, y: usize) -> &[u8] {
    &rgba[at(x, y)..at(x, y) + 4]
}

#[test]
fn painted_shells_preserve_geometry_and_leave_the_switch_cavity_neutral() {
    let original = shell_slot_faces(ShellColour::Graphite);
    let [dark, slider] = power_switch_faces_for(ShellColour::Graphite);
    for colour in ShellColour::FINISHES {
        let faces = shell_slot_faces(colour);
        for layer in 0..2 {
            assert_eq!(
                faces[layer].len(),
                (SLOT_TEXTURE_W * SLOT_TEXTURE_H * 4) as usize
            );
            assert!(faces[layer]
                .chunks_exact(4)
                .zip(original[layer].chunks_exact(4))
                .all(|(a, b)| a[3] == b[3]));
        }
        let [switch, handle] = power_switch_faces_for(colour);
        assert_eq!(handle.rgba, slider.rgba);
        let cavity = ((208 * switch.w + 435) * 4) as usize;
        assert_eq!(
            &switch.rgba[cavity..cavity + 4],
            &dark.rgba[cavity..cavity + 4]
        );
        let base = shell_rgb(colour);
        for c in 0..3 {
            assert!((sample(faces[1], 240, 66)[c] as i16 - base[c] as i16).abs() < 20);
        }
    }
}

#[test]
fn paint_has_fine_low_contrast_grain_instead_of_deep_pebbles() {
    let front = shell_slot_faces(ShellColour::Platinum)[1];
    let values: Vec<f32> = (230..350).map(|x| sample(front, x, 66)[0] as f32).collect();
    let avg = values.iter().sum::<f32>() / values.len() as f32;
    let rms = (values.iter().map(|v| (v - avg).powi(2)).sum::<f32>() / values.len() as f32).sqrt();
    assert!((0.3..4.0).contains(&rms), "grain RMS {rms}");
}

#[test]
fn svg_slot_keeps_an_open_throat_and_shallow_bowed_lip() {
    for colour in ShellColour::FINISHES {
        let [back, front] = shell_slot_faces(colour);
        assert_eq!(
            sample(front, 360, 32)[3],
            0,
            "cartridge must pass through the throat"
        );
        assert_eq!(
            sample(front, 360, 53)[3],
            255,
            "lip must hide the seated cartridge foot"
        );
        let metal = sample(back, 360, 36);
        assert!(
            (115..210).contains(&metal[0]),
            "liner must read as silver metal"
        );
        assert_eq!(
            metal,
            sample(shell_slot_faces(ShellColour::Graphite)[0], 360, 36),
            "metal liner must not inherit the shell paint color"
        );
        assert!(
            sample(back, 268, 15)[3] > 250,
            "rear deck must remain opaque behind the ribs"
        );
        assert_eq!(
            sample(back, 359, 14),
            sample(shell_slot_faces(ShellColour::Graphite)[0], 359, 14),
            "small metal screw must stay neutral"
        );
    }
}

#[test]
fn lip_is_a_broad_continuous_scoop_without_a_long_flat_centre() {
    let front = shell_slot_faces(ShellColour::Platinum)[1];
    let top = |x: usize| {
        let px = x * (SLOT_TEXTURE_W / SLOT_W) as usize;
        (0..SLOT_TEXTURE_H as usize)
            .find(|y| front[(y * SLOT_TEXTURE_W as usize + px) * 4 + 3] > 200)
            .unwrap() as f32
            / (SLOT_TEXTURE_H / SLOT_H) as f32
    };
    assert!(
        (top(360) - 47.0).abs() < 1.0,
        "retain the cartridge occlusion plane"
    );
    assert!(top(240) < top(280) && top(280) < top(320) && top(320) < top(360));
    for x in (220..360).step_by(10) {
        assert!(top(x + 10) >= top(x), "left scoop reverses at {x}");
        assert!(
            (top(x) - top(720 - x)).abs() <= 0.5,
            "scoop is asymmetric at {x}"
        );
    }
    assert!(
        top(360) - top(320) >= 0.5,
        "the old long flat centre remains"
    );
}

#[test]
fn screw_edges_never_leak_the_svg_red_paint_marker() {
    for layer in shell_slot_faces(ShellColour::Graphite) {
        for pixel in layer.chunks_exact(4).filter(|p| p[3] > 0) {
            assert!(
                pixel[0] <= pixel[1] + 2,
                "red marker leaked at a metal/paint boundary: {pixel:?}"
            );
        }
    }
}

#[test]
fn lower_shell_stays_flat_without_a_bulge_or_cast_shadow() {
    for colour in ShellColour::FINISHES {
        let front = shell_slot_faces(colour)[1];
        let base = shell_rgb(colour);
        for x in 230..491 {
            for y in 53..71 {
                let p = sample(front, x, y);
                assert_eq!(p[3], 255);
                for c in 0..3 {
                    assert!(
                        (p[c] as i16 - base[c] as i16).abs() < 15,
                        "bulge or shadow below slot at {x},{y}: {p:?}"
                    );
                }
            }
        }
        assert_eq!(sample(front, 360, 36)[3], 0);
    }
}

#[test]
fn sp_body_has_rounded_shoulders_and_recessed_corner_screws() {
    for colour in ShellColour::FINISHES {
        let [back, front] = shell_slot_faces(colour);
        for x in [40usize, 150, 570, 680] {
            for y in 0..72 {
                assert_eq!(sample(back, x, y)[3], 0);
                assert_eq!(sample(front, x, y)[3], 0);
            }
        }
        assert_eq!(sample(front, 161, 2)[3], 0, "shoulder is square");
        assert_eq!(sample(front, 177, 20)[3], 255);
        let mean = |x0: usize, x1: usize| {
            let mut sum = 0.0;
            let mut n = 0;
            for y in 58..68 {
                for x in x0..x1 {
                    sum += sample(front, x, y)[0] as f32;
                    n += 1;
                }
            }
            sum / n as f32
        };
        assert!(
            (mean(201, 205) - mean(206, 210)).abs() < 4.0,
            "vertical seam between shoulder and flat deck"
        );
        assert!(
            (mean(515, 519) - mean(510, 514)).abs() < 4.0,
            "vertical seam between right shoulder and flat deck"
        );
        for x in [188, 532] {
            assert!(sample(front, x, 36)[0] < 80, "corner screw well missing");
            assert!(sample(front, x + 2, 45)[0] > 80, "metal screw head missing");
        }
    }
}

#[test]
fn power_markings_keep_subtle_embossed_relief_in_each_finish() {
    for colour in ShellColour::FINISHES {
        let [face, _] = power_switch_faces_for(colour);
        for (x0, x1) in [(465, 495), (515, 536)] {
            let mut lo = 255u8;
            let mut hi = 0u8;
            for y in 245..262 {
                for x in x0..x1 {
                    let value = face.rgba[(y * face.w as usize + x) * 4];
                    lo = lo.min(value);
                    hi = hi.max(value);
                }
            }
            assert!(hi - lo >= 8, "{colour:?}: molded marking disappeared");
            assert!(hi - lo < 60, "{colour:?}: marking became a heavy outline");
        }
    }
}

#[test]
fn slot_end_returns_have_recessed_volume_inside_the_aperture() {
    for colour in ShellColour::FINISHES {
        let [back, front] = shell_slot_faces(colour);
        for x in [213, 507] {
            assert_eq!(sample(back, x, 16)[3], 255);
            assert_eq!(
                sample(front, x, 16)[3],
                0,
                "grip must pass in front of the end wall"
            );
        }
        assert_eq!(sample(front, 233, 16)[3], 0);
        assert_eq!(sample(front, 487, 16)[3], 0);
        let mean = |x0: usize, x1: usize| {
            let values: Vec<_> = (18..26)
                .flat_map(|y| (x0..x1).map(move |x| sample(back, x, y)[0] as f32))
                .collect();
            values.iter().sum::<f32>() / values.len() as f32
        };
        assert!(
            mean(190, 196) > mean(210, 216) + 12.0,
            "left rolled return lost its depth"
        );
        assert!(
            mean(524, 530) > mean(504, 510) + 12.0,
            "right rolled return lost its depth"
        );
    }
}

#[test]
fn central_screw_head_stays_circular_and_centred_at_high_density() {
    let a = shell_slot_faces(ShellColour::Graphite)[0];
    let b = shell_slot_faces(ShellColour::PearlPink)[0];
    let scale = (SLOT_TEXTURE_W / SLOT_W) as usize;
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    // Untinted metal is identical across finishes; painted plastic is not.
    for y in 7 * scale..26 * scale {
        for x in 348 * scale..373 * scale {
            let i = (y * SLOT_TEXTURE_W as usize + x) * 4;
            if a[i + 3] > 250 && a[i..i + 3] == b[i..i + 3] {
                xs.push(x);
                ys.push(y);
            }
        }
    }
    let w = xs.iter().max().unwrap() - xs.iter().min().unwrap() + 1;
    let h = ys.iter().max().unwrap() - ys.iter().min().unwrap() + 1;
    assert!(
        w.abs_diff(h) <= 1,
        "central metal screw is warped: {w} by {h}"
    );
    assert!(
        (10 * scale..=14 * scale).contains(&w),
        "wrong fastener scale: {w}"
    );
    let centre_x =
        (*xs.iter().min().unwrap() + *xs.iter().max().unwrap() + 1) as f32 / (2 * scale) as f32;
    let centre_y =
        (*ys.iter().min().unwrap() + *ys.iter().max().unwrap() + 1) as f32 / (2 * scale) as f32;
    assert!(
        (centre_x - 360.0).abs() <= 0.5 && (centre_y - 16.125).abs() <= 0.5,
        "screw is not centered in the upper plastic strip: {centre_x},{centre_y}"
    );
    let mut dark = Vec::new();
    for y in 12 * scale..21 * scale {
        for x in 356 * scale..365 * scale {
            let px = (x as f32 + 0.5) / scale as f32;
            let py = (y as f32 + 0.5) / scale as f32;
            let dx = px - 360.0;
            let dy = py - 16.125;
            if dx * dx + dy * dy < 13.0 && a[(y * SLOT_TEXTURE_W as usize + x) * 4] < 85 {
                dark.push((px, py));
            }
        }
    }
    assert!(!dark.is_empty(), "driver recess disappeared");
    let cx = dark.iter().map(|p| p.0).sum::<f32>() / dark.len() as f32;
    let cy = dark.iter().map(|p| p.1).sum::<f32>() / dark.len() as f32;
    assert!(
        (cx - 360.0).abs() < 0.5 && (cy - 16.125).abs() < 0.5,
        "driver recess is off-center: {cx},{cy}"
    );
}

#[test]
fn visible_gba_grip_meets_the_aperture_and_gb_bodies_match_its_lower_body() {
    use slot_store::{Cart, Platform};
    use slot_ui::{cart_face, seated_box};
    let [back, _] = shell_slot_faces(ShellColour::Platinum);
    let [other, _] = shell_slot_faces(ShellColour::PearlPink);
    // Untinted liner bounds measure the opening independently of painted relief.
    let metal: Vec<_> = (180..540)
        .filter(|x| sample(back, *x, 34)[..3] == sample(other, *x, 34)[..3])
        .collect();
    let left = *metal.first().unwrap() as f32;
    let right = (*metal.last().unwrap() + 1) as f32;
    let face = |platform| {
        cart_face(&Cart {
            platform,
            stem: "Test".into(),
            title: "Test".into(),
            rom: "test.gba".into(),
            code: String::new(),
            shell: None,
            label: None,
        })
    };
    let gba = face(Platform::Gba);
    let span = |f: &slot_ui::CartFace, y: u32| {
        let xs: Vec<_> = (0..f.w)
            .filter(|x| f.rgba[((y * f.w + x) * 4 + 3) as usize] > 200)
            .collect();
        (
            *xs.first().unwrap() as f32,
            (*xs.last().unwrap() + 1) as f32,
        )
    };
    let scale = seated_box(Platform::Gba).0 as f32 / gba.w as f32;
    let grip = (0..gba.h / 5)
        .map(|y| span(&gba, y))
        .max_by(|a, b| (a.1 - a.0).total_cmp(&(b.1 - b.0)))
        .unwrap();
    let cart_left = 360.0 - gba.w as f32 * scale / 2.0 + grip.0 * scale;
    let cart_right = 360.0 - gba.w as f32 * scale / 2.0 + grip.1 * scale;
    assert!(
        (cart_left - left).abs() <= 1.0 && (cart_right - right).abs() <= 1.0,
        "visible GBA grip {cart_left}..{cart_right} leaves a gap in aperture {left}..{right}"
    );
    let body = span(&gba, gba.h / 2);
    let body_width = (body.1 - body.0) * scale;
    for platform in [Platform::Gb, Platform::Gbc] {
        let f = face(platform);
        let body = span(&f, f.h / 2);
        let width = (body.1 - body.0) * seated_box(platform).0 as f32 / f.w as f32;
        assert!(
            (width - body_width).abs() < 1.0,
            "{platform:?} body {width} versus GBA {body_width}"
        );
    }
}

#[test]
fn shoulder_and_rib_paint_never_crosses_the_metal_boundary() {
    let a = shell_slot_faces(ShellColour::Platinum)[0];
    let b = shell_slot_faces(ShellColour::PearlPink)[0];
    for y in 32..46 {
        for x in 207..513 {
            assert_eq!(
                sample(a, x, y),
                sample(b, x, y),
                "paint spills onto metal at {x},{y}"
            );
        }
    }
}

#[test]
fn photo_slot_opening_occupies_about_seventy_seven_percent_of_the_shell() {
    let [back, front] = shell_slot_faces(ShellColour::Platinum);
    let [other, _] = shell_slot_faces(ShellColour::PearlPink);
    let shell: Vec<_> = (0..SLOT_W as usize)
        .filter(|x| sample(front, *x, 68)[3] > 200)
        .collect();
    let mouth: Vec<_> = (180..540)
        .filter(|x| sample(back, *x, 34)[..3] == sample(other, *x, 34)[..3])
        .collect();
    let shell_width = shell.last().unwrap() - shell.first().unwrap() + 1;
    let mouth_width = mouth.last().unwrap() - mouth.first().unwrap() + 1;
    let ratio = mouth_width as f32 / shell_width as f32;
    assert!(
        (0.76..0.78).contains(&ratio),
        "mouth/shell ratio {ratio}, widths {mouth_width}/{shell_width}"
    );
    assert_eq!(
        mouth_width, 308,
        "do not alter the established cartridge opening"
    );
}

#[test]
fn about_shell_raster_has_headroom_without_changing_logical_dimensions() {
    assert_eq!((SLOT_W, SLOT_H), (720, 72));
    assert_eq!((SLOT_TEXTURE_W, SLOT_TEXTURE_H), (1440, 144));
    assert_eq!((ABOUT_BODY_W, ABOUT_BODY_H), (400, 360));
    assert_eq!((ABOUT_BODY_TEXTURE_W, ABOUT_BODY_TEXTURE_H), (800, 720));
    let original = shell_about_body(ShellColour::Graphite);
    for colour in ShellColour::FINISHES {
        let body = shell_about_body(colour);
        assert_eq!(
            body.len(),
            (ABOUT_BODY_TEXTURE_W * ABOUT_BODY_TEXTURE_H * 4) as usize
        );
        assert!(body
            .chunks_exact(4)
            .zip(original.chunks_exact(4))
            .all(|(a, b)| a[3] == b[3]));
    }
}

#[test]
fn about_body_and_slot_join_without_a_dark_or_bright_seam() {
    for colour in ShellColour::FINISHES {
        let front = shell_slot_faces(colour)[1];
        let body = shell_about_body(colour);
        for c in 0..3 {
            let mut delta = 0.0;
            for x in 230..490 {
                let i = ((x - 160) * (ABOUT_BODY_TEXTURE_W / ABOUT_BODY_W) as usize) * 4;
                assert_eq!(body[i + 3], 255);
                delta += body[i + c] as f32 - sample(front, x, 71)[c] as f32;
            }
            assert!(
                (delta / 260.0).abs() < 2.0,
                "{colour:?}: body/slot paint seam in channel {c}"
            );
        }
    }
}
