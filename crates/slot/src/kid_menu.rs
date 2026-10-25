use crate::{
    kid_policy::{Policy, Rule},
    parent_code::Gate,
};
use slot_input::{Action, Btn};
use slot_ui::{TexId, UndoFace};
use std::{path::PathBuf, time::Instant};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Home,
    Child,
    Games,
    Unlock,
    Set,
    Confirm,
    Notice,
    Warning,
}
pub struct Menu {
    root: PathBuf,
    gate: Gate,
    pub draft: Policy,
    page: Page,
    selected: usize,
    code: Vec<u8>,
    first: Vec<u8>,
    games: Vec<slot_store::Cart>,
    game_at: usize,
    game_removal: Option<(usize, Instant, bool)>,
    start_after_code: bool,
    status: String,
    pub visible: bool,
    pub face: Option<TexId>,
    pub changed: Option<Policy>,
    pub child_action: Option<Action>,
    pub child_volume: u8,
    pub child_brightness: u8,
    child_in_game: bool,
    remaining: Option<(u64, u64)>,
}
impl Menu {
    pub fn new(root: PathBuf, policy: Policy) -> Self {
        let gate = Gate::new(Policy::directory(&root).join("code"));
        let page = if gate.enabled() || policy.active {
            Page::Unlock
        } else {
            Page::Home
        };
        Self {
            root,
            gate,
            draft: policy,
            page,
            selected: 0,
            code: Vec::new(),
            first: Vec::new(),
            games: Vec::new(),
            game_at: 0,
            game_removal: None,
            start_after_code: false,
            status: if page == Page::Unlock {
                "Recovery: Config/kids/parent-code.txt".into()
            } else {
                String::new()
            },
            visible: true,
            face: None,
            changed: None,
            child_action: None,
            child_volume: 0,
            child_brightness: 0,
            child_in_game: false,
            remaining: None,
        }
    }
    pub fn child(root: PathBuf, policy: Policy, in_game: bool, volume: u8, brightness: u8) -> Self {
        let mut menu = Self::new(root, policy);
        menu.page = Page::Child;
        menu.status.clear();
        menu.child_in_game = in_game;
        menu.child_volume = volume;
        menu.child_brightness = brightness;
        menu
    }
    pub fn time_warning(&self) -> bool { self.page == Page::Warning }
    pub fn set_remaining(&mut self, remaining: Option<(u64, u64)>) {
        if self.remaining != remaining { self.remaining = remaining; self.face = None; }
    }
    pub fn remaining_seconds(&self) -> Option<u64> { self.remaining.map(|r| r.0) }
    pub fn warning(root: PathBuf, policy: Policy, seconds: u64) -> Self {
        let budget = policy.time_budget();
        let mut menu = Self::new(root, policy);
        menu.page = Page::Warning;
        menu.status.clear();
        menu.remaining = Some((seconds, budget));
        menu
    }
    fn add_time_meter(&self, face: &mut UndoFace) {
        if !self.draft.active { return; }
        if let Some((seconds, budget)) = self.remaining {
            let meter = slot_ui::kid_time_meter(seconds, budget);
            let x = face.w.saturating_sub(meter.w) / 2;
            let y = (slot_ui::OUT_H as f32 - slot_ui::MOUTH_H
                - slot_ui::MENU_HINT_H as f32 - 24.0 - meter.h as f32).max(0.0) as u32;
            blit(face, &meter, x, y);
        }
    }
    pub fn child_menu(&self) -> bool { self.page == Page::Child }
    fn child_rows(&self) -> Vec<(String, String, bool)> {
        let mut rows = vec![
            (if self.child_in_game { "Resume" } else { "Games" }.into(), String::new(), false),
            ("Volume".into(), format!("{}%", self.child_volume), true),
            ("Brightness".into(), format!("{}%", self.child_brightness), true),
        ];
        if self.child_in_game { rows.push(("Exit game".into(), String::new(), false)); }
        rows.push(("Parent controls".into(), String::new(), false));
        rows
    }
    pub fn set_games(&mut self, games: Vec<slot_store::Cart>) {
        self.games = games;
    }
    pub fn entered_buttons(&self) -> &[u8] {
        &self.code
    }
    pub fn entering_code(&self) -> bool {
        matches!(self.page, Page::Set | Page::Confirm | Page::Unlock)
    }
    pub fn notice(root: PathBuf, policy: Policy, message: &str) -> Self {
        let mut menu = Self::new(root, policy);
        menu.page = Page::Notice;
        menu.status = message.into();
        menu
    }
    pub fn lines(&self) -> Vec<String> {
        if self.page == Page::Warning {
            return vec![slot_ui::kid_time_text(self.remaining_seconds().unwrap_or(0))];
        }
        let title = match self.page {
            Page::Set => "Choose four buttons",
            Page::Confirm => "Repeat your code",
            Page::Unlock => "Parent code",
            Page::Notice => "Time for a break",
            Page::Home | Page::Games | Page::Child | Page::Warning => "",
        };
        if self.page == Page::Child {
            return self.child_rows().into_iter().enumerate().map(|(i, (label, value, _))|
                format!("{} {label} {value}", if i == self.selected { "SELECTED" } else { "ROW" })).collect();
        }
        if self.page == Page::Games {
            let confirming = self.game_removal.is_some_and(|(at, since, _)|
                at == self.game_at && since.elapsed().as_millis() < u128::from(slot_ui::FAVORITE_REMOVE_MS));
            let mut lines = vec![String::new(), if confirming {
                "Press A again to remove approval".into()
            } else { "Approved games · A toggles approval".into() }, "COMPACT".into()];
            if self.games.is_empty() {
                lines.push("ROW No games on the content card".into());
            }
            let first = self.game_at.saturating_sub(2).min(self.games.len().saturating_sub(6));
            for (i, cart) in self.games.iter().enumerate().skip(first).take(6) {
                lines.push(format!("{} {} {} / {}",
                    if i == self.game_at { "SELECTED" } else { "ROW" },
                    if self.draft.allows(cart) { "+" } else { "−" },
                    cart.platform.dir_name(), cart.stem));
            }
            lines.push(if confirming { "PARENT REMOVE HINTS" } else { "PARENT GAMES HINTS" }.into());
            return lines;
        }
        if self.page == Page::Home {
            let value = if self.draft.rule == Rule::Until {
                format!(
                    "Until {:02}:{:02}",
                    self.draft.bedtime / 60,
                    self.draft.bedtime % 60
                )
            } else {
                format!("{} minutes", self.draft.minutes)
            };
            let rows = [
                self.draft.rule.label().to_string(), value,
                format!("Volume limit {}%", self.draft.volume_limit),
                format!("Kid mode {}", if self.draft.active { "On" } else { "Off" }),
                if self.gate.enabled() { "Change parent code" } else { "Set parent code" }.into(),
                format!("Approved games {}", self.draft.approved.len()),
            ];
            let status = if self.status.is_empty() {
                format!(
                    "Kid mode {} · {} approved games",
                    if self.draft.active { "on" } else { "off" },
                    self.draft.approved.len()
                )
            } else {
                self.status.clone()
            };
            let mut lines = vec![String::new(), status, "COMPACT".into()];
            for (i, row) in rows.into_iter().enumerate() {
                lines.push(format!(
                    "{} {row}",
                    if i == self.selected {
                        "SELECTED"
                    } else {
                        "ROW"
                    }
                ));
            }
            lines.push("PARENT HOME HINTS".into());
            return lines;
        }
        let status = if self.page == Page::Unlock && self.gate.remaining(Instant::now()) > 0 {
            format!(
                "Try again in {} seconds",
                self.gate.remaining(Instant::now())
            )
        } else {
            if self.entering_code() && self.status.is_empty() {
                format!("{} of 4 buttons · A/B/X/Y or D-pad", self.code.len())
            } else if self.entering_code() {
                format!("{} · {} of 4 buttons", self.status, self.code.len())
            } else {
                self.status.clone()
            }
        };
        vec![
            title.into(),
            status,
            String::new(),
            if self.page == Page::Notice {
                "PARENT NOTICE HINTS".into()
            } else {
                "PARENT CODE HINTS".into()
            },
        ]
    }
    pub fn screen_face(&self) -> UndoFace {
        if self.page == Page::Warning {
            let mut face = slot_ui::parent_footer_face(5, false);
            let title = slot_ui::quick_value_face(&slot_ui::kid_time_text(self.remaining_seconds().unwrap_or(0)), true);
            let x = face.w.saturating_sub(title.w) / 2;
            blit(&mut face, &title, x, 190);
            return face;
        }
        if self.page == Page::Child {
            let mut face = slot_ui::parent_settings_face(&self.child_rows(), self.selected, 0, "");
            self.add_time_meter(&mut face);
            return face;
        }
        if self.page == Page::Home {
            let kind = match self.draft.rule { Rule::Daily => "Daily", Rule::Session => "Duration", Rule::Until => "Bedtime" };
            let time = if self.draft.rule == Rule::Until {
                format!("{:02}:{:02}", self.draft.bedtime / 60, self.draft.bedtime % 60)
            } else { format!("{} min", self.draft.minutes) };
            let rows = vec![
                ("Play limit".into(), kind.into(), true),
                ("Time".into(), time, true),
                ("Volume limit".into(), format!("{}%", self.draft.volume_limit), true),
                ("Kid mode".into(), if self.draft.active { "On" } else { "Off" }.into(), true),
                ("Parent code".into(), if self.gate.enabled() { "Change" } else { "Set" }.into(), false),
                ("Approved games".into(), self.draft.approved.len().to_string(), false),
            ];
            let status = if self.status == "Parent code saved" { "" } else { &self.status };
            let mut face = slot_ui::parent_settings_face(&rows, self.selected, 0, status);
            if status.is_empty() { self.add_time_meter(&mut face); }
            return face;
        }
        if self.page == Page::Games {
            let rows: Vec<_> = self.games.iter().map(|cart| (
                if cart.title.is_empty() { cart.stem.clone() } else { cart.title.clone() }, if self.draft.allows(cart) { "Yes" } else { "No" }.into(), false
            )).collect();
            let confirming = self.game_removal.is_some_and(|(at, since, _)|
                at == self.game_at && since.elapsed().as_millis() < u128::from(slot_ui::FAVORITE_REMOVE_MS));
            return slot_ui::parent_settings_face(&rows, self.game_at,
                if confirming { 4 } else { 3 }, if rows.is_empty() { "No games on the content card" } else { "" });
        }
        let mut face = slot_ui::parent_footer_face(if self.page == Page::Notice { 2 } else { 1 }, false);
        let title = match self.page {
            Page::Set => "Choose four buttons", Page::Confirm => "Repeat your code",
            Page::Notice => "Time for a break", _ => "Parent code",
        };
        let title = slot_ui::quick_value_face(title, true);
        let left = (face.w - title.w) / 2;
        blit(&mut face, &title, left, 112);
        if self.entering_code() {
            let buttons = slot_ui::button_code_face(&self.code);
            let left = (face.w - buttons.w) / 2;
            blit(&mut face, &buttons, left, 208);
            let count = slot_ui::quick_description_face(&[&format!("{} / 4", self.code.len())]);
            let left = (face.w - count.w) / 2;
            blit(&mut face, &count, left, 284);
        }
        let status = if self.status.starts_with("Recovery:") || self.status.starts_with("Use A/B") {
            ""
        } else { &self.status };
        if !status.is_empty() {
            let status = slot_ui::quick_description_face(&[status]);
            let left = face.w.saturating_sub(status.w) / 2;
            blit(&mut face, &status, left, 332);
        }
        face
    }
    fn save(&mut self, policy: Policy) {
        match policy.save(&self.root) {
            Ok(()) => {
                self.draft = policy.clone();
                self.changed = Some(policy);
                self.visible = false;
            }
            Err(_) => self.status = "Could not save parent settings".into(),
        }
    }
    fn save_open(&mut self, policy: Policy) {
        self.save(policy);
        if self.changed.is_some() { self.visible = true; self.status.clear(); }
    }
    fn toggle_mode(&mut self, wall: i64) {
        let mut next = self.draft.clone();
        if next.active {
            next.leave();
            self.save_open(next);
        } else {
            if let Err(e) = next.begin(wall) { self.status = e.to_string(); return; }
            if !self.gate.enabled() {
                self.start_after_code = true;
                self.page = Page::Set;
                self.status.clear();
                self.code.clear();
            } else {
                self.save_open(next);
            }
        }
    }
    pub fn input(&mut self, action: Action, wall: i64) {
        self.face = None;
        if action == Action::QuickMenu {
            self.visible = false;
            return;
        }
        if self.page == Page::Warning {
            match action {
                Action::GbaDown(Btn::A) => self.visible = false,
                Action::GbaDown(Btn::B) => { self.child_action = Some(Action::Eject); self.visible = false; }
                _ => {}
            }
            return;
        }
        if self.page == Page::Child {
            let count = self.child_rows().len();
            match action {
                Action::GbaDown(Btn::B) => self.visible = false,
                Action::GbaDown(Btn::Up) => self.selected = (self.selected + count - 1) % count,
                Action::GbaDown(Btn::Down) => self.selected = (self.selected + 1) % count,
                Action::GbaDown(Btn::Left) | Action::GbaDown(Btn::Right) => {
                    let up = action == Action::GbaDown(Btn::Right);
                    self.child_action = match self.selected {
                        1 => Some(if up { Action::VolumeUp } else { Action::VolumeDown }),
                        2 => Some(if up { Action::BrightnessUp } else { Action::BrightnessDown }),
                        _ => None,
                    };
                }
                Action::GbaDown(Btn::A) => match self.selected {
                    0 => self.visible = false,
                    1 | 2 => self.input(Action::GbaDown(Btn::Right), wall),
                    n if n == count - 1 => { self.page = Page::Unlock; self.selected = 0; self.code.clear(); self.status.clear(); }
                    _ if self.child_in_game => { self.child_action = Some(Action::Eject); self.visible = false; }
                    _ => {}
                },
                _ => {}
            }
            return;
        }
        if self.page == Page::Notice {
            match action {
                Action::GbaDown(Btn::A) => {
                    self.page = Page::Unlock;
                    self.status.clear();
                }
                Action::GbaDown(Btn::B) => self.visible = false,
                _ => {}
            }
            return;
        }
        if self.page == Page::Games {
            if self.game_removal.is_some_and(|(_, since, _)|
                since.elapsed().as_millis() >= u128::from(slot_ui::FAVORITE_REMOVE_MS)) {
                self.game_removal = None;
            }
            match action {
                Action::GbaUp(Btn::A) => {
                    if let Some((_, _, released)) = &mut self.game_removal { *released = true; }
                }
                Action::GbaDown(Btn::A) | Action::GbaUp(_) => {}
                _ => self.game_removal = None,
            }
            match action {
                Action::GbaDown(Btn::B) => self.page = Page::Home,
                Action::GbaDown(Btn::Up) if !self.games.is_empty() => {
                    self.game_at = (self.game_at + self.games.len() - 1) % self.games.len();
                }
                Action::GbaDown(Btn::Down) if !self.games.is_empty() => {
                    self.game_at = (self.game_at + 1) % self.games.len();
                }
                Action::GbaDown(Btn::A) => {
                    if let Some(cart) = self.games.get(self.game_at) {
                        if self.draft.allows(cart) {
                            match self.game_removal {
                                Some((at, _, true)) if at == self.game_at => {}
                                Some((at, _, false)) if at == self.game_at => return,
                                _ => {
                                    self.game_removal = Some((self.game_at, Instant::now(), false));
                                    return;
                                }
                            }
                        }
                        self.game_removal = None;
                        self.draft.toggle(cart);
                        self.save_open(self.draft.clone());
                    }
                }
                _ => {}
            }
            return;
        }
        if self.page == Page::Home {
            match action {
                Action::GbaDown(Btn::B) => self.visible = false,
                Action::GbaDown(Btn::Up) => self.selected = (self.selected + 5) % 6,
                Action::GbaDown(Btn::Down) => self.selected = (self.selected + 1) % 6,
                Action::GbaDown(Btn::Left) | Action::GbaDown(Btn::Right) => {
                    if self.selected == 3 {
                        self.toggle_mode(wall);
                        return;
                    }
                    let by = if action == Action::GbaDown(Btn::Left) {
                        -1
                    } else {
                        1
                    };
                    if self.selected == 0 {
                        self.draft.rule = self.draft.rule.next(by);
                    }
                    if self.selected == 2 {
                        self.draft.volume_limit =
                            (self.draft.volume_limit as i32 + 10 * by).clamp(10, 80) as u8;
                    }
                    if self.selected == 1 {
                        if self.draft.rule == Rule::Until {
                            self.draft.bedtime =
                                (self.draft.bedtime as i32 + 15 * by).rem_euclid(1440) as u32;
                        } else {
                            self.draft.minutes =
                                (self.draft.minutes as i32 + 5 * by).clamp(5, 240) as u32;
                        }
                    }
                    if self.selected < 3 {
                        let mut next = self.draft.clone();
                        if next.active && self.selected < 2 {
                            if let Err(e) = next.begin(wall) { self.status = e.to_string(); return; }
                        }
                        self.save_open(next);
                    }
                }
                Action::GbaDown(Btn::A) => match self.selected {
                    0 | 1 | 2 => self.input(Action::GbaDown(Btn::Right), wall),
                    3 => self.toggle_mode(wall),
                    4 => {
                        self.start_after_code = false;
                        self.page = Page::Set;
                        self.status.clear();
                        self.code.clear();
                    }
                    5 => self.page = Page::Games,
                    _ => {}
                },
                _ => {}
            }
            return;
        }
        if action == Action::GbaDown(Btn::Select) {
            self.code.pop();
            return;
        }
        if action == Action::GbaDown(Btn::Start) {
            if self.code.len() != 4 {
                self.status = "Enter all four buttons".into();
                return;
            }
            match self.page {
                Page::Unlock => match self.gate.verify(&self.code, Instant::now()) {
                    Ok(true) => {
                        self.page = Page::Home;
                        self.status.clear();
                    }
                    Ok(false) => self.status = "Code not recognized".into(),
                    Err(_) => self.status = "Parent code could not be read".into(),
                },
                Page::Set => {
                    self.first = self.code.clone();
                    self.page = Page::Confirm;
                    self.status.clear();
                }
                Page::Confirm => {
                    if self.first != self.code {
                        self.page = Page::Set;
                        self.status = "Codes did not match. Try again.".into();
                    } else if self
                        .gate
                        .set(&self.code)
                        .and_then(|_| self.draft.save(&self.root))
                        .is_ok()
                    {
                        self.page = Page::Home;
                        self.status.clear();
                        if self.start_after_code {
                            let mut next = self.draft.clone();
                            match next.begin(wall) {
                                Ok(()) => self.save_open(next),
                                Err(e) => self.status = e.to_string(),
                            }
                        }
                        self.start_after_code = false;
                    } else {
                        self.status = "Could not save parent code".into();
                    }
                    self.first.fill(0);
                    self.first.clear();
                }
                _ => {}
            }
            self.code.fill(0);
            self.code.clear();
            return;
        }
        let button = match action {
            Action::GbaDown(Btn::A) => Some(0),
            Action::GbaDown(Btn::B) => Some(1),
            Action::GbaDown(Btn::X) => Some(2),
            Action::GbaDown(Btn::Y) => Some(3),
            Action::GbaDown(Btn::Up) => Some(4),
            Action::GbaDown(Btn::Down) => Some(5),
            Action::GbaDown(Btn::Left) => Some(6),
            Action::GbaDown(Btn::Right) => Some(7),
            _ => None,
        };
        if let Some(button) = button {
            if self.code.len() < 4 {
                self.code.push(button);
            }
        }
    }
}
fn blit(dst: &mut UndoFace, src: &UndoFace, left: u32, top: u32) {
    for y in 0..src.h.min(dst.h.saturating_sub(top)) {
        for x in 0..src.w.min(dst.w.saturating_sub(left)) {
            let s = ((y * src.w + x) * 4) as usize;
            let d = (((y + top) * dst.w + x + left) * 4) as usize;
            let a = u32::from(src.rgba[s + 3]);
            let behind = u32::from(dst.rgba[d + 3]) * (255 - a) / 255;
            let alpha = a + behind;
            if alpha == 0 { continue; }
            for c in 0..3 {
                dst.rgba[d + c] = ((u32::from(src.rgba[s + c]) * a
                    + u32::from(dst.rgba[d + c]) * behind) / alpha) as u8;
            }
            dst.rgba[d + 3] = alpha as u8;
        }
    }
}

impl Drop for Menu {
    fn drop(&mut self) {
        use zeroize::Zeroize;
        self.code.zeroize();
        self.first.zeroize();
    }
}
