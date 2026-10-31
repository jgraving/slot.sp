use slot_store::{
    Binding, CarouselAction, CarouselKeys, Controls, GameButton, Platform, Shortcut, ShortcutKeys,
};
pub type Page = (String, Vec<(String, String, bool)>);
fn rows(pairs: &[(&str, &str)]) -> Vec<(String, String, bool)> {
    pairs
        .iter()
        .map(|(a, b)| (a.to_string(), b.to_string(), false))
        .collect()
}
pub fn pages(
    kid: bool,
    in_game: bool,
    platform: Platform,
    controls: Controls,
    shortcuts: ShortcutKeys,
    carousel: CarouselKeys,
) -> Vec<Page> {
    let mut pages = vec![(
        "Choose a game".into(),
        rows(&[
            ("Choose game", "Left / Right"),
            ("Skip a letter", "Up / Down"),
            ("Play", "A"),
            ("Menu", "MENU"),
        ]),
    )];
    for action in [CarouselAction::PreviousShelf, CarouselAction::NextShelf] {
        pages[0].1.push((
            action.label().into(),
            Binding::text(carousel.get(action)),
            false,
        ));
    }
    if kid {
        pages.push((
            "Kid menu".into(),
            rows(&[
                ("Open menu", "MENU"),
                ("Select", "Up / Down, then A"),
                ("Volume / brightness", "Left / Right"),
                ("Return to game", "B"),
                ("Leave a game", "Exit game in menu"),
                ("Parent settings", "Parental controls"),
            ]),
        ));
    } else {
        pages.push((
            "While playing".into(),
            rows(&[
                ("Game menu", "Tap MENU"),
                ("Exit game", "Hold MENU"),
                ("Saved states", "Double-tap MENU"),
                ("Link play", "SELECT+MENU"),
                ("GB / GBC scaling", "L1 / R1"),
                ("Volume / mute", "Volume buttons"),
            ]),
        ));
        pages.push((
            "On the shelf".into(),
            vec![
                (
                    "Favorites".into(),
                    Binding::text(carousel.get(CarouselAction::Favorite)),
                    false,
                ),
                ("Approve for kids".into(), "X".into(), false),
                ("Choose GBA emulator".into(), "START".into(), false),
                ("Start without resume".into(), "Hold A".into(), false),
            ],
        ));
        let mapped: Vec<_> = Shortcut::ALL
            .iter()
            .map(|s| (s.label().into(), Binding::text(shortcuts.get(*s)), false))
            .collect();
        for (i, chunk) in mapped.chunks(6).enumerate() {
            pages.push((format!("Shortcuts {}", i + 1), chunk.to_vec()));
        }
    }
    if in_game {
        let mapped: Vec<_> = GameButton::of(platform)
            .iter()
            .filter(|g| !kid || !matches!(g, GameButton::TurboA | GameButton::TurboB))
            .map(|g| (g.label().into(), Binding::text(controls.game(*g)), false))
            .collect();
        for (i, chunk) in mapped.chunks(6).enumerate() {
            pages.push((format!("Game buttons {}", i + 1), chunk.to_vec()));
        }
    }
    pages
}
