mod common;
use slot::{app::App,frame_blending};
use slot_input::{Action,Btn};
use slot_store::{Platform,Core};
use slot_ui::QuickRow;
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
fn blending_is_opt_in_persisted_per_system_and_uses_declared_core_options() {
    let d = common::tmp_root_with_carts(&["Alpha"]);
    for p in Platform::ALL {
        assert!(!frame_blending::read(d.path(), p));
    }
    let mut a = common::app_playing_in(d.path(), "Alpha");
    screen_row(&mut a, QuickRow::FrameBlending);
    press(&mut a, Btn::Right);
    assert_eq!(a.take_frame_blending(), Some(true));
    assert!(frame_blending::read(d.path(), Platform::Gba));
    assert!(!frame_blending::read(d.path(), Platform::Gb));
    assert_eq!(
        frame_blending::option(Core::Mgba, true),
        ("mgba_interframe_blending", "mix")
    );
    assert_eq!(
        frame_blending::option(Core::Gpsp, true),
        ("gpsp_frame_mixing", "enabled")
    );
    assert_eq!(frame_blending::option(Core::Mgba, false).1, "OFF");
    assert_eq!(frame_blending::option(Core::Gpsp, false).1, "disabled");
}
