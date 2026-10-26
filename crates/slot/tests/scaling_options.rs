mod common;
use slot::{app::{App,Phase}, video_mode::{self,VideoMode}};
use slot_input::{Action,Btn};
use slot_store::{Platform,Core};
use slot_ui::{QuickRow,QuickValue};
fn press(a: &mut App, b: Btn) {
    a.apply(Action::GbaDown(b));
    a.apply(Action::GbaUp(b));
}
fn screen_row(a: &mut App, want: QuickRow) {
    a.apply(Action::QuickMenu);
    for target in [QuickRow::Settings, QuickRow::PlayScreen, want] {
        for _ in 0..16 {
            if a.in_game_menu() == Some(target) {
                break;
            }
            press(a, Btn::Down);
        }
        assert_eq!(a.in_game_menu(), Some(target));
        if target != want {
            press(a, Btn::A);
        }
    }
}
#[test]
fn fit_preserves_aspect_and_integer_uses_whole_physical_pixels() {
    for panel in [
        (720, 480),
        (640, 480),
        (480, 320),
        (1280, 720),
        (1920, 1080),
    ] {
        for platform in Platform::ALL {
            let (sw, sh) = platform.picture();
            for mode in [VideoMode::Fit, VideoMode::Integer] {
                let [x, y, w, h] = mode.destination(platform, panel).unwrap();
                let (x, y, w, h) = (
                    x / 720.0 * panel.0 as f32,
                    y / 480.0 * panel.1 as f32,
                    w / 720.0 * panel.0 as f32,
                    h / 480.0 * panel.1 as f32,
                );
                assert!((w / h - sw as f32 / sh as f32).abs() < 0.0001);
                assert!(
                    x >= 0.0
                        && y >= 0.0
                        && x + w <= panel.0 as f32 + 0.001
                        && y + h <= panel.1 as f32 + 0.001
                );
                assert!(((panel.0 as f32 - w) / 2.0 - x).abs() < 1.001);
                if mode == VideoMode::Integer {
                    assert!((w / sw as f32 - (w / sw as f32).round()).abs() < 0.001);
                    assert!((h / sh as f32 - (h / sh as f32).round()).abs() < 0.001);
                }
            }
        }
    }
    assert_eq!(
        VideoMode::Integer.destination(Platform::Gba, (640, 480)),
        Some([90.0, 80.0, 540.0, 320.0])
    );
}
#[test]
fn modes_round_trip_and_crop_only_the_game_pixels() {
    let d = common::tmp_root_with_gb_carts(&["Tetris"]);
    for mode in [
        VideoMode::Actual,
        VideoMode::Stretch,
        VideoMode::Fit,
        VideoMode::Integer,
    ] {
        video_mode::write_video_mode(d.path(), "Tetris", mode).unwrap();
        assert_eq!(video_mode::video_mode_for(d.path(), "Tetris"), mode);
    }
    let crop = video_mode::source_rect(Platform::Gb, VideoMode::Fit);
    assert_eq!(
        crop,
        [40.0 / 240.0, 8.0 / 160.0, 160.0 / 240.0, 144.0 / 160.0]
    );
    assert_eq!(VideoMode::Actual.step(true).step(false), VideoMode::Actual);
}
#[test]
fn scaling_menu_saves_per_game_and_keeps_the_selected_rectangle_while_paused() {
    let d = common::tmp_root_with_carts(&["Alpha", "Beta"]);
    let mut a = common::app_playing_in(d.path(), "Alpha");
    a.set_panel((640, 480));
    screen_row(&mut a, QuickRow::Scaling);
    press(&mut a, Btn::Right);
    press(&mut a, Btn::Right);
    assert_eq!(a.quick_value(QuickRow::Scaling), Some(QuickValue::Fit));
    assert_eq!(
        video_mode::video_mode_for(d.path(), "Alpha"),
        VideoMode::Fit
    );
    assert_eq!(
        video_mode::video_mode_for(d.path(), "Beta"),
        VideoMode::Actual
    );
    let rect = a.game_destination();
    press(&mut a, Btn::B);
    assert_eq!(a.game_destination(), rect);
    assert!(matches!(a.phase(), Phase::Playing { .. }));
}
