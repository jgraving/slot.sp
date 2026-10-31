mod common;
use slot::app::{App, Phase};
use slot_input::{Action, Btn};
use slot_store::{FavoriteSort, Platform, PlayHistory, Recent, SlotState, read_slot_state, write_slot_state};
use slot_ui::{QuickRow, QuickValue};

fn press(a: &mut App, b: Btn) { a.apply(Action::GbaDown(b)); a.apply(Action::GbaUp(b)); }
fn open_at(a: &mut App, row: QuickRow) {
    a.apply(Action::QuickMenu);
    let mut path = vec![row];
    while let Some(parent) = path.last().and_then(|r| r.parent()) { path.push(parent); }
    for (depth, step) in path.iter().rev().enumerate() {
        if depth > 0 { press(a, Btn::A); }
        for _ in 0..step.position(&a.hidden_rows()) { press(a, Btn::Down); }
    }
    assert_eq!(a.quick_menu(), Some(row));
}
fn saved_time(a: &mut App, root: &std::path::Path) -> u64 {
    a.settle_saves(); PlayHistory::read(root).unwrap().get(Platform::Gba, "Alpha").milliseconds
}
#[test]
fn playtime_counts_real_active_time_and_excludes_menus_loading_and_sleep() {
    let d = common::tmp_root_with_carts(&["Alpha", "Beta"]);
    let mut a = common::app_playing_in(d.path(), "Alpha");
    a.update(5.0); assert_eq!(saved_time(&mut a,d.path()),0);
    a.set_game_ready(true); a.update(2.0);
    assert_eq!(saved_time(&mut a,d.path()),2000);
    a.apply(Action::QuickMenu); a.update(45.0);
    assert_eq!(saved_time(&mut a,d.path()),2000);
    a.apply(Action::QuickMenu); a.update(3.0);
    assert_eq!(saved_time(&mut a,d.path()),5000);
    a.apply(Action::LidClose); a.update(60.0);
    assert!(matches!(a.phase(),Phase::Doze{..}));
    assert_eq!(saved_time(&mut a,d.path()),5000);
    a.apply(Action::LidOpen); a.update(1.0);
    assert_eq!(saved_time(&mut a,d.path()),6000);
    a.apply(Action::Eject); a.update(5.0);
    assert_eq!(saved_time(&mut a,d.path()),6000);
    assert!(PlayHistory::read(d.path()).unwrap().get(Platform::Gba,"Alpha").last_utc.is_some());
}
#[test]
fn sorting_setting_persists_and_never_trims_favorites_or_changes_recents() {
    let names: Vec<_>=(0..35).map(|n|format!("Game {n:02}")).collect();
    let refs: Vec<_>=names.iter().map(String::as_str).collect();
    let d=common::tmp_root_with_carts(&refs);
    let favorites: Vec<_>=names.iter().map(|stem|Recent{platform:Platform::Gba,stem:stem.clone()}).collect();
    write_slot_state(d.path(),&SlotState{clock_set:true,favorites:favorites.clone(),..Default::default()}).unwrap();
    let mut h=PlayHistory::default();h.add_time(Platform::Gba,"Game 34",90000);h.started(Platform::Gba,"Game 33",Some(1_800_000_000));h.save(d.path()).unwrap();
    let mut a=App::boot(d.path());
    for _ in 0..6 { if a.shelf_platform_name()==Some("Favorites"){break;} a.apply(Action::NextShelf); }
    assert_eq!(a.shelf_platform_name(),Some("Favorites"));
    open_at(&mut a,QuickRow::FavoriteSort);
    assert_eq!(a.quick_value(QuickRow::FavoriteSort),Some(QuickValue::Alphabetical));
    press(&mut a,Btn::Right);assert_eq!(read_slot_state(d.path()).favorite_sort,FavoriteSort::MostUsed);
    press(&mut a,Btn::A);assert_eq!(read_slot_state(d.path()).favorite_sort,FavoriteSort::RecentlyUsed);
    while a.quick_menu().is_some() { a.apply(Action::QuickMenu); }
    assert_eq!(a.selected_stem(),Some("Game 00"), "changing sort keeps the selected favorite");
    a.apply(Action::ShelfLeft); a.apply(Action::GbaUp(Btn::Left)); a.update(1.0);
    assert_eq!(a.selected_stem(),Some("Game 33"));
    let saved=read_slot_state(d.path());assert_eq!(saved.favorites,favorites);assert!(saved.recents.is_empty());
}
#[test]
fn details_entries_are_absent_from_both_settings_menus() {
    assert!(!QuickRow::MAIN.contains(&QuickRow::GameDetails));
    assert!(!QuickRow::SETTINGS.contains(&QuickRow::PlayDetails));
}

#[test]
fn unreadable_history_is_never_overwritten_by_playing() {
    let d=common::tmp_root_with_carts(&["Alpha","Beta"]);let path=d.path().join("Config/play-history.txt");
    std::fs::write(&path,b"keep this corrupt original").unwrap();
    let mut a=common::app_playing_in(d.path(),"Alpha");a.set_game_ready(true);a.update(61.0);a.settle_saves();
    assert_eq!(std::fs::read(path).unwrap(),b"keep this corrupt original");
}
