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
    Games,
    Unlock,
    Set,
    Confirm,
    Notice,
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
    start_after_code: bool,
    status: String,
    pub visible: bool,
    pub face: Option<TexId>,
    pub changed: Option<Policy>,
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
            start_after_code: false,
            status: if page == Page::Unlock {
                "Recovery: Config/kids/parent-code.txt".into()
            } else {
                String::new()
            },
            visible: true,
            face: None,
            changed: None,
        }
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
        let title = match self.page {
            Page::Set => "Choose four buttons",
            Page::Confirm => "Repeat your code",
            Page::Unlock => "Parent code",
            Page::Notice => "Time for a break",
            Page::Home | Page::Games => "",
        };
        if self.page == Page::Games {
            let mut lines = vec![String::new(), "Approved games · A toggles approval".into(), "COMPACT".into()];
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
            lines.push("PARENT GAMES HINTS".into());
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
                self.draft.rule.label().to_string(),
                value,
                format!("Volume limit {}%", self.draft.volume_limit),
                if self.draft.active {
                    "Apply limits"
                } else {
                    "Start kid mode"
                }
                .into(),
                if self.draft.active {
                    "Exit kid mode"
                } else {
                    "Back to games"
                }
                .into(),
                if self.gate.enabled() { "Change parent code" } else { "Set parent code" }.into(),
                "Approved games".into(),
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
        let mut face = crate::bluetooth::screen_face(&self.lines());
        if matches!(self.page, Page::Set | Page::Confirm | Page::Unlock) {
            let buttons = slot_ui::button_code_face(&self.code);
            let left = (face.w - buttons.w) / 2;
            blit(&mut face, &buttons, left, 208);
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
    pub fn input(&mut self, action: Action, wall: i64) {
        self.face = None;
        if action == Action::QuickMenu {
            self.visible = false;
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
                        self.draft.toggle(cart);
                    }
                }
                _ => {}
            }
            return;
        }
        if self.page == Page::Home {
            match action {
                Action::GbaDown(Btn::B) => {
                    if self.draft.active {
                        self.visible = false;
                    } else {
                        self.save(self.draft.clone());
                    }
                }
                Action::GbaDown(Btn::Up) => self.selected = (self.selected + 6) % 7,
                Action::GbaDown(Btn::Down) => self.selected = (self.selected + 1) % 7,
                Action::GbaDown(Btn::Left) | Action::GbaDown(Btn::Right) => {
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
                }
                Action::GbaDown(Btn::A) => match self.selected {
                    0 => self.draft.rule = self.draft.rule.next(1),
                    1 | 2 => self.input(Action::GbaDown(Btn::Right), wall),
                    3 if !self.gate.enabled() => {
                        self.start_after_code = true;
                        self.page = Page::Set;
                        self.status = "Use A/B/X/Y or the D-pad".into();
                        self.code.clear();
                    }
                    3 => {
                        let mut next = self.draft.clone();
                        match next.begin(wall) {
                            Ok(()) => self.save(next),
                            Err(e) => self.status = e.to_string(),
                        }
                    }
                    4 => {
                        let mut next = self.draft.clone();
                        next.leave();
                        self.save(next);
                    }
                    5 => {
                        self.start_after_code = false;
                        self.page = Page::Set;
                        self.status.clear();
                        self.code.clear();
                    }
                    6 => self.page = Page::Games,
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
                        self.status = "Parent code saved".into();
                        if self.start_after_code {
                            let mut next = self.draft.clone();
                            match next.begin(wall) {
                                Ok(()) => self.save(next),
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
