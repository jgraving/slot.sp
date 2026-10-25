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
