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
    for target in [QuickRow::Settings, QuickRow::Screen, want] {
        for _ in 0..16 {
            let current = a.in_game_menu().unwrap();
            if current == target { break; }
            if slot_ui::menu_grid::is_page(current.page()) {
                let hidden = a.in_game_hidden();
                let cols = slot_ui::menu_grid::columns(current, &hidden);
                let now = current.position(&hidden);
                let to = target.position(&hidden);
                press(a, if now % cols < to % cols { Btn::Right }
                    else if now % cols > to % cols { Btn::Left } else { Btn::Down });
            } else { press(a, Btn::Down); }
        }
        assert_eq!(a.in_game_menu(), Some(target));
        if target != want { press(a, Btn::A); }
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
