//! Walk the on-device editor through the production frontend in both themes.
#[cfg(target_os = "linux")]
#[path = "../../slot-ui/examples/support/mod.rs"]
mod support;
#[cfg(target_os = "linux")]
fn main() {
    use slot::{app::Phase, frontend::Frontend};
    use slot_gfx::Compositor;
    use slot_input::{Btn, InputSource, Millis, RawEvent};
    use slot_ui::QuickRow;
    struct Input(Vec<RawEvent>);
    impl InputSource for Input {
        fn poll(&mut self, _: Millis) -> Vec<RawEvent> {
            std::mem::take(&mut self.0)
        }
    }
    struct Review {
        f: Frontend,
        g: Compositor,
        now: u64,
        dir: std::path::PathBuf,
        width: u32,
    }
    impl Review {
        fn frame(&mut self, events: Vec<RawEvent>) {
            self.now += 17;
            self.f.advance_at(&mut Input(events), self.now, 1.0 / 60.0);
            self.f.compose(&mut self.g);
        }
        fn tap(&mut self, b: Btn) {
            self.frame(vec![RawEvent::Down(b)]);
            self.frame(vec![RawEvent::Up(b)]);
        }
        fn settle(&mut self) {
            for _ in 0..45 {
                self.frame(vec![]);
            }
        }
        fn save(&mut self, name: &str) {
            let until = std::time::Instant::now() + std::time::Duration::from_secs(10);
            while self
                .f
                .app()
                .studio
                .as_ref()
                .is_some_and(|m| !m.preview_ready())
            {
                assert!(std::time::Instant::now() < until);
                std::thread::sleep(std::time::Duration::from_millis(5));
                self.frame(vec![]);
            }
            self.settle();
            let mut e = png::Encoder::new(
                std::fs::File::create(self.dir.join(format!("{name}.png"))).unwrap(),
                self.width,
                480,
            );
            e.set_color(png::ColorType::Rgba);
            e.set_depth(png::BitDepth::Eight);
            e.write_header()
                .unwrap()
                .write_image_data(&self.g.read_frame())
                .unwrap();
        }
        fn nav(&mut self, to: QuickRow) {
            for _ in 0..32 {
                let at = self.f.app().quick_menu().unwrap();
                if at == to {
                    return;
                }
                assert_eq!(at.page(), to.page());
                let b = if at.page() == QuickRow::MAIN {
                    let a = at.position(&[]);
                    let b = to.position(&[]);
                    if a % 4 < b % 4 {
                        Btn::Right
                    } else if a % 4 > b % 4 {
                        Btn::Left
                    } else {
                        Btn::Down
                    }
                } else {
                    Btn::Down
                };
                self.tap(b);
            }
            panic!("unreachable row");
        }
    }
    let dir = std::path::PathBuf::from(std::env::args().nth(1).unwrap());
    std::fs::create_dir_all(&dir).unwrap();
    let width = std::env::args()
        .nth(3)
        .and_then(|s| s.parse().ok())
        .unwrap_or(640);
    let root = dir.join("card");
    if root.exists() {
        std::fs::remove_dir_all(&root).unwrap();
    }
    slot::root::ensure(&root);
    let state = slot_store::SlotState {
        clock_set: true,
        appearance: if std::env::args().nth(4).as_deref() == Some("light") {
            slot_store::Appearance::Light
        } else {
            slot_store::Appearance::Dark
        },
        ..Default::default()
    };
    slot_store::write_slot_state(&root, &state).unwrap();
    for name in ["Aurora", "Borealis"] {
        std::fs::write(
            root.join("Games/GBA").join(format!("{name}.gba")),
            vec![0; 256],
        )
        .unwrap();
    }
    let surface = support::Egl::new(std::env::args().nth(2).as_deref() != Some("gl"));
    let mut g = Compositor::new(&surface).unwrap();
    g.fit((width, 480));
    let mut f = Frontend::boot(Box::new(slot_power::SimPlatform::at(root.clone())));
    f.upload_faces(&mut g);
    f.render(&mut g, (width, 480));
    let mut a = Review {
        f,
        g,
        now: 0,
        dir,
        width,
    };
    a.settle();
    assert!(matches!(a.f.app().phase(), Phase::Shelf));
    a.tap(Btn::Menu);
    a.settle();
    a.nav(QuickRow::Game);
    a.tap(Btn::A);
    a.nav(QuickRow::CartStudio);
    a.save("gameplay-studio-row");
    a.tap(Btn::A);
    assert!(a.f.app().studio.is_some());
    a.save("studio-offline");
    a.tap(Btn::A);
    a.save("studio-label-choice");
    a.tap(Btn::A);
    a.save("studio-title-applied");
    a.tap(Btn::B);
    for _ in 0..3 {a.tap(Btn::Down);}
    a.tap(Btn::Right);a.save("studio-clear-shell");
    a.tap(Btn::Right);a.save("studio-glitter-shell");
    for _ in 0..2 {a.tap(Btn::Up);}
    a.tap(Btn::A);
    a.save("studio-label-palette");
    a.tap(Btn::Right);
    a.tap(Btn::Right);
    a.tap(Btn::A);
    a.save("studio-custom-rgb");
    a.tap(Btn::Right);
    a.tap(Btn::Y);
    a.save("studio-custom-hex");
    a.tap(Btn::B);
    a.tap(Btn::B);
    a.tap(Btn::B);
    a.tap(Btn::B);
    let until = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while a.f.app().studio.is_some() {
        assert!(std::time::Instant::now() < until);
        std::thread::sleep(std::time::Duration::from_millis(5));
        a.frame(vec![]);
    }
    a.save("studio-saved");
    assert!(root.join("Labels/GBA/Aurora.png").is_file());
}
#[cfg(not(target_os = "linux"))]
fn main() {}
