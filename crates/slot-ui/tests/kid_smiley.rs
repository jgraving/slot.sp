use slot_store::{Cart, Platform};
use slot_ui::{Draw, Placed, TexId};
fn cart(p: Platform) -> Cart {
    Cart {
        platform: p,
        stem: "Approved".into(),
        title: "Approved".into(),
        rom: "missing".into(),
        label: None,
        code: String::new(),
        shell: None,
    }
}
fn png(path: &std::path::Path, rgba: &[u8], w: u32, h: u32) {
    let f = std::fs::File::create(path).unwrap();
    let mut e = png::Encoder::new(f, w, h);
    e.set_color(png::ColorType::Rgba);
    e.set_depth(png::BitDepth::Eight);
    e.write_header().unwrap().write_image_data(rgba).unwrap();
}
#[test]
fn smiley_is_gold_antialiased_and_anchored_to_the_cart_top_left() {
    let face = slot_ui::kid_smiley_face();
    assert_eq!((face.w, face.h), (30, 30));
    assert!(face.rgba.chunks_exact(4).any(|p| p[3] > 0 && p[3] < 255));
    assert!(
        face.rgba
            .chunks_exact(4)
            .filter(|p| p[3] == 255 && u16::from(p[0]) > u16::from(p[2]) + 20)
            .count()
            > 200
    );
    for platform in Platform::ALL {
        let c = cart(platform);
        let mut out = Vec::new();
        let body = Placed {
            x: 30.,
            y: 40.,
            w: 360.,
            h: 300.,
        };
        slot_ui::draw_kid(&c, (TexId::from_raw(42), 30., 30.), body, 0.6, 0., &mut out);
        let Draw::Tex {
            x, y, w, h, alpha, ..
        } = out[0]
        else {
            panic!("missing smiley");
        };
        assert!(x >= body.x && x < body.x + body.w / 2.);
        assert!(y >= body.y && y < body.y + body.h / 2.);
        assert!(x + w < body.x + body.w);
        assert!(y + h < body.y + body.h);
        assert_eq!(alpha, 0.6);
    }
    if let Ok(dir) = std::env::var("SLOT_KID_UI_PREVIEW") {
        png(
            &std::path::Path::new(&dir).join("kid-smiley.png"),
            &face.rgba,
            face.w,
            face.h,
        );
    }
}
#[test]
fn about_sticker_preview_uses_the_requested_credits() {
    let f = slot_ui::StickerFields {
        battery: Some(87),
        serial: "04738857",
        version: "kid-mode-preview",
        model: "RG SP",
    };
    let face = slot_ui::sticker_face(&f);
    if let Ok(dir) = std::env::var("SLOT_KID_UI_PREVIEW") {
        png(
            &std::path::Path::new(&dir).join("about-sticker.png"),
            &face.rgba,
            face.w,
            face.h,
        );
    }
}

#[test]
fn approval_follows_cart_geometry_and_is_drawn_behind_the_slot_front() {
    use slot_ui::{Canvas, Shelf, SlotChrome};
    let face = TexId::from_raw(101);
    let kid = (TexId::from_raw(102), 30., 30.);
    let front = TexId::from_raw(103);
    for platform in Platform::ALL {
        let c = cart(platform);
        let (rest, scale) = Shelf::new(vec![c.clone()]).selected_at(Canvas::FRAMED);
        for depth in [0., 0.1, 0.4, 0.8, 1., 0.8, 0.4, 0.] {
            let mut out = Vec::new();
            SlotChrome {
                cart: &c, face: Some(face), rest, scale, seat: depth,
                alert: None, dim: 0., screen: 0., game: false, over: true,
                behind: [None, None], slot: Some((TexId::from_raw(104), front)),
                canvas: Canvas::FRAMED,
            }.draw_with_marks(None, Some(kid), &mut out);
            let body = out.iter().find_map(|d| match *d {
                Draw::Tex { x, y, w, h, tex, .. } if tex == face => Some(Placed { x, y, w, h }),
                _ => None,
            }).unwrap();
            let mut expected = Vec::new();
            slot_ui::draw_kid(&c, kid, body, 1., 0., &mut expected);
            let mark = out.iter().position(|d| matches!(d, Draw::Tex { tex, .. } if *tex == kid.0)).unwrap();
            let Draw::Tex { x, y, w, h, alpha, .. } = out[mark] else { panic!() };
            let Draw::Tex { x: ex, y: ey, w: ew, h: eh, alpha: ea, .. } = expected[0] else { panic!() };
            assert_eq!((x, y, w, h, alpha), (ex, ey, ew, eh, ea));
            let lip = out.iter().position(|d| matches!(d, Draw::Part { tex, .. } if *tex == front)).unwrap();
            assert!(mark < lip, "approval must disappear under the slot with the cart");
        }
    }
}

#[test]
fn approval_face_has_readable_dark_eyes_and_smile_at_native_size() {
    let face = slot_ui::kid_smiley_face();
    let dark_pixels = |xs: std::ops::Range<u32>, ys: std::ops::Range<u32>| {
        ys.flat_map(|y| xs.clone().map(move |x| (x, y))).filter(|&(x, y)| {
            let i = ((y * face.w + x) * 4) as usize;
            let p = &face.rgba[i..i + 4];
            p[3] > 240 && p[0] < 55 && p[1] < 40 && p[2] < 30
        }).count()
    };
    assert!(dark_pixels(8..14, 9..15) >= 4, "left eye disappears into the foil");
    assert!(dark_pixels(16..22, 9..15) >= 4, "right eye disappears into the foil");
    assert!(dark_pixels(8..23, 17..24) >= 12, "smile is too fine at native size");
}

#[test]
fn status_smiley_is_white_and_distinct_from_the_foil_sticker() {
    let face=slot_ui::kid_status_face();
    assert_eq!((face.w,face.h),(24,24));
    assert!(face.rgba.chunks_exact(4).any(|p|p[3]>0));
    assert!(face.rgba.chunks_exact(4).filter(|p|p[3]>0).all(|p|p[0].abs_diff(p[1])<4 && p[1].abs_diff(p[2])<8));
}
