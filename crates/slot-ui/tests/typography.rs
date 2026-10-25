use slot_ui::text::{self, Role};

#[test]
fn each_profile_measures_and_renders_its_own_font() {
    let mut pictures = Vec::new();
    for role in Role::ALL {
        let font = text::font(role).expect("bundled Google Sans Flex profile");
        let layout = text::fit(font, "KID MODE 50%", 600.0, 1, 30.0, 30.0);
        assert_eq!(layout.role, role);
        let pixels = text::coverage(640, 48, &layout);
        assert!(pixels.iter().filter(|a| **a > 127).count() > 100);
        pictures.push(pixels);
    }
    for i in 0..pictures.len() {
        for j in i + 1..pictures.len() {
            assert_ne!(pictures[i], pictures[j], "profiles {i} and {j} lost their variations");
        }
    }
}

#[test]
fn changing_numeric_values_keeps_the_same_advance_width() {
    let font = text::font(Role::Value).unwrap();
    for (a, b) in [("11:11", "88:88"), ("11%", "88%"), ("1 H 11 MIN", "8 H 88 MIN")] {
        assert!((text::line_width(font, a, 30.0, 1.0)
            - text::line_width(font, b, 30.0, 1.0)).abs() < 0.001);
    }
}

#[test]
fn fallback_keeps_greek_and_cyrillic_game_names_visible() {
    for role in Role::ALL {
        let font = text::font(role).unwrap();
        let layout = text::fit(font, "Ω Ж", 300.0, 1, 30.0, 30.0);
        let pixels = text::coverage(320, 48, &layout);
        let missing = text::fit(font, "\u{10ffff} \u{10ffff}", 300.0, 1, 30.0, 30.0);
        assert_ne!(pixels, text::coverage(320, 48, &missing));
        assert!(pixels.iter().filter(|p| **p > 127).count() > 100);
    }
}

#[test]
fn every_style_fits_long_labels_inside_the_available_box() {
    for role in Role::ALL {
        let font = text::font(role).unwrap();
        let layout = text::fit_box(font, "A VERY LONG SETTING OR DEVICE NAME", 280.0, 64.0, 2, 30.0, 12.0);
        assert!(layout.lines.len() <= 2);
        for line in &layout.lines {
            assert!(text::line_width(font, line, layout.px, layout.tracking) <= 280.01);
        }
    }
}
