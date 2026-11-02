use slot_store::{Cart, FavoriteSort, Platform, PlayHistory};

fn cart(platform: Platform, name: &str) -> Cart {
    Cart { platform, stem: name.into(), rom: name.into(), label: None, title: String::new(), code: String::new(), shell: None }
}
#[test]
fn history_round_trips_unicode_delimiters_and_same_names_on_different_systems() {
    let dir = tempfile::tempdir().unwrap(); let mut h = PlayHistory::default();
    let name = "Pokémon\t雪\n(USA)";
    h.started(Platform::Gba, name, Some(1_800_000_000)); h.add_time(Platform::Gba, name, 12345);
    h.started(Platform::Gb, name, None); h.save(dir.path()).unwrap();
    let loaded = PlayHistory::read(dir.path()).unwrap();
    assert_eq!(loaded.get(Platform::Gba, name), h.get(Platform::Gba, name));
    assert_eq!(loaded.get(Platform::Gb, name).milliseconds, 0);
    assert_eq!(loaded.get(Platform::Gb, name).last_utc, None);
}
#[test]
fn recently_used_keeps_all_favorites_even_after_many_other_games_are_played() {
    let mut h = PlayHistory::default(); let mut favorites: Vec<_> = (0..35).map(|n| cart(Platform::Gba, &format!("Game {n:02}"))).collect();
    for c in &favorites { h.started(c.platform, &c.stem, None); }
    for n in 0..100 { h.started(Platform::Gb, &format!("Other {n}"), None); }
    h.sort(&mut favorites, FavoriteSort::RecentlyUsed);
    assert_eq!(favorites.len(), 35); assert_eq!(favorites[0].stem, "Game 34"); assert_eq!(favorites[34].stem, "Game 00");
}
#[test]
fn most_used_is_playtime_and_ties_are_alphabetical() {
    let mut h = PlayHistory::default(); let mut carts = vec![cart(Platform::Gb,"Zelda"),cart(Platform::Gba,"Alpha"),cart(Platform::Gba,"Beta")];
    h.add_time(Platform::Gb,"Zelda", 1000); h.add_time(Platform::Gba,"Beta", 1000);
    h.started(Platform::Gba,"Alpha", Some(1_900_000_000));
    h.sort(&mut carts,FavoriteSort::MostUsed);
    assert_eq!(carts.iter().map(|c|c.stem.as_str()).collect::<Vec<_>>(),["Beta","Zelda","Alpha"]);
    h.sort(&mut carts,FavoriteSort::Alphabetical); assert_eq!(carts[0].stem,"Alpha");
}
#[test]
fn recency_uses_session_order_even_if_the_clock_moves_backward() {
    let mut h = PlayHistory::default();h.started(Platform::Gb,"A",Some(1_900_000_000));h.started(Platform::Gb,"B",Some(1_800_000_000));
    let mut carts=vec![cart(Platform::Gb,"A"),cart(Platform::Gb,"B")];h.sort(&mut carts,FavoriteSort::RecentlyUsed);assert_eq!(carts[0].stem,"B");
}
#[test]
fn malformed_history_is_reported_without_altering_the_file() {
    let d=tempfile::tempdir().unwrap();std::fs::create_dir(d.path().join("Config")).unwrap();
    let path=d.path().join("Config/play-history.txt");let text="slot-play-history-v1\nGB\tzz\t2\t0\t1\n";std::fs::write(&path,text).unwrap();
    assert!(PlayHistory::read(d.path()).is_err());assert_eq!(std::fs::read_to_string(path).unwrap(),text);
}
