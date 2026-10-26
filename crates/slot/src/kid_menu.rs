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
    Time,
    Unlock,
    Set,
    Confirm,
    Notice,
    Warning,
    Reference,
}
pub struct Menu {
    root: PathBuf,
    gate: Gate,
    pub draft: Policy,
    page: Page,
    selected: usize,
    code: Vec<u8>,
    first: Vec<u8>,
    time_selected: usize,
    rejected: Option<f32>,
    repeat: Option<(Btn, f32)>,
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
    reference: Vec<crate::reference::Page>,
    reference_at: usize,
    reference_child: bool,
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
            time_selected: 0,
            rejected: None,
            repeat: None,
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
            reference: Vec::new(),
            reference_at: 0,
            reference_child: false,
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
    pub fn set_reference(&mut self, pages: Vec<crate::reference::Page>) { self.reference = pages; }
    pub fn reference(root: PathBuf, policy: Policy, pages: Vec<crate::reference::Page>) -> Self {
        let mut menu = Self::new(root,policy);
        menu.page = Page::Reference;
        menu.reference = pages;
        menu
    }
    pub fn reference_open(&self) -> bool { self.page == Page::Reference }
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
        rows.push(("Buttons & shortcuts".into(), String::new(), false));
        rows.push(("Parent controls".into(), String::new(), false));
        rows
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
        if self.page == Page::Reference {
            return self.reference.get(self.reference_at).map(|(title,rows)| {
                let mut lines=vec![title.clone(),format!("Page {} / {}",self.reference_at+1,self.reference.len())];
                lines.extend(rows.iter().map(|(a,b,_)| format!("{a}: {b}"))); lines
            }).unwrap_or_default();
        }
        if self.page == Page::Warning {
            return vec![slot_ui::kid_time_text(self.remaining_seconds().unwrap_or(0))];
        }
        let title = match self.page {
            Page::Set => "Choose four buttons",
            Page::Confirm => "Repeat your code",
            Page::Unlock => "Parent code",
            Page::Notice => "Time for a break",
            Page::Home | Page::Time | Page::Child | Page::Warning | Page::Reference => "",
        };
        if self.page == Page::Child {
            return self.child_rows().into_iter().enumerate().map(|(i, (label, value, _))|
                format!("{} {label} {value}", if i == self.selected { "SELECTED" } else { "ROW" })).collect();
        }
        if self.page == Page::Time {
            let minutes = self.time_minutes();
            return vec![String::new(), format!("Hours {}", minutes / 60),
                format!("Minutes {}", minutes % 60), self.status.clone()];
        }
        if self.page == Page::Home {
            let value = if self.draft.rule == Rule::Until {
                format!(
                    "Until {:02}:{:02}",
                    self.draft.bedtime / 60,
                    self.draft.bedtime % 60
                )
            } else {
                duration_label(self.draft.minutes)
            };
            let rows = [
                format!("Kid mode {}", if self.draft.active { "On" } else { "Off" }),
                self.draft.rule.label().to_string(), value,
                format!("Volume limit {}%", self.draft.volume_limit),
                "Set parent code".into(),
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
        if self.page == Page::Reference {
            let mut face = slot_ui::parent_footer_face(6,false);
            if let Some((title,rows)) = self.reference.get(self.reference_at) {
                let title=slot_ui::quick_heading_face(title);
                let x=face.w.saturating_sub(title.w)/2;
                blit(&mut face,&title,x,40);
                for (i,(label,value,_)) in rows.iter().take(6).enumerate() {
                    let label=slot_ui::quick_value_face(label,false);
                    let value=slot_ui::quick_value_face(value,true);
                    let right=face.w.saturating_sub(value.w+32);
                    blit(&mut face,&label,32,108+i as u32*42);
                    blit(&mut face,&value,right,108+i as u32*42);
                }
                let page=slot_ui::quick_description_face(&[&format!("{} / {}",self.reference_at+1,self.reference.len())]);
                let x=face.w.saturating_sub(page.w)/2;
                blit(&mut face,&page,x,370);
            }
            return face;
        }
        if self.page == Page::Warning {
            let mut face = slot_ui::parent_footer_face(5, false);
            let title = slot_ui::quick_heading_face(&slot_ui::kid_time_text(self.remaining_seconds().unwrap_or(0)));
            let x = face.w.saturating_sub(title.w) / 2;
            blit(&mut face, &title, x, 190);
            return face;
        }
        if self.page == Page::Child {
            let mut face = slot_ui::parent_settings_face_with_space(&self.child_rows(), self.selected, 0, "", if self.child_in_game {32.0} else {0.0});
            self.add_time_meter(&mut face);
            return face;
        }
        if self.page == Page::Home {
            let kind = match self.draft.rule { Rule::Daily => "Daily", Rule::Session => "Duration", Rule::Until => "Bedtime" };
            let time = if self.draft.rule == Rule::Until {
                format!("{:02}:{:02}", self.draft.bedtime / 60, self.draft.bedtime % 60)
            } else { duration_label(self.draft.minutes) };
            let rows = vec![
                ("Kid mode".into(), if self.draft.active { "On" } else { "Off" }.into(), true),
                ("Play limit".into(), kind.into(), true),
                ("Time".into(), time, true),
                ("Volume limit".into(), format!("{}%", self.draft.volume_limit), true),
                ("Set parent code".into(), String::new(), false),
            ];
            let status = if self.status == "Parent code saved" { "" } else { &self.status };
            let mut face = slot_ui::parent_settings_face(&rows, self.selected, 0, status);
            if status.is_empty() { self.add_time_meter(&mut face); }
            return face;
        }
        if self.page == Page::Time {
            let minutes = self.time_minutes();
            let rows = vec![("Hours".into(), (minutes / 60).to_string(), true),
                ("Minutes".into(), format!("{:02}", minutes % 60), true)];
            let mut face = slot_ui::parent_settings_face(&rows, self.time_selected, 0, &self.status);
            if self.status.is_empty() { self.add_time_meter(&mut face); }
            return face;
        }
        let mut face = slot_ui::parent_footer_face(if self.page == Page::Notice { 2 } else { 1 }, false);
        let title = match self.page {
            Page::Set => "Choose four buttons", Page::Confirm => "Repeat your code",
            Page::Notice => "Time for a break", _ => "Parent code",
        };
        let title = slot_ui::quick_heading_face(title);
        let left = (face.w - title.w) / 2;
        blit(&mut face, &title, left, 112);
        if self.entering_code() {
            let buttons = slot_ui::button_code_face(&self.code);
            let left = ((face.w - buttons.w) as i32 / 2 + self.code_offset()) as u32;
            blit(&mut face, &buttons, left, 208);
            let count = slot_ui::quick_description_face(&[&format!("{} / 4", self.code.len())]);
            let left = (face.w - count.w) / 2;
            blit(&mut face, &count, left, 284);
        }
        let cooldown = self.gate.remaining(Instant::now());
        let retry = format!("Try again in {cooldown} seconds");
        let status = if self.page == Page::Unlock && cooldown > 0 { &retry }
        else if self.status.starts_with("Recovery:") || self.status.starts_with("Use A/B") {
            ""
        } else { &self.status };
        if !status.is_empty() {
            let status = slot_ui::quick_description_face(&[status]);
            let left = face.w.saturating_sub(status.w) / 2;
            blit(&mut face, &status, left, 332);
        }
        face
    }
    pub fn code_offset(&self) -> i32 {
        let Some(elapsed) = self.rejected else { return 0; };
        let u = (elapsed / 0.36).clamp(0.0, 1.0);
        ((u * std::f32::consts::TAU * 3.0).sin() * 10.0 * (1.0 - u)).round() as i32
    }
    pub fn stop_repeat(&mut self) { self.repeat = None; }
    pub fn update(&mut self, dt: f32) -> Option<Action> {
        if let Some(elapsed) = &mut self.rejected {
            *elapsed += dt.max(0.0);
            if *elapsed >= 0.36 {
                self.clear_code();
                self.rejected = None;
            }
            self.face = None;
        }
        if dt > 0.25 { self.stop_repeat(); }
        if let Some((button, wait)) = &mut self.repeat {
            *wait -= dt.max(0.0);
            if *wait <= 0.0 {
                *wait = 0.08;
                return Some(Action::GbaDown(*button));
            }
        }
        None
    }
    fn clear_code(&mut self) { self.code.fill(0); self.code.clear(); }
    fn complete_code(&mut self, wall: i64) {
        match self.page {
            Page::Unlock => match self.gate.verify(&self.code, Instant::now()) {
                Ok(true) => { self.page = Page::Home; self.status.clear(); self.clear_code(); }
                Ok(false) => { self.status.clear(); self.rejected = Some(0.0); }
                Err(_) => { self.status = "Parent code could not be read".into(); self.rejected = Some(0.0); }
            },
            Page::Set => {
                self.first = self.code.clone();
                self.clear_code();
                self.page = Page::Confirm;
                self.status.clear();
            }
            Page::Confirm => {
                if self.first != self.code {
                    self.status.clear();
                    self.rejected = Some(0.0);
                    return;
                }
                if self.gate.set(&self.code).and_then(|_| self.draft.save(&self.root)).is_ok() {
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
                    self.clear_code();
                    return;
                }
                self.first.fill(0);
                self.first.clear();
                self.clear_code();
            }
            _ => {}
        }
    }
    fn time_minutes(&self) -> u32 {
        if self.draft.rule == Rule::Until { self.draft.bedtime } else { self.draft.minutes }
    }
    fn adjust_time(&mut self, by: i32, wall: i64) {
        let mut next = self.draft.clone();
        if next.rule == Rule::Until {
            next.bedtime = (next.bedtime as i32 + by).rem_euclid(1440) as u32;
        } else { next.minutes = (next.minutes as i32 + by).clamp(5, 240) as u32; }
        if next.active {
            if let Err(e) = next.begin(wall) {
                self.stop_repeat();
                self.status = e.to_string();
                return;
            }
        }
        self.save_open(next);
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
            if let Err(e) = next.begin(wall) {
                self.status = if next.approved.is_empty() { "Mark games with X on the carousel".into() } else { e.to_string() };
                return;
            }
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
        match action {
            Action::GbaDown(button @ (Btn::Left | Btn::Right))
                if self.page == Page::Time || (self.page == Page::Home && self.selected == 2) => {
                if self.repeat.is_none_or(|(held, _)| held != button) { self.repeat = Some((button, 0.4)); }
            }
            Action::GbaUp(button) if self.repeat.is_some_and(|(held, _)| held == button) => self.stop_repeat(),
            Action::GbaDown(_) | Action::QuickMenu => self.stop_repeat(),
            _ => {}
        }
        if action == Action::QuickMenu {
            self.visible = false;
            return;
        }
        if self.page == Page::Reference {
            match action {
                Action::GbaDown(Btn::B) => if self.reference_child { self.page=Page::Child; } else { self.visible=false; },
                Action::GbaDown(Btn::Left | Btn::Right) if !self.reference.is_empty() => {
                    let n=self.reference.len();
                    self.reference_at=(self.reference_at+if action==Action::GbaDown(Btn::Right) {1} else {n-1})%n;
                }
                _=>{}
            }
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
                    n if n == count - 2 => { self.page=Page::Reference; self.reference_at=0; self.reference_child=true; },
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
        if self.page == Page::Time {
            match action {
                Action::GbaDown(Btn::B | Btn::A) => self.page = Page::Home,
                Action::GbaDown(Btn::Up | Btn::Down) => self.time_selected = 1 - self.time_selected,
                Action::GbaDown(Btn::Left | Btn::Right) => {
                    let by = if action == Action::GbaDown(Btn::Left) { -1 } else { 1 };
                    self.adjust_time(by * if self.time_selected == 0 { 60 } else { 1 }, wall);
                }
                _ => {}
            }
            return;
        }
        if self.page == Page::Home {
            match action {
                Action::GbaDown(Btn::B) => self.visible = false,
                Action::GbaDown(Btn::Up) => self.selected = (self.selected + 4) % 5,
                Action::GbaDown(Btn::Down) => self.selected = (self.selected + 1) % 5,
                Action::GbaDown(Btn::Left) | Action::GbaDown(Btn::Right) => {
                    if self.selected == 0 {
                        self.toggle_mode(wall);
                        return;
                    }
                    let by = if action == Action::GbaDown(Btn::Left) {
                        -1
                    } else {
                        1
                    };
                    if self.selected == 2 {
                        self.adjust_time(by, wall);
                        return;
                    }
                    if self.selected == 1 { self.draft.rule = self.draft.rule.next(by); }
                    if self.selected == 3 {
                        self.draft.volume_limit = (self.draft.volume_limit as i32 + 10 * by).clamp(10, 80) as u8;
                    }
                    if matches!(self.selected, 1 | 3) {
                        let mut next = self.draft.clone();
                        if next.active && self.selected == 1 {
                            if let Err(e) = next.begin(wall) { self.status = e.to_string(); return; }
                        }
                        self.save_open(next);
                    }
                }
                Action::GbaDown(Btn::A) => match self.selected {
                    0 => self.toggle_mode(wall),
                    1 | 3 => self.input(Action::GbaDown(Btn::Right), wall),
                    2 => { self.time_selected = 0; self.page = Page::Time; },
                    4 => {
                        self.start_after_code = false;
                        self.page = Page::Set;
                        self.status.clear();
                        self.code.clear();
                    }
                    _ => {}
                },
                _ => {}
            }
            return;
        }
        if self.rejected.is_some() { return; }
        if action == Action::GbaDown(Btn::Select) {
            self.code.pop();
            return;
        }
        if self.page == Page::Unlock && self.gate.remaining(Instant::now()) > 0 { return; }
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
                if self.code.len() == 4 { self.complete_code(wall); }
            }
        }
    }
}
fn duration_label(minutes: u32) -> String {
    match (minutes / 60, minutes % 60) {
        (0, m) => format!("{m} min"),
        (h, 0) => format!("{h} h"),
        (h, m) => format!("{h} h {m} min"),
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
