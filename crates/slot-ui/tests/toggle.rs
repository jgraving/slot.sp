use slot_ui::{toggle_face, QuickRow, MENU_PAD};

#[test]
fn switches_use_colour_and_thumb_position_over_a_transparent_background() {
    for lit in [false, true] {
        let off = toggle_face(false, lit);
        let on = toggle_face(true, lit);
        assert_eq!((on.w, on.h), (80 + 2 * MENU_PAD, 40));
        let pixel = |face: &slot_ui::UndoFace, x| {
            let i = ((20 * face.w + MENU_PAD + x) * 4) as usize;
            <[u8; 4]>::try_from(&face.rgba[i..i+4]).unwrap()
        };
        assert!(pixel(&off, 62)[0] > pixel(&off, 62)[1]);
        assert!(pixel(&on, 18)[1] > pixel(&on, 18)[0]);
        assert!(pixel(&off, 18)[0] > 240);
        assert!(pixel(&on, 62)[0] > 240);
        assert_eq!(on.rgba[3], 0);
        assert_eq!(off.rgba[3], 0);
        assert!(on.rgba.chunks_exact(4).any(|p| p[3] > 0 && p[3] < 200));
    }
}

#[test]
fn binary_settings_use_switches_while_multi_value_choices_keep_their_values() {
    for row in [QuickRow::AutoLoad, QuickRow::EjectSave, QuickRow::BootAnimation,
        QuickRow::CarouselSounds, QuickRow::Rumble, QuickRow::Recents, QuickRow::ShowTitles,
        QuickRow::FastForwardSound, QuickRow::ColourCorrection, QuickRow::Rewind,
        QuickRow::Turbo, QuickRow::LeftStick] { assert!(row.toggle(), "{row:?}"); }
    for row in [QuickRow::FastForward, QuickRow::BlueLight, QuickRow::GbaShader,
        QuickRow::WallpaperTint, QuickRow::ShellColour, QuickRow::SleepTimeout,
        QuickRow::Brightness] { assert!(!row.toggle(), "{row:?}"); }
}
