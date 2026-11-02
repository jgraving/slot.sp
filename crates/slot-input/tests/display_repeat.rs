use slot_input::{
    Action, Btn, Chord, Context, Contexts, Entry, Gestures, Mod, RawEvent, VOLUME_REPEAT_DELAY_MS,
    VOLUME_REPEAT_MS,
};

fn entry(modifier: Option<Mod>, button: Btn, action: Action) -> Entry {
    Entry {
        chord: Chord { modifier, button },
        action,
        contexts: Contexts::ALL,
    }
}

#[test]
fn all_four_display_adjustments_use_volume_timing_without_bursts() {
    for (button, action) in [
        (Btn::Up, Action::BrightnessUp),
        (Btn::Down, Action::BrightnessDown),
        (Btn::Left, Action::BlueLightDown),
        (Btn::Right, Action::BlueLightUp),
    ] {
        let mut g = Gestures::new();
        g.set_entries(vec![entry(Some(Mod::Select), button, action)]);
        assert!(g.feed(RawEvent::Down(Btn::Select), 0).is_empty());
        assert_eq!(g.feed(RawEvent::Down(button), 10), vec![action]);
        let due = 10 + VOLUME_REPEAT_DELAY_MS;
        assert!(g.tick(due - 1).is_empty());
        assert_eq!(g.tick(due), vec![action]);
        assert!(g.feed(RawEvent::Down(button), due + 1).is_empty());
        assert!(g.tick(due + VOLUME_REPEAT_MS - 1).is_empty());
        assert_eq!(g.tick(due + VOLUME_REPEAT_MS), vec![action]);
        assert_eq!(
            g.tick(10_000),
            vec![action],
            "late ticks must not jump many levels"
        );
        assert!(g.tick(10_001).is_empty());
        assert!(g.feed(RawEvent::Up(button), 10_002).is_empty());
        assert!(g.feed(RawEvent::Up(Btn::Select), 10_003).is_empty());
        assert!(g.tick(20_000).is_empty());
    }
}

#[test]
fn releasing_either_button_stops_default_and_remapped_chords() {
    let action = Action::BrightnessUp;
    for (modifier, button, first, second) in [
        (Some(Mod::Select), Btn::Up, Btn::Select, Btn::Up),
        (Some(Mod::Menu), Btn::Up, Btn::Menu, Btn::Up),
        (Some(Mod::Button(Btn::Start)), Btn::Up, Btn::Start, Btn::Up),
        (Some(Mod::Button(Btn::L1)), Btn::R1, Btn::L1, Btn::R1),
        (Some(Mod::Button(Btn::L1)), Btn::R1, Btn::R1, Btn::L1),
    ] {
        for release_first in [false, true] {
            let mut g = Gestures::new();
            g.set_entries(vec![entry(modifier, button, action)]);
            g.feed(RawEvent::Down(first), 0);
            assert_eq!(g.feed(RawEvent::Down(second), 10), vec![action]);
            assert_eq!(g.tick(10 + VOLUME_REPEAT_DELAY_MS), vec![action]);
            let (released, remaining) = if release_first {
                (first, second)
            } else {
                (second, first)
            };
            g.feed(RawEvent::Up(released), 500);
            assert!(
                g.tick(2_000).is_empty(),
                "{modifier:?}: releasing {released:?} kept repeating"
            );
            g.feed(RawEvent::Up(remaining), 2_001);
            g.feed(RawEvent::Down(first), 3_000);
            assert_eq!(g.feed(RawEvent::Down(second), 3_010), vec![action]);
            assert!(g.tick(3_010 + VOLUME_REPEAT_DELAY_MS - 1).is_empty());
            assert_eq!(g.tick(3_010 + VOLUME_REPEAT_DELAY_MS), vec![action]);
        }
    }
}

#[test]
fn a_plain_display_binding_repeats_without_repeating_its_shared_shelf_action() {
    let mut g = Gestures::new();
    g.set_entries(vec![
        entry(None, Btn::R1, Action::BrightnessUp),
        entry(None, Btn::R1, Action::NextShelf),
    ]);
    g.set_context(Context::Shelf);
    let down = g.feed(RawEvent::Down(Btn::R1), 0);
    assert!(down.contains(&Action::BrightnessUp) && down.contains(&Action::NextShelf));
    assert_eq!(g.tick(VOLUME_REPEAT_DELAY_MS), vec![Action::BrightnessUp]);
    g.feed(RawEvent::Up(Btn::R1), 500);
    assert!(g.tick(2_000).is_empty());
}

#[test]
fn context_binding_changes_and_captured_input_cancel_an_active_ramp() {
    for change in 0..4 {
        let mut g = Gestures::new();
        g.set_entries(vec![entry(
            Some(Mod::Select),
            Btn::Up,
            Action::BrightnessUp,
        )]);
        g.feed(RawEvent::Down(Btn::Select), 0);
        g.feed(RawEvent::Down(Btn::Up), 10);
        match change {
            0 => g.set_context(Context::Menu),
            1 => g.set_entries(Vec::new()),
            2 => g.observe(RawEvent::Up(Btn::Select)),
            _ => g.cancel_display_ramps(),
        }
        assert!(g.tick(1_000).is_empty());
        g.set_context(Context::Game);
        assert!(
            g.tick(2_000).is_empty(),
            "returning to the game resumed an old ramp"
        );
    }
}

#[test]
fn a_modifier_released_in_a_recorder_cannot_start_a_phantom_repeat() {
    let mut g = Gestures::new();
    g.set_entries(vec![entry(Some(Mod::Select), Btn::Up, Action::BrightnessUp)]);
    g.feed(RawEvent::Down(Btn::Select), 0);
    g.observe(RawEvent::Up(Btn::Select));
    g.feed(RawEvent::Down(Btn::Up), 10);
    assert!(g.tick(10 + VOLUME_REPEAT_DELAY_MS).is_empty());
}

#[test]
fn holds_leave_one_shot_shortcuts_as_one_shot_actions() {
    for action in [
        Action::NightToggle,
        Action::SaveState,
        Action::LoadState,
        Action::Screenshot,
        Action::ColourCorrectionToggle,
    ] {
        let mut g = Gestures::new();
        g.set_entries(vec![entry(Some(Mod::Select), Btn::X, action)]);
        g.feed(RawEvent::Down(Btn::Select), 0);
        assert_eq!(g.feed(RawEvent::Down(Btn::X), 10), vec![action]);
        assert!(g.tick(2_000).is_empty(), "{action:?} repeated");
    }
}
