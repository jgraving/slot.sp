use slot_store::{parse_stamp, FF_SPEEDS};
use slot_ui::{
    date_time_text, quick_caret_face, quick_label_face, quick_value_face, ClockPicker, QuickRow,
    QuickValue, UndoFace, MENU_PAD,
};

fn inked(f: &UndoFace, x: u32, y: u32) -> bool {
    f.rgba[((y * f.w + x) * 4 + 3) as usize] > 128
}

fn ink_columns(f: &UndoFace) -> (u32, u32) {
    let cols: Vec<u32> = (0..f.w)
        .filter(|&x| (0..f.h).any(|y| inked(f, x, y)))
        .collect();
    (
        *cols.first().expect("an empty face"),
        *cols.last().expect("an empty face"),
    )
}

#[test]
fn the_type_sits_exactly_menu_pad_in_from_both_sides_of_its_face() {
    for f in [
        quick_label_face(QuickRow::FastForwardSound, true),
        quick_label_face(QuickRow::Rumble, true),
        quick_value_face("Off", false),
        quick_value_face(QuickValue::Speed6.text(), true),
        quick_value_face("Sep 15 16:35", true),
    ] {
        let (first, last) = ink_columns(&f);
        let right = f.w - 1 - MENU_PAD;
        assert!(
            (MENU_PAD..MENU_PAD + 4).contains(&first),
            "ink starts at {first} of {}",
            f.w
        );
        assert!(
            (right - 4..=right).contains(&last),
            "ink ends at {last} of {}",
            f.w
        );
    }
}

#[test]
fn a_long_label_is_set_as_large_as_a_short_one() {
    let tall = |f: &UndoFace| {
        (0..f.h)
            .filter(|&y| (0..f.w).any(|x| inked(f, x, y)))
            .count()
    };
    let long = tall(&quick_label_face(QuickRow::FastForwardSound, true));
    let short = tall(&quick_label_face(QuickRow::Rumble, true));
    assert!(long + 1 >= short, "{long} rows of ink against {short}");
}

#[test]
fn the_fast_forward_row_offers_off_a_slow_speed_and_the_four_ceilings() {
    assert_eq!(FF_SPEEDS, [0, 6, 8, 12, 16, 24]);
    assert_eq!(
        FF_SPEEDS.map(QuickValue::speed),
        [
            Some(QuickValue::Off),
            Some(QuickValue::Speed1_5),
            Some(QuickValue::Speed2),
            Some(QuickValue::Speed3),
            Some(QuickValue::Speed4),
            Some(QuickValue::Speed6),
        ]
    );
    assert_eq!(QuickValue::Speed1_5.text(), "1.5×");
    for other in [1, 2, 3, 4, 5, 7, 9, 28, 255] {
        assert_eq!(
            QuickValue::speed(other),
            None,
            "{other} quarters is not a value the row has"
        );
    }
}

#[test]
fn the_rows_run_in_the_order_the_user_chose() {
    assert_eq!(
        QuickRow::MAIN.map(QuickRow::label),
        ["General", "Screen", "Sound", "Gameplay", "Power & lid", "Device controls", "Parental controls", "About"]
    );
    assert_eq!(
        QuickRow::GENERAL.map(QuickRow::label),
        [
            "Wallpaper",
            "Wallpaper Tint",
            "Shell Color",
            "Theme",
            "Recents",
            "Game Titles",
            "Sort Favorites",
            "Boot Animation",
            "Carousel Sounds",
            "Date & Time"
        ]
    );
    assert_eq!(
        QuickRow::CONTROL_PAGES.map(QuickRow::label),
        ["Shelf", "Shortcuts", "Buttons & Shortcuts", "In-Game"]
    );
    assert_eq!(
        QuickRow::SHELF_KEYS.map(QuickRow::label),
        ["Previous Shelf", "Next Shelf", "Favorite"]
    );
    assert_eq!(
        QuickRow::SCREEN.map(QuickRow::label),
        ["Brightness", "Color temp", "Night mode", "Game Boy Advance", "Game Boy Color", "Game Boy"]
    );
    assert_eq!(
        QuickRow::SCREEN_GENERAL.map(QuickRow::label),
        ["Brightness", "Color temp", "Night mode"]
    );
    assert_eq!(
        QuickRow::SCREEN_GBA.map(QuickRow::label),
        ["Shader", "Color Correction", "Bezel"]
    );
    assert_eq!(
        QuickRow::SCREEN_GBC.map(QuickRow::label),
        ["Shader", "Color Correction", "Bezel"]
    );
    assert_eq!(
        QuickRow::SCREEN_GB.map(QuickRow::label),
        ["Shader", "Color Correction", "Palette", "Bezel"]
    );
    assert_eq!(
        QuickRow::GAME.map(QuickRow::label),
        [
            "Fast Forward",
            "Fast Forward Sound",
            "Rewind",
            "Turbo Buttons",
            "Rumble",
            "Left Stick as D-Pad",
            "Load State on Insert",
            "Save State on Eject"
        ]
    );
    let opens: Vec<QuickRow> = QuickRow::ALL.into_iter().filter(|r| r.opens()).collect();
    assert_eq!(
        opens,
        [
            QuickRow::Screen,
            QuickRow::DateTime,
            QuickRow::About,
            QuickRow::Game,
            QuickRow::Resume,
            QuickRow::States,
            QuickRow::Screenshots,
            QuickRow::Settings,
            QuickRow::ExitGame,
            QuickRow::PlayScreen,
            QuickRow::PlayGameplay,
            QuickRow::ScreenGba,
            QuickRow::ScreenGbc,
            QuickRow::ScreenGb,
            QuickRow::Bluetooth,
            QuickRow::PlaySound,
            QuickRow::Power,
            QuickRow::Kids,
            QuickRow::SleepNow,
            QuickRow::ShutDown,
            QuickRow::General,
            QuickRow::PlayControls,
            QuickRow::Shortcuts,
            QuickRow::Controls,
            QuickRow::ControlsShelf,
            QuickRow::ControlsInGame,
            QuickRow::ScreenGeneral,
            QuickRow::UndoSave,
            QuickRow::UndoLoad,
            QuickRow::Reference,
            QuickRow::PlayReference,
            QuickRow::GameDetails,
            QuickRow::PlayDetails,
            QuickRow::DisplaySchedule
        ]
    );
    for page in [
        &QuickRow::MAIN[..],
        &QuickRow::SCREEN[..],
        &QuickRow::GAME[..],
        &QuickRow::GENERAL[..],
        &QuickRow::PLAY[..],
        &QuickRow::SETTINGS[..],
        &QuickRow::PLAY_SCREEN[..],
        &QuickRow::SCREEN_GBA[..],
        &QuickRow::SCREEN_GBC[..],
        &QuickRow::SCREEN_GB[..],
        &QuickRow::POWER[..],
    ] {
        for &row in page {
            assert_eq!(row.page(), page, "{row:?}");
        }
    }
}

#[test]
fn a_hidden_row_is_stepped_over_and_takes_no_place() {
    let hidden = [QuickRow::WallpaperTint];
    assert_eq!(QuickRow::Wallpaper.down(&hidden), QuickRow::ShellColour);
    assert_eq!(QuickRow::ShellColour.up(&hidden), QuickRow::Wallpaper);
    assert_eq!(QuickRow::Wallpaper.up(&hidden), QuickRow::DateTime);
    assert_eq!(QuickRow::ShellColour.position(&hidden), 1);
    assert_eq!(QuickRow::Wallpaper.down(&[]), QuickRow::WallpaperTint);
    assert_eq!(QuickRow::ShellColour.position(&[]), 2);
    assert_eq!(
        slot_ui::quick_row_top(QuickRow::ShellColour, &hidden)
            - slot_ui::quick_row_top(QuickRow::Wallpaper, &hidden),
        slot_ui::QUICK_PITCH
    );
}

#[test]
fn the_values_read_as_the_menu_prints_them() {
    assert_eq!(
        QuickValue::ALL.map(QuickValue::text),
        [
            "1.5×",
            "2×",
            "3×",
            "4×",
            "6×",
            "On",
            "Off",
            "LCD3x",
            "Grid",
            "Dot",
            "Simpletex",
            "Dark",
            "Light",
            "Show",
            "Hide",
            "25%",
            "45%",
            "65%",
            "Light Sleep",
            "Deep Sleep",
            "Power Off",
            "3 min",
            "15 min",
            "30 min",
            "Never",
            "Match Theme",
            "Graphite",
            "Platinum",
            "Pearl Blue",
            "Pearl Pink",
            "1 min",
            "5 min",
            "10 min",
            "0",
            "1",
            "2",
            "3",
            "4",
            "5",
            "6",
            "7",
            "8",
            "9",
            "Clean",
            "Filename",
            "Alphabetically",
            "Most used",
            "Recently used",
            "Neutral", "Warm 11%", "Warm 22%", "Warm 33%", "Warm 44%",
            "Warm 55%", "Warm 66%", "Warm 77%", "Warm 88%", "Warm 100%",
        ]
    );
    assert_eq!(QuickValue::flag(true), QuickValue::On);
    assert_eq!(QuickValue::flag(false), QuickValue::Off);
}

#[test]
fn the_date_and_time_read_as_a_month_a_day_and_the_carousels_24_hour_clock() {
    let at = |stamp: &str| date_time_text(parse_stamp(stamp).expect("a stamp"));
    assert_eq!(at("2026-09-15_16-35-00"), "Sep 15 16:35");
    assert_eq!(at("2027-01-05_04-07-59"), "Jan 5 04:07");
}

#[test]
fn a_picker_for_a_set_clock_starts_at_the_local_time_and_its_offset() {
    let utc = parse_stamp("2026-09-15_21-35-42").expect("a stamp");
    let p = ClockPicker::local(utc, -300);
    assert_eq!(p.offset_min(), -300);
    assert_eq!(
        p.secs(),
        utc - 42,
        "the picker does not show the minute it was opened in"
    );
    assert!(p.text().starts_with("2026-09-15 16:35"), "{}", p.text());
}

#[test]
fn a_value_is_grey_until_its_row_is_in_hand() {
    let inkiest = |lit| {
        let f = quick_value_face("4×", lit);
        f.rgba
            .chunks(4)
            .max_by_key(|p| p[3])
            .map(|p| [p[0], p[1], p[2]])
            .expect("an empty face")
    };
    assert_eq!(inkiest(true), [0xf6, 0xf4, 0xef]);
    assert_eq!(inkiest(false), [0x9a, 0x9a, 0xa4]);
}

#[test]
fn the_arrows_are_faces_the_height_of_a_value() {
    for right in [false, true] {
        let caret = quick_caret_face(right);
        assert!(
            caret.w > 0 && caret.rgba.chunks(4).any(|p| p[3] > 0),
            "an empty arrow"
        );
        assert_eq!(caret.h, quick_value_face("On", true).h);
    }
}

#[test]
fn the_two_carets_point_different_ways() {
    assert_ne!(
        slot_ui::quick_caret_face(false).rgba,
        slot_ui::quick_caret_face(true).rgba,
        "a caret glyph is missing from the symbols font"
    );
}

#[test]
fn the_pointer_is_a_crisp_pixel_arrow_in_the_menus_ink() {
    let p = slot_ui::quick_pointer_face();
    assert_eq!(p.w, 32);
    let opaque: Vec<&[u8]> = p.rgba.chunks(4).filter(|c| c[3] > 0).collect();
    assert!(!opaque.is_empty(), "an empty pointer");
    assert!(
        opaque.iter().all(|c| c == &[0xf6, 0xf4, 0xef, 0xff]),
        "the pointer is not one solid colour"
    );
    assert!(
        p.rgba.chunks(4).all(|c| c[3] == 0 || c[3] == 0xff),
        "the arrow was smoothed, not drawn in pixels"
    );
}

#[test]
fn a_label_is_grey_until_its_row_is_in_hand() {
    let inkiest = |lit| {
        let f = quick_label_face(QuickRow::Rumble, lit);
        f.rgba
            .chunks(4)
            .max_by_key(|p| p[3])
            .map(|p| [p[0], p[1], p[2]])
            .expect("an empty face")
    };
    assert_eq!(inkiest(true), [0xf6, 0xf4, 0xef]);
    assert_eq!(inkiest(false), [0x9a, 0x9a, 0xa4]);
}

#[test]
fn the_in_game_pages_hold_their_rows_in_order() {
    assert_eq!(
        QuickRow::PLAY,
        [
            QuickRow::Resume,
            QuickRow::UndoSave,
            QuickRow::UndoLoad,
            QuickRow::States,
            QuickRow::Screenshots,
            QuickRow::Settings,
            QuickRow::ExitGame
        ]
    );
    assert_eq!(
        QuickRow::SETTINGS,
        [
            QuickRow::PlayScreen,
            QuickRow::PlaySound,
            QuickRow::PlayGameplay,
            QuickRow::PlayControls,
            QuickRow::PlayReference,
        ]
    );
    assert_eq!(
        QuickRow::PLAY_SCREEN,
        [
            QuickRow::Shader,
            QuickRow::Bezel,
            QuickRow::Palette,
            QuickRow::ColourCorrection,
            QuickRow::PlayBrightness,
            QuickRow::PlayBlueLight
        ]
    );
    for row in QuickRow::PLAY {
        assert_eq!(row.parent_in_game(), None, "{row:?}");
    }
}

#[test]
fn in_game_pages_climb_back_through_settings() {
    assert_eq!(QuickRow::Settings.child(), Some(QuickRow::PlayScreen));
    assert_eq!(QuickRow::PlayScreen.child(), Some(QuickRow::Shader));
    assert_eq!(
        QuickRow::PlayBrightness.parent_in_game(),
        Some(QuickRow::PlayScreen)
    );
    assert_eq!(QuickRow::PlayGameplay.child(), Some(QuickRow::FastForward));
    assert_eq!(
        QuickRow::Shader.parent_in_game(),
        Some(QuickRow::PlayScreen)
    );
    assert_eq!(
        QuickRow::PlayGameplay.parent_in_game(),
        Some(QuickRow::Settings)
    );
    assert_eq!(
        QuickRow::Rumble.parent_in_game(),
        Some(QuickRow::PlayGameplay)
    );
    assert_eq!(QuickRow::Rumble.parent(), Some(QuickRow::Game));
}

#[test]
fn in_game_rows_read_as_the_menu_shows_them() {
    let labels = [
        (QuickRow::Resume, "Resume"),
        (QuickRow::States, "States"),
        (QuickRow::Settings, "Settings"),
        (QuickRow::ExitGame, "Exit Game"),
        (QuickRow::PlayScreen, "Screen"),
        (QuickRow::PlayGameplay, "Gameplay"),
        (QuickRow::Shader, "Shader"),
        (QuickRow::ColourCorrection, "Color Correction"),
        (QuickRow::Palette, "Palette"),
        (QuickRow::PlayBrightness, "Brightness"),
        (QuickRow::PlayBlueLight, "Color temp"),
    ];
    for (row, label) in labels {
        assert_eq!(row.label(), label);
    }
    for row in QuickRow::PLAY.into_iter().chain(QuickRow::SETTINGS) {
        assert!(row.centred(), "{row:?}");
    }
    for row in QuickRow::PLAY_SCREEN {
        assert!(!row.opens(), "{row:?}");
        assert!(!row.centred(), "{row:?}");
    }
    assert!(QuickRow::General.centred());
    assert!(!QuickRow::Rumble.centred());
}

#[test]
fn a_hidden_palette_and_bezel_leave_four_screen_rows() {
    let hidden = [QuickRow::Palette, QuickRow::Bezel];
    assert_eq!(
        QuickRow::Shader.shown(&hidden),
        vec![
            QuickRow::Shader,
            QuickRow::ColourCorrection,
            QuickRow::PlayBrightness,
            QuickRow::PlayBlueLight
        ]
    );
    assert_eq!(QuickRow::PlayBlueLight.down(&hidden), QuickRow::Shader);
}

#[test]
fn the_in_game_screen_page_shows_five_rows_and_scrolls_the_rest() {
    assert_eq!(slot_ui::quick_window(QuickRow::Shader,&[]),(0,5));
    assert_eq!(slot_ui::quick_window(QuickRow::PlayBlueLight,&[]),(1,5));
    assert_eq!(slot_ui::quick_window(QuickRow::PlayBlueLight,&[QuickRow::Palette,QuickRow::Bezel]),(0,4));
}

#[test]
fn a_scroll_bar_shows_only_on_a_page_with_rows_out_of_sight() {
    let top = slot_ui::quick_scroll(QuickRow::Shader, &[]).expect("six rows in three");
    assert!(
        (top.thumb_y - top.y).abs() < 0.5,
        "the thumb is not at the top"
    );
    assert!(
        (top.thumb_h - top.h * 5.0 / 6.0).abs() < 0.5,
        "the thumb does not represent the five visible rows of six"
    );
    let end = slot_ui::quick_scroll(QuickRow::PlayBlueLight, &[]).expect("six rows in three");
    assert!(
        (end.thumb_y + end.thumb_h - (end.y + end.h)).abs() < 0.5,
        "the thumb is not at the bottom"
    );
    assert!(
        slot_ui::quick_scroll(QuickRow::Shader, &[QuickRow::Palette, QuickRow::Bezel]).is_none(),
        "four rows fit without scrolling"
    );
    assert!(slot_ui::quick_scroll(QuickRow::Rumble, &[]).is_some(), "eight gameplay rows scroll within the fork’s six-row limit");
    assert!(slot_ui::quick_scroll(QuickRow::GameUp, &[]).is_some());
}

#[test]
fn the_thumb_reaches_the_bottom_only_on_the_last_row() {
    let rows = QuickRow::CONTROLS;
    let mut last_y = f32::MIN;
    for (i, row) in rows.iter().enumerate() {
        let bar = slot_ui::quick_scroll(*row, &[]).expect("the controls page scrolls");
        let at_bottom = (bar.thumb_y + bar.thumb_h - (bar.y + bar.h)).abs() < 0.5;
        assert_eq!(at_bottom, i == rows.len() - 1, "{row:?} at {i}");
        assert!(bar.thumb_y > last_y, "{row:?}: the thumb did not move down");
        last_y = bar.thumb_y;
    }
}

#[test]
fn screen_opens_a_page_per_console_and_each_backs_out_to_its_console() {
    assert_eq!(QuickRow::Screen.child(), Some(QuickRow::Brightness));
    assert_eq!(QuickRow::ScreenGeneral.child(), Some(QuickRow::Brightness));
    assert!(QuickRow::ScreenGeneral.opens());
    assert!(QuickRow::ScreenGeneral.centred());
    assert!(QuickRow::Brightness.centred());
    assert!(!QuickRow::Brightness.opens());
    for (console, page) in [
        (QuickRow::ScreenGba, &QuickRow::SCREEN_GBA[..]),
        (QuickRow::ScreenGbc, &QuickRow::SCREEN_GBC[..]),
        (QuickRow::ScreenGb, &QuickRow::SCREEN_GB[..]),
    ] {
        assert_eq!(console.parent(), Some(QuickRow::Screen));
        assert_eq!(console.child(), Some(page[0]));
        for &row in page {
            assert_eq!(row.parent(), Some(console), "{row:?}");
        }
    }
    assert!(QuickRow::ScreenGb.centred());
    assert!(!QuickRow::GbShader.centred());
}

#[test]
fn screen_rows_in_game_fill_the_normal_menu_from_the_top() {
    let hidden = [QuickRow::Palette];
    let shader = slot_ui::quick_row_top(QuickRow::Shader, &hidden);
    let bezel = slot_ui::quick_row_top(QuickRow::Bezel, &hidden);
    assert_eq!(shader, 88.0);
    assert_eq!(bezel - shader, slot_ui::QUICK_PITCH);
    assert_eq!(shader, slot_ui::quick_row_top(QuickRow::FastForward, &[]));
    assert!(bezel + 44.0 < 378.0);
}

#[test]
fn the_wallpaper_tint_reads_as_how_much_it_darkens() {
    assert_eq!(
        [QuickValue::Low, QuickValue::Medium, QuickValue::High].map(QuickValue::text),
        ["25%", "45%", "65%"]
    );
}

#[test]
fn show_titles_sits_under_recents_on_the_general_page() {
    let at = |r: QuickRow| QuickRow::GENERAL.iter().position(|g| *g == r);
    assert_eq!(
        at(QuickRow::ShowTitles),
        at(QuickRow::Recents).map(|i| i + 1)
    );
    assert_eq!(QuickRow::ShowTitles.parent(), Some(QuickRow::General));
}

#[test]
fn long_pages_scroll_five_spaced_rows_with_the_selection_and_keep_clear_of_status_and_hints() {
    use slot_ui::*;
    let id = |n| TexId::from_raw(n);
    let faces = QuickMenuFaces {
        headings: vec![],
        labels: QuickRow::ALL
            .iter()
            .map(|r| {
                [
                    (id(r.index() * 2), 260, 40),
                    (id(r.index() * 2 + 1), 260, 40),
                ]
            })
            .collect(),
        pointer: (id(1000), 32, 48),
        values: vec![],
        toggles: [[(id(1005), 116, 40); 2]; 2],
        carets: [(id(1001), 20, 40), (id(1002), 20, 40)],
        legend: [(id(1003), 80); 8],
        legend_light: [(id(1003), 80); 8],
        descriptions: [(id(1004), 200, 23); 2],
    };
    for hidden in [&[][..], &[QuickRow::WallpaperTint][..]] {
        for selected in QuickRow::GENERAL
            .into_iter()
            .chain(QuickRow::GAME)
            .chain(QuickRow::MAIN)
            .chain(QuickRow::CONTROLS)
            .chain(QuickRow::SHORTCUTS)
        {
            if hidden.contains(&selected) {
                continue;
            }
            let shown = selected.shown(hidden);
            let first = quick_window_start(selected, hidden);
            assert!((first..first + QUICK_VISIBLE_ROWS.min(shown.len()))
                .contains(&selected.position(hidden)));
            for width in [640.0, 720.0] {
                let mut out = Vec::new();
                QuickMenu {
                    row: selected,
                    hidden,
                    values: [None; QuickRow::ALL.len()],
                    clock: None,
                    wallpaper: None,
                    faces: Some(&faces),
                    ground: None,
                    palette: None,
                    bound: None,
                    description: selected.binds().then_some((id(1004), 200, 23)),
                    gba_bezel: None,
                    gb_bezel: None,
                    gbc_bezel: None,
                    bezel: None,
                    over_game: false,
                    footer_legends: true,
                    footer_top: 480.,
                    canvas: Canvas { w: width, h: 480. },
                }
                .draw(&mut out);
                let surfaces:Vec<_> = out.iter().filter_map(|d| match d {
                    Draw::RoundRect{x,w,radius,..}=>Some((*x,*w,*radius)),_=>None,
                }).collect();
                assert_eq!(surfaces,vec![(56.0,width-112.0,12.0)],"only selection gets a surface");
                let labels: Vec<_> = out
                    .iter()
                    .filter_map(|d| match d {
                        Draw::Tex { tex, y, h, .. }
                            if shown.iter().any(|r| {
                                [id(r.index() * 2), id(r.index() * 2 + 1)].contains(tex)
                            }) =>
                        {
                            Some((*tex, *y, *h))
                        }
                        _ => None,
                    })
                    .collect();
                assert_eq!(labels.len(), shown.len().min(QUICK_VISIBLE_ROWS));
                assert!(labels.iter().all(
                    |(_, y, h)| *y >= 40. && y + h <= 480. - MOUTH_H - MENU_HINT_H as f32 - 12.
                ));
                assert!(labels
                    .iter()
                    .any(|(tex, ..)| *tex == id(selected.index() * 2 + 1)));
                if selected.binds() {
                    let y = out
                        .iter()
                        .find_map(|d| match *d {
                            Draw::Tex { tex, y, .. } if tex == id(1004) => Some(y),
                            _ => None,
                        })
                        .unwrap();
                    assert!(labels.iter().all(|(_, top, h)| top + h + 8. <= y));
                }
            }
        }
    }
}

#[test]
fn display_settings_use_the_same_percentage_scale() {
    for n in 0..10 {
        assert_eq!(QuickValue::level(n).settings_text(),QuickValue::temperature(n).settings_text());
        assert!(QuickValue::level(n).settings_text().ends_with('%'));
    }
    let chord=slot_ui::quick_binding_face("MENU+L1",true,false);
    let warning=slot_ui::quick_binding_face("MENU+L1",true,true);
    assert!(warning.w>chord.w);
    assert_eq!(chord.h,40);
}

#[test]
fn control_names_are_text_but_device_chords_are_keycaps() {
    let label=slot_ui::quick_label_face(QuickRow::GameA,true);
    let name=slot_ui::quick_value_face("A button",true);
    assert_eq!(label.rgba,name.rgba);
    assert_eq!(slot_ui::quick_binding_face("SEL+X",true,false).rgba,slot_ui::quick_binding_face("SELECT + X",true,false).rgba);
    assert_ne!(label.rgba,slot_ui::quick_binding_face("A",true,false).rgba);
    for value in ["SEL+X","MENU+L1","Up / Down, then A","Tap MENU","Hold MENU","Double-tap MENU","SELECT+MENU","Volume buttons"] {
        let face=slot_ui::reference_binding_face(value,256.0);
        assert!(face.w<=256+2*slot_ui::MENU_PAD,"{value} exceeds its column");
        assert!(face.rgba.chunks_exact(4).any(|p|p[0]==224 && p[1]==227 && p[2]==232 && p[3]==255),"{value} has no native keycap");
    }
}

#[test]
fn carousel_names_and_jump_letters_are_native_larger_type() {
    let name=slot_ui::carousel_name_face("Favorites",false);
    let letter=slot_ui::carousel_name_face("M",true);
    assert_eq!(name.h,36);assert_eq!(letter.h,48);
    assert!(name.h>slot_ui::word_face("Favorites").h);
    let mut out=vec![];
    slot_ui::draw_slot_name_sized(slot_ui::Canvas::FRAMED,slot_ui::Printed::new(slot_ui::TexId::from_raw(44),letter.w),letter.h,1.0,&mut out);
    assert!(matches!(out[0],slot_ui::Draw::Tex{y,h,..}if y+h<480.0-slot_ui::MOUTH_H));
}
