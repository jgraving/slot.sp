//! Render the actual menu draw list through the production GLES/GL compositor.
//! cargo run -p slot-ui --example expressive_menu_preview -- /tmp/previews [gl]
#[cfg(target_os = "linux")]
mod support;
#[cfg(target_os = "linux")]
fn main() {
    use slot_gfx::Compositor;
    use slot_ui::*;
    let path = std::path::PathBuf::from(std::env::args().nth(1).expect("output directory"));
    std::fs::create_dir_all(&path).unwrap();
    let surface = support::Egl::new(std::env::args().nth(2).as_deref() != Some("gl"));
    let mut compositor = Compositor::new(&surface).unwrap();
    let width = std::env::args()
        .nth(3)
        .and_then(|n| n.parse::<u32>().ok())
        .unwrap_or(720);
    let canvas = Canvas {
        w: width as f32,
        h: 480.0,
    };
    compositor.fit((width, 480));
    // Exercise the production shader's corners, interior, and uniform reset.
    compositor.begin_frame();
    compositor.draw_list(&[
        Draw::Rect {
            x: 0.,
            y: 0.,
            w: 720.,
            h: 480.,
            colour: [0., 0., 0., 1.],
        },
        Draw::RoundRect {
            x: 10.,
            y: 10.,
            w: 40.,
            h: 40.,
            radius: 20.,
            colour: [1.; 4],
        },
        Draw::Rect {
            x: 60.,
            y: 10.,
            w: 20.,
            h: 20.,
            colour: [1., 0., 0., 1.],
        },
    ]);
    let pixels = compositor.read_frame();
    let (rw, _) = compositor.frame_size();
    let pixel =
        |x: u32, y: u32| &pixels[((y * rw + x) * 4) as usize..((y * rw + x) * 4 + 3) as usize];
    assert!(pixel(10, 10).iter().all(|c| *c < 10), "rounded corner");
    assert!(pixel(30, 30).iter().all(|c| *c > 245), "rounded interior");
    assert!(
        pixel(60, 10)[0] > 245 && pixel(60, 10)[1] < 10,
        "rounding uniform leaked into next sprite"
    );
    let mut up = |f: UndoFace| {
        let tex = compositor.create_texture(f.w, f.h, &f.rgba);
        (tex, f.w, f.h)
    };
    let faces = QuickMenuFaces {
        headings: QuickRow::ALL
            .into_iter()
            .filter(|r| r.page()[0] == *r)
            .map(|r| (r, up(quick_heading_face(r.heading()))))
            .collect(),
        labels: QuickRow::ALL
            .into_iter()
            .map(|r| [false, true].map(|lit| up(quick_label_face(r, lit))))
            .collect(),
        pointer: up(quick_pointer_face()),
        values: QuickValue::ALL
            .into_iter()
            .map(|v| {
                [false, true].map(|lit| up(quick_value_face(&menu_sentence_case(v.text()), lit)))
            })
            .collect(),
        toggles: [false, true].map(|on| [false, true].map(|lit| up(toggle_face(on, lit)))),
        carets: [false, true].map(|r| up(quick_caret_face(r))),
        legend: quick_legend_faces(footer_ink(false)).map(|f| {
            let (t, w, _) = up(f);
            (t, w)
        }),
        legend_light: quick_legend_faces(footer_ink(true)).map(|f| {
            let (t, w, _) = up(f);
            (t, w)
        }),
        descriptions: [
            up(quick_description_face(&[
                "Supports single button and MENU+button.",
            ])),
            up(quick_description_face(&[
                "Supports single button and two button chords.",
            ])),
        ],
    };
    let ink = footer_ink(false);
    let (ct, cw, _) = up(word_ink("12:34", ink));
    let (pt, pw, _) = up(word_ink("82%", ink));
    let as_undo = |f: CartFace| UndoFace {
        rgba: f.rgba,
        w: f.w,
        h: f.h,
    };
    let (bt, _, _) = up(as_undo(icon_face(Icon::Bluetooth, 20.0, ink)));
    let (mt, _, _) = up(as_undo(icon_face(Icon::VolumeMuted, 20.0, ink)));
    let (bolt, _, _) = up(as_undo(icon_face(Icon::Charging, 18.0, ink)));
    let ground = compositor.create_texture(720, 480, default_background().unwrap());
    let parts = shell_slot_faces(slot_store::ShellColour::FINISHES[0]);
    let slot = parts.map(|f| compositor.create_texture(SLOT_W, SLOT_H, &f));
    let mut motion = MenuMotion::default();
    let mut draws = Vec::with_capacity(96);
    let mut peak = 0;
    let mut cpu = std::time::Duration::ZERO;
    let scenarios = [
        ("settings", QuickRow::Screen),
        ("general", QuickRow::BootAnimation),
        ("gameplay", QuickRow::FastForwardSound),
        ("back", QuickRow::Game),
    ];
    for (scene, (name, row)) in scenarios.into_iter().enumerate() {
        for frame in 0..60 {
            let row = if name == "settings" && (frame < 18 || frame >= 38) {
                QuickRow::General
            } else {
                row
            };
            motion.update(Some(row), &[], Some(frame < 32), 1.0 / 60.0);
            let mut values = [None; QuickRow::ALL.len()];
            for r in QuickRow::ALL {
                if r.toggle() {
                    values[r.index()] = Some(QuickValue::On);
                }
            }
            values[QuickRow::FastForwardSound.index()] = Some(QuickValue::flag(frame < 32));
            values[QuickRow::BootAnimation.index()] = Some(QuickValue::flag(frame < 32));
            values[QuickRow::FastForward.index()] = Some(QuickValue::Speed2);
            values[QuickRow::WallpaperTint.index()] = Some(QuickValue::Off);
            values[QuickRow::ShowTitles.index()] = Some(QuickValue::Clean);
            values[QuickRow::FavoriteSort.index()] = Some(QuickValue::Alphabetical);
            values[QuickRow::ShellColour.index()] = Some(QuickValue::Graphite);
            values[QuickRow::Theme.index()] = Some(QuickValue::Dark);
            draws.clear();
            let start = std::time::Instant::now();
            if motion.outgoing_row().is_some() {
                draw_menu_ground(canvas, Some((ground, 720)), &mut draws);
            }
            for outgoing in [true, false] {
                let current = row;
                let Some(row) = (if outgoing {
                    motion.outgoing_row()
                } else {
                    Some(row)
                }) else {
                    continue;
                };
                QuickMenu {
                    row,
                    hidden: &[],
                    values,
                    clock: None,
                    wallpaper: None,
                    faces: Some(&faces),
                    ground: Some((ground, 720)),
                    palette: None,
                    gba_bezel: None,
                    gb_bezel: None,
                    gbc_bezel: None,
                    bezel: None,
                    bound: None,
                    description: None,
                    over_game: false,
                    footer_legends: true,
                    footer_top: 480.,
                    canvas,
                }
                .draw_animated(
                    if outgoing {
                        motion.outgoing_frame(row, &[])
                    } else {
                        motion.frame(current, &[])
                    },
                    &mut draws,
                );
            }
            draw_empty_slot(canvas, Some((slot[0], slot[1])), &mut draws);
            for (right, (tex, w)) in [
                (false, faces.legend[0]),
                (true, faces.legend[if row.opens() { 2 } else { 1 }]),
            ] {
                draws.push(Draw::Tex {
                    x: if right {
                        canvas.w - 16.0 - w as f32 + HINT_EDGE as f32
                    } else {
                        16.0
                    },
                    y: 378.,
                    w: w as f32,
                    h: 32.,
                    tex,
                    alpha: 1.0,
                });
            }
            let battery = Some(slot_power::Battery {
                percent: 82,
                charge: slot_power::Charge::Charging,
            });
            let percent = Printed::new(pt, pw);
            let clock = Printed::new(ct, cw);
            draw_status(canvas, battery, percent, Some(bolt), clock, ink, &mut draws);
            draw_corner_audio(battery, percent, Some(bt), Some(mt), &mut draws);
            cpu += start.elapsed();
            peak = peak.max(draws.len());
            compositor.begin_frame();
            compositor.draw_list(&draws);
            {
                let rgba = compositor.read_frame();
                let (w, h) = compositor.frame_size();
                let target = format!("motion-{:03}.png", scene * 60 + frame);
                let mut enc =
                    png::Encoder::new(std::fs::File::create(path.join(target)).unwrap(), w, h);
                enc.set_color(png::ColorType::Rgba);
                enc.set_depth(png::BitDepth::Eight);
                enc.write_header().unwrap().write_image_data(&rgba).unwrap();
                if frame == 59 {
                    std::fs::copy(
                        path.join(format!("motion-{:03}.png", scene * 60 + frame)),
                        path.join(format!("{name}.png")),
                    )
                    .unwrap();
                }
            }
        }
    }
    println!(
        "Menu/status draw construction: {:.1} us average over 240 frames; peak {peak} quads",
        cpu.as_secs_f64() * 1e6 / 240.0
    );
}
#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("This review exporter uses surfaceless EGL on Linux.");
}
