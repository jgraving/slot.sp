//! Read cached game definitions and earned state without HTTP or modifying the ledger.
use std::{collections::{BTreeMap, BTreeSet}, path::{Path, PathBuf}};
use crate::storage::{self, Game, Unlock};

#[derive(Clone, Debug)]
pub struct AchievementEntry {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub points: u32,
    pub earned: bool,
    pub badge: Option<PathBuf>,
}
#[derive(Clone, Debug)]
pub struct AchievementList {
    pub title: String,
    pub entries: Vec<AchievementEntry>,
}

/// Call on a background worker. This never starts a session or downloads images.
pub fn cached_achievements(root: &Path, username: &str, rom: &Path) -> Result<AchievementList, &'static str> {
    if username.trim().is_empty() { return Err("Sign in to browse achievements"); }
    if !rom.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("gba")) {
        return Err("Achievement tracking currently supports GBA games");
    }
    let dir = storage::account_dir(root, username);
    let hash = crate::library::content_hash(rom).map_err(|_| "Cannot read this game")?;
    let (game, server): (Game, BTreeSet<u32>) = storage::read(&dir.join(format!("{hash}.json")))
        .map_err(|_| "Prepare this game's achievements online first")?;
    if !crate::tracked_console(game.console) { return Err("No supported achievements for this game"); }
    let ledger = dir.join("unlocks.json");
    let local: BTreeMap<u32, Unlock> = if ledger.exists() {
        storage::read(&ledger).map_err(|_| "Cannot read earned achievements")?
    } else { BTreeMap::new() };
    Ok(AchievementList { title: game.title, entries: game.achievements.into_iter()
        .filter(|a|a.flags == 3).map(|a| AchievementEntry {
            earned: server.contains(&a.id) || local.contains_key(&a.id),
            badge: crate::badges::path(&dir, &a.badge),
            id:a.id, title:a.title, description:a.description, points:a.points,
        }).collect() })
}
