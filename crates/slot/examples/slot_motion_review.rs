//! Review actual App insertion/ejection timing through the production renderer.
//! Gameplay is deliberately hidden so the connector stop remains visible.
//! cargo run -p slot --no-default-features --example slot_motion_review -- OUT [gba|gb|gbc] [gles|gl] [640|720]
#[cfg(target_os = "linux")]
#[path = "../../slot-ui/examples/support/mod.rs"]
mod support;

#[cfg(target_os = "linux")]
fn main() {
    use slot::{app::{App, Phase}, audio::Sfx};
    use slot_gfx::Compositor;
    use slot_input::Action;
    use slot_store::{Platform, ShellColour};
    use slot_ui::{Canvas, Draw, Shelf, SlotChrome};
    use std::{fs::File, io::Write, path::PathBuf};

    const FPS: usize = 60;
    const FRAMES: usize = 300;
    const HZ: usize = 48_000;
    let dir = PathBuf::from(std::env::args().nth(1).expect("output directory"));
    std::fs::create_dir_all(&dir).unwrap();
    let platform = match std::env::args().nth(2).as_deref() {
        Some("gb") => Platform::Gb,
        Some("gbc") => Platform::Gbc,
        _ => Platform::Gba,
    };
    let width = std::env::args().nth(4).and_then(|s| s.parse().ok()).unwrap_or(640);
    assert!([640, 720].contains(&width));
    // A second cart is required for the normal user-facing Eject action.
    let root = dir.join("card");
    slot::root::ensure(&root);
    let state = slot_store::SlotState { clock_set: true, ..Default::default() };
    slot_store::write_slot_state(&root, &state).unwrap();
    let (folder, extension) = match platform {
        Platform::Gba => ("GBA", "gba"),
        Platform::Gb => ("GB", "gb"),
        Platform::Gbc => ("GBC", "gbc"),
    };
    for title in ["ADVENTURE", "CLASSICS"] {
        let mut rom = vec![0; 0x150];
        let offset = if platform == Platform::Gba { 0xa0 } else { 0x134 };
        rom[offset..offset + title.len()].copy_from_slice(title.as_bytes());
        if platform == Platform::Gbc { rom[0x143] = 0xc0; }
        std::fs::write(root.join(format!("Games/{folder}/{title}.{extension}")), rom).unwrap();
    }
    let mut app = App::boot(&root);
    app.set_panel((width, 480));
    app.update(1.0);
    assert!(matches!(app.phase(), Phase::Shelf));
    let selected = app.selected_stem().unwrap().to_owned();
    let cart = app.carts().find(|c| c.stem == selected).unwrap().clone();
    assert_eq!(cart.platform, platform);
    let shelf = Shelf::new(vec![cart.clone()]);
    let canvas = Canvas { w: width as f32, h: 480.0 };
    let (rest, scale) = shelf.selected_at(canvas);
    let surface = support::Egl::new(std::env::args().nth(3).as_deref() != Some("gl"));
    let mut compositor = Compositor::new(&surface).unwrap();
    compositor.fit((width, 480));
    let face = slot_ui::cart_face(&cart);
    let face = compositor.create_texture(face.w, face.h, &face.rgba);
    let inside = slot_ui::cart_interior(&cart);
    let inside = compositor.create_texture(inside.w, inside.h, &inside.rgba);
    let behind = if slot_ui::shows_interior(&cart) { [Some(inside), None] } else { [None, None] };
    let [back, front] = slot_ui::shell_slot_faces(ShellColour::Platinum)
        .map(|pixels| compositor.create_texture(slot_ui::SLOT_TEXTURE_W, slot_ui::SLOT_TEXTURE_H, pixels));
    let background = compositor.create_texture(720, 480, slot_ui::default_background().unwrap());
    let clock = slot_ui::status_word_face("12:34", slot_ui::footer_ink(false));
    let clock_tex = compositor.create_texture(clock.w, clock.h, &clock.rgba);
    let battery = slot_power::Battery { percent: 82, charge: slot_power::Charge::Discharging };
    let percent = slot_ui::status_battery_face(battery);
    let percent_tex = compositor.create_texture(percent.w, percent.h, &percent.rgba);
    let mut audio = vec![0i16; FRAMES * HZ / FPS * 2];
    let mut trace = File::create(dir.join("motion.csv")).unwrap();
    writeln!(trace, "frame,seconds,phase,seat,sfx").unwrap();
    let (mut insert_count, mut eject_count, mut max_seat) = (0, 0, 0.0f32);
    for frame in 0..FRAMES {
        if frame == 30 {
            app.apply(Action::Insert);
            app.on_core_ready();
            app.set_game_ready(true);
        }
        if frame == 150 {
            assert!(matches!(app.phase(), Phase::Playing { .. }));
            app.apply(Action::Eject);
            assert!(matches!(app.phase(), Phase::Ejecting { .. }));
        }
        app.tick_ms(((frame + 1) * 1000 / FPS) as u64);
        app.update(1.0 / FPS as f32);
        let sound = app.take_sfx();
        if let Some(sound) = sound {
            match sound {
                Sfx::Insert => insert_count += 1,
                Sfx::Eject => eject_count += 1,
                other => panic!("unexpected sound in stationary fixture: {other:?}"),
            }
            let offset = frame * HZ / FPS * 2;
            for (dst, sample) in audio[offset..].iter_mut().zip(sound.render(HZ as u32)) {
                *dst = dst.saturating_add(sample);
            }
        }
        max_seat = max_seat.max(app.seat());
        let phase = match app.phase() {
            Phase::Shelf => "shelf",
            Phase::Inserting { .. } => "inserting",
            Phase::Playing { .. } => "seated",
            Phase::Ejecting { .. } => "ejecting",
            other => panic!("unexpected phase {other:?}"),
        };
        writeln!(trace, "{frame},{:.6},{phase},{:.8},{:?}", frame as f64 / FPS as f64, app.seat(), sound).unwrap();
        let mut draws = Vec::<Draw>::new();
        slot_ui::draw_ground(canvas, Some((background, 720)), &mut draws);
        // Only presentation flags differ: the App continues its real picture
        // hold and release timing while the review leaves the cartridge visible.
        SlotChrome {
            cart: &cart, face: Some(face), rest, scale, seat: app.seat(),
            alert: None, dim: 0.0, screen: 0.0, game: false, over: true,
            behind, slot: Some((back, front)), canvas,
        }.draw(&mut draws);
        slot_ui::draw_status(canvas, Some(battery), slot_ui::Printed::new(percent_tex, percent.w),
            None, slot_ui::Printed::new(clock_tex, clock.w), slot_ui::footer_ink(false), &mut draws);
        compositor.begin_frame();
        compositor.draw_list(&draws);
        let mut encoder = png::Encoder::new(File::create(dir.join(format!("motion-{frame:04}.png"))).unwrap(), width, 480);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.write_header().unwrap().write_image_data(&compositor.read_frame()).unwrap();
    }
    assert_eq!((insert_count, eject_count), (1, 1));
    assert_eq!(max_seat, 1.0, "cartridge never reached connector stop");
    assert!(matches!(app.phase(), Phase::Shelf));
    assert_eq!(app.seat(), 0.0);
    std::fs::write(dir.join("audio.s16le"), audio.into_iter().flat_map(i16::to_le_bytes).collect::<Vec<_>>()).unwrap();
    println!("Validated {FRAMES} frames: {platform:?}, one insert, one eject, connector stop and return to shelf");
}

#[cfg(not(target_os = "linux"))]
fn main() { panic!("native motion reviews require Linux EGL"); }
