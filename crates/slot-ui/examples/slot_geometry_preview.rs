//! Native photo-comparison renders: finishes × cartridge formats × physical poses.
//! cargo run -p slot-ui --example slot_geometry_preview -- OUT [gles|gl] [640|720]
#[cfg(target_os = "linux")]
mod support;

#[cfg(target_os = "linux")]
fn main() {
    use slot_gfx::Compositor;
    use slot_store::{Cart, Platform, ShellColour};
    use slot_ui::*;
    use std::{fs::File, path::PathBuf};
    let dir = PathBuf::from(std::env::args().nth(1).expect("output directory"));
    std::fs::create_dir_all(&dir).unwrap();
    let width = std::env::args().nth(3).and_then(|s| s.parse().ok()).unwrap_or(640);
    let canvas = Canvas { w: width as f32, h: 480.0 };
    let surface = support::Egl::new(std::env::args().nth(2).as_deref() != Some("gl"));
    let mut compositor = Compositor::new(&surface).unwrap();
    compositor.fit((width, 480));
    let label = sticker_face(&StickerFields { battery: Some(82), serial: "B052C00A", version: "review", model: "RG SP" });
    let sticker = compositor.create_texture(label.w, label.h, &label.rgba);
    for colour in ShellColour::FINISHES {
        let [back, front] = shell_slot_faces(colour)
            .map(|p| compositor.create_texture(SLOT_TEXTURE_W, SLOT_TEXTURE_H, p));
        let body = compositor.create_texture(ABOUT_BODY_TEXTURE_W, ABOUT_BODY_TEXTURE_H, shell_about_body(colour));
        for platform in Platform::ALL {
            let cart = Cart { platform, stem: "ADVENTURE".into(), rom: "review.gba".into(),
                label: None, code: String::new(), title: "Adventure".into(),
                shell: (platform == Platform::Gbc).then(|| slot_store::ShellChoice::parse("rounded 7585a4 clear").unwrap()) };
            let face = cart_face(&cart);
            let face = compositor.create_texture(face.w, face.h, &face.rgba);
            let inner = cart_interior(&cart);
            let inner = compositor.create_texture(inner.w, inner.h, &inner.rgba);
            let behind = if shows_interior(&cart) { [Some(inner), None] } else { [None, None] };
            let shelf = Shelf::new(vec![cart.clone()]);
            let (rest, scale) = shelf.selected_at(canvas);
            for (name, seat) in [("empty", None), ("entering", Some(0.55)), ("seated", Some(1.0)), ("about", Some(1.0))] {
                let mut draws = vec![Draw::Rect { x: 0.0, y: 0.0, w: canvas.w, h: canvas.h, colour: [0.83, 0.84, 0.85, 1.0] }];
                let begin = draws.len();
                if let Some(seat) = seat {
                    SlotChrome { cart: &cart, face: Some(face), rest, scale, seat, alert: None,
                        dim: 0.0, screen: 0.0, game: false, over: true, behind,
                        slot: Some((back, front)), canvas }.draw(&mut draws);
                } else { draw_empty_slot(canvas, Some((back, front)), &mut draws); }
                if name == "about" {
                    draw_about_body(canvas, Some(body), &mut draws);
                    draw_about_sticker(canvas, Some(sticker), &mut draws);
                    let mut motion = AboutMotion::default();
                    for _ in 0..180 { motion.update(true, 1.0 / 60.0); }
                    motion.transform(canvas, &mut draws[begin..]);
                    motion.clip_hardware(canvas, &mut draws, begin);
                }
                compositor.begin_frame();compositor.draw_list(&draws);
                let path = dir.join(format!("{}-{}-{name}.png", colour.key(), platform.dir_name().to_lowercase()));
                let mut encoder = png::Encoder::new(File::create(path).unwrap(), width, 480);
                encoder.set_color(png::ColorType::Rgba);encoder.set_depth(png::BitDepth::Eight);
                encoder.write_header().unwrap().write_image_data(&compositor.read_frame()).unwrap();
            }
        }
    }
    println!("Exported 48 native geometry captures at {width}×480");
}

#[cfg(not(target_os = "linux"))]
fn main() { panic!("native geometry reviews require Linux EGL"); }
