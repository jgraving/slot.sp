//! Review actual control faces and animated quads through the production compositor.
#[cfg(target_os = "linux")]
#[path = "../../slot-ui/examples/support/mod.rs"]
mod support;
#[cfg(target_os = "linux")]
fn main() {
    use slot_gfx::Compositor;
    use slot_ui::*;
    let path = std::path::PathBuf::from(std::env::args().nth(1).expect("output directory"));
    std::fs::create_dir_all(&path).unwrap();
    let width = std::env::args()
        .nth(3)
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(640);
    let canvas = Canvas {
        w: width as f32,
        h: 480.0,
    };
    let surface = support::Egl::new(std::env::args().nth(2).as_deref() != Some("gl"));
    let mut gpu = Compositor::new(&surface).unwrap();
    gpu.fit((width, 480));
    let ground = gpu.create_texture(720, 480, default_background().unwrap());
    let slot = shell_slot_faces(slot_store::ShellColour::FINISHES[0])
        .map(|f| gpu.create_texture(SLOT_W, SLOT_H, &f));
    let mut save =
        |name: &str, face: UndoFace, mut overlays: Vec<Draw>, focus: Option<[f32; 4]>| {
            let tex = gpu.create_texture(face.w, face.h, &face.rgba);
            let mut draws = vec![];
            draw_menu_ground(canvas, Some((ground, 720)), &mut draws);
            if let Some(r) = focus {
                expressive::rounded(
                    &mut draws,
                    r[0],
                    r[1],
                    r[2],
                    r[3],
                    12.0,
                    expressive::SELECTED,
                );
            }
            draws.push(Draw::Tex {
                x: 0.0,
                y: 0.0,
                w: canvas.w,
                h: 480.0,
                tex,
                alpha: 1.0,
            });
            draws.append(&mut overlays);
            draw_empty_slot(canvas, Some((slot[0], slot[1])), &mut draws);
            gpu.begin_frame();
            gpu.draw_list(&draws);
            let mut encoder = png::Encoder::new(
                std::fs::File::create(path.join(format!("{name}.png"))).unwrap(),
                width,
                480,
            );
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .unwrap()
                .write_image_data(&gpu.read_frame())
                .unwrap();
        };
    let rows = [
        ("System", "GBA"),
        ("Playtime", "123 h 59 min"),
        ("Last played", "2026-10-30"),
        ("At", "23:59"),
    ]
    .map(|(a, b)| (a.into(), b.into(), false));
    save(
        "details",
        reference_face(canvas, "Game details", &rows, 0, 1),
        vec![],
        None,
    );
    let mut schedule = slot::display_schedule::Menu::new(slot::display_settings::DisplaySettings {
        schedule: true,
        bedtime: (2, 9),
        ..Default::default()
    });
    schedule.input(slot_input::Action::GbaDown(slot_input::Btn::Up), true);
    let mut switches = expressive::SwitchMotion::default();
    switches.update(&schedule.visible_switches(), 1.0);
    let mut sliders = expressive::SliderMotion::default();
    sliders.update(&schedule.visible_sliders(), 1.0);
    let mut controls = vec![];
    let frame = MenuFrame::settled(QuickRow::DisplaySchedule, &[]);
    draw_menu_switches(
        canvas,
        &schedule.visible_switches(),
        &switches,
        frame,
        &mut controls,
    );
    draw_menu_sliders(
        canvas,
        &schedule.visible_sliders(),
        &sliders,
        frame,
        &mut controls,
    );
    save(
        "schedule",
        schedule.screen_face_animated_on(canvas),
        controls,
        None,
    );
    save("parent-code", slot_ui::button_code_face(&[0,1,2,3]), vec![], None);
    drop(save);
    let icons = Icon::ALL
        .into_iter()
        .map(|icon| {
            let f = icon_face(icon, HUD_ICON_PX, [245, 242, 239]);
            gpu.create_texture(f.w, f.h, &f.rgba)
        })
        .collect();
    let mut hud = Hud::new();
    hud.set_icons(icons);
    hud.show(HudKind::Volume, 65, false, 0);
    for ms in [0, 50, 100, 300, 1300, 1400, 1490] {
        let mut out = vec![];
        draw_menu_ground(canvas, Some((ground, 720)), &mut out);
        hud.draw(canvas, ms, &mut out);
        gpu.begin_frame();
        gpu.draw_list(&out);
        write(
            &path.join(format!("slider-{ms}.png")),
            width,
            &gpu.read_frame(),
        );
    }
    let toasts = Toast::all().into_iter().map(|t| {
        let f=toast_face(t); gpu.create_texture(f.w,f.h,&f.rgba)
    }).collect();
    hud.hush(); hud.set_toasts(toasts); hud.toast(Toast::DeepSleepWake,0);
    let mut notice=vec![]; draw_menu_ground(canvas,Some((ground,720)),&mut notice);
    hud.draw(canvas,200,&mut notice); gpu.begin_frame(); gpu.draw_list(&notice);
    write(&path.join("wake-notice.png"),width,&gpu.read_frame());
    let picker = ClockPicker::local(slot_store::parse_stamp("2026-10-30_12-00-00").unwrap(), 60);
    let f = picker.face_on(canvas);
    let line = gpu.create_texture(f.w, f.h, &f.rgba);
    let f = set_clock_hint_face();
    let hint = gpu.create_texture(f.w, f.h, &f.rgba);
    let f = prompt_legends([246, 244, 239], "Back", "Save")
        .into_iter()
        .next()
        .unwrap();
    let back = (gpu.create_texture(f.w, f.h, &f.rgba), f.w);
    let mut out = vec![];
    picker.draw(canvas, Some(line), Some(hint), Some(back), &mut out);
    draw_empty_slot(canvas, Some((slot[0], slot[1])), &mut out);
    gpu.begin_frame();
    gpu.draw_list(&out);
    write(&path.join("clock.png"), width, &gpu.read_frame());
}
#[cfg(target_os = "linux")]
fn write(path: &std::path::Path, width: u32, pixels: &[u8]) {
    let mut e = png::Encoder::new(std::fs::File::create(path).unwrap(), width, 480);
    e.set_color(png::ColorType::Rgba);
    e.set_depth(png::BitDepth::Eight);
    e.write_header().unwrap().write_image_data(pixels).unwrap();
}
#[cfg(not(target_os = "linux"))]
fn main() {}
