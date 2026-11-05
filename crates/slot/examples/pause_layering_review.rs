//! Production UI captures and a pixel regression for issue #209.
#[cfg(target_os = "linux")]
#[path = "../../slot-ui/examples/support/mod.rs"]
mod support;

#[cfg(target_os = "linux")]
fn main() {
    use slot::{app::Phase, frontend::Frontend};
    use slot_gfx::{Compositor, Draw};
    use slot_input::{Btn, InputSource, Millis, RawEvent};
    use slot_store::{Appearance, Platform, SlotState};
    use slot_ui::QuickRow;
    struct Input(Vec<RawEvent>);
    impl InputSource for Input {
        fn poll(&mut self, _: Millis) -> Vec<RawEvent> { std::mem::take(&mut self.0) }
    }
    struct Review { f: Frontend, g: Compositor, now: u64, width: u32, dir: std::path::PathBuf }
    impl Review {
        fn frame(&mut self, events: Vec<RawEvent>) {
            self.now += 17;
            self.f.advance_at(&mut Input(events), self.now, 1.0/60.0);
            self.f.compose(&mut self.g);
        }
        fn tap(&mut self, b: Btn) {
            self.frame(vec![RawEvent::Down(b)]);
            self.frame(vec![RawEvent::Up(b)]);
        }
        fn settle(&mut self) { for _ in 0..90 { self.frame(vec![]); } }
        fn save(&mut self, name: &str, platform: Platform) {
            self.settle();
            let mut draws = vec![];
            self.f.app().draw(&mut draws);
            // Use the exact same draw list twice: the only difference is the
            // seated cartridge (including its interior). A correctly opaque
            // card must have identical pixels with or without that cartridge.
            let (cw, ch) = slot_ui::cart_box(platform);
            let sw = slot_ui::seated_box(platform).0 as f32;
            let sh = ch as f32 * sw / cw as f32;
            let is_cart = |d: &Draw| matches!(d, Draw::Tex { w, h, .. }
                if (*w-sw).abs()<0.1 && (*h-sh).abs()<0.1);
            assert!(draws.iter().any(is_cart), "no real {platform:?} cartridge in the regression fixture");
            self.g.draw_list(&draws);
            let actual = self.g.read_frame();
            let without: Vec<_> = draws.iter().copied().filter(|d| !is_cart(d)).collect();
            self.g.draw_list(&without);
            let expected = self.g.read_frame();
            let canvas = self.f.app().canvas();
            let rect = if name == "settings" { slot_ui::expressive::card_rect(canvas) }
                else { slot_ui::menu_grid::pause_rect(canvas, QuickRow::Resume.shown(&self.f.app().in_game_hidden()).len()) };
            let [x, y, w, h] = rect;
            let mut compared = 0;
            for yy in (y+32.0) as usize..(y+h-32.0) as usize {
                let start = (yy*self.width as usize+(x+32.0) as usize)*4;
                let end = (yy*self.width as usize+(x+w-32.0) as usize)*4;
                assert_eq!(&actual[start..end], &expected[start..end], "{platform:?} {name}: cartridge covers card at y={yy}");
                compared += end-start;
            }
            assert!(compared > 1000, "empty card comparison");
            let file = std::fs::File::create(self.dir.join(format!("{name}.png"))).unwrap();
            let mut encoder = png::Encoder::new(file, self.width, 480);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder.write_header().unwrap().write_image_data(&actual).unwrap();
            self.f.compose(&mut self.g);
        }
    }
    let dir = std::path::PathBuf::from(std::env::args().nth(1).expect("output directory"));
    let es = std::env::args().nth(2).as_deref() != Some("gl");
    let width = std::env::args().nth(3).unwrap_or_else(|| "640".into()).parse::<u32>().unwrap();
    let light = std::env::args().nth(4).as_deref() == Some("light");
    let platform = match std::env::args().nth(5).as_deref() { Some("gb") => Platform::Gb, Some("gbc") => Platform::Gbc, _ => Platform::Gba };
    std::fs::create_dir_all(&dir).unwrap();
    let root = dir.join("card");
    slot::root::ensure(&root);
    slot_store::write_slot_state(&root, &SlotState {
        clock_set: true, appearance: if light { Appearance::Light } else { Appearance::Dark }, ..Default::default()
    }).unwrap();
    let mut rom = vec![0; 0x150];
    if platform == Platform::Gbc { rom[0x143] = 0xc0; }
    std::fs::write(root.join("Games").join(platform.dir_name()).join(format!("Aurora.{}", platform.extensions()[0])), rom).unwrap();
    let surface = support::Egl::new(es);
    let mut g = Compositor::new(&surface).unwrap();
    g.fit((width, 480));
    let mut f = Frontend::boot(Box::new(slot_power::SimPlatform::at(root).stopped_at(1_793_774_400)));
    f.upload_faces(&mut g);
    f.render(&mut g, (width, 480));
    let mut a = Review { f, g, now: 0, width, dir };
    a.settle(); a.tap(Btn::A);
    for _ in 0..300 { a.frame(vec![]); if matches!(a.f.app().phase(), Phase::Playing { .. }) { break; } }
    assert!(matches!(a.f.app().phase(), Phase::Playing { .. }));
    a.settle(); a.tap(Btn::Menu); a.settle();
    assert_eq!(a.f.app().in_game_menu(), Some(QuickRow::Resume));
    a.save("resume", platform);
    a.tap(Btn::Down);
    assert_eq!(a.f.app().in_game_menu(), Some(QuickRow::Settings));
    a.save("settings-focus", platform);
    a.tap(Btn::A);
    assert_eq!(a.f.app().in_game_menu(), Some(QuickRow::General));
    a.save("settings", platform);
    a.tap(Btn::B); a.settle();
    assert_eq!(a.f.app().in_game_menu(), Some(QuickRow::Settings));
    a.tap(Btn::Down);
    assert_eq!(a.f.app().in_game_menu(), Some(QuickRow::Exit));
    a.save("exit", platform);
    a.tap(Btn::B); a.settle();
    assert_eq!(a.f.app().in_game_menu(), None);
    assert!(matches!(a.f.app().phase(), Phase::Playing { .. }));
}
#[cfg(not(target_os = "linux"))]
fn main() {}
