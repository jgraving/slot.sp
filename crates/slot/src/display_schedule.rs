//! Small editor for local-time display scheduling; the app owns persistence.
use crate::display_settings::DisplaySettings;
use slot_input::{Action, Btn};
use slot_ui::{TexId, UndoFace};

pub struct Menu {
    pub settings: DisplaySettings,
    selected: usize,
    time_field: Option<usize>,
    status: String,
    pub face: Option<TexId>,
    pub visible: bool,
}

impl Menu {
    pub fn new(settings: DisplaySettings) -> Self {
        Self { settings, selected: 0, time_field: None, status: String::new(), face: None, visible: true }
    }
    /// Returns updated schedule preferences, including the shared night warmth preset.
    pub fn input(&mut self, action: Action, clock_set: bool) -> Option<DisplaySettings> {
        self.face = None;
        let Action::GbaDown(button) = action else {
            if action == Action::QuickMenu { self.visible = false; }
            return None;
        };
        if button == Btn::B {
            if self.time_field.take().is_none() { self.visible = false; }
            self.status.clear();
            return None;
        }
        if matches!(button, Btn::Up | Btn::Down) {
            if let Some(field) = &mut self.time_field { *field = 1 - *field; }
            else { self.selected = (self.selected + if button == Btn::Down {1} else {3}) % 4; }
            self.status.clear();
            return None;
        }
        if button == Btn::A && matches!(self.selected,1|2) {
            self.time_field = Some(0);
            return None;
        }
        if !matches!(button, Btn::A | Btn::Left | Btn::Right) { return None; }
        let mut next = self.settings;
        if self.selected == 0 {
            if !clock_set { self.status = "Set Date & Time first".into(); return None; }
            next.schedule = !next.schedule;
        } else if self.selected==3 {
            next.bedtime.1=if button==Btn::Left {next.bedtime.1.saturating_sub(1)} else {next.bedtime.1.saturating_add(1).min(9)};
        } else {
            let step = match self.time_field { Some(0) => 60, Some(_) => 1, None => 15 };
            let delta = if button == Btn::Left { -step } else { step };
            let value = if self.selected == 1 { &mut next.day_start } else { &mut next.night_start };
            *value = (i32::from(*value) + delta).rem_euclid(1440) as u16;
            if next.day_start == next.night_start {
                self.status = "Day and night need different times".into(); return None;
            }
        }
        next.override_until = None;
        self.settings = next;
        self.status.clear();
        Some(next)
    }
    pub fn notice_text(&self) -> Option<&str> { (!self.status.is_empty()).then_some(self.status.as_str()) }
    pub fn focus_info(&self) -> (usize,usize) { (self.time_field.unwrap_or(self.selected),0) }
    pub fn page_key(&self) -> u64 { 3000 + u64::from(self.time_field.is_some()) }
    pub fn screen_face(&self) -> UndoFace { self.screen_face_on(slot_ui::Canvas::FRAMED) }
    pub fn visible_sliders(&self)->Vec<slot_ui::MenuSlider> {
        if self.time_field.is_none(){vec![slot_ui::MenuSlider{span:slot_ui::MENU_SLIDER_SPAN,id:self.page_key()*16+3,fraction:self.settings.bedtime.1 as f32/9.0,y:94.0+3.0*slot_ui::QUICK_PITCH}]}else{vec![]}
    }
    pub fn visible_switches(&self) -> Vec<slot_ui::MenuSwitch> {
        if self.time_field.is_none() {vec![slot_ui::MenuSwitch{id:self.page_key()*16,on:self.settings.schedule,y:94.0}]} else {vec![]}
    }
    pub fn screen_face_on(&self, canvas: slot_ui::Canvas) -> UndoFace {self.render_face(canvas,false)}
    pub fn screen_face_animated_on(&self, canvas: slot_ui::Canvas) -> UndoFace {self.render_face(canvas,true)}
    fn render_face(&self, canvas: slot_ui::Canvas, animated: bool) -> UndoFace {
        let settings_face=if animated {slot_ui::parent_settings_face_animated_on} else {slot_ui::parent_settings_face_on};
        let (title, rows, selected) = if let Some(field) = self.time_field {
            let minute = if self.selected == 1 { self.settings.day_start } else { self.settings.night_start };
            (if self.selected == 1 { "Day Begins" } else { "Night Begins" }, vec![
                ("Hours".into(), format!("{:02}", minute / 60), true),
                ("Minutes".into(), format!("{:02}", minute % 60), true),
            ], field)
        } else {
            ("Night mode", vec![
                ("Automatic".into(), if self.settings.schedule { "On" } else { "Off" }.into(), true),
                ("Day begins".into(), time(self.settings.day_start), true),
                ("Night begins".into(), time(self.settings.night_start), true),
                ("Color temp".into(),slot_ui::QuickValue::temperature(self.settings.bedtime.1).settings_text().to_string(),true),
            ], self.selected)
        };
        let status = if animated { "" } else if self.status.is_empty() && self.time_field.is_none() { "A on a time edits hours and minutes" } else { &self.status };
        let mut face = settings_face(canvas, &rows, selected, 0, status, 0.0);
        let heading = slot_ui::quick_heading_face(title);
        let x = slot_ui::QUICK_LABEL_X as u32 - slot_ui::MENU_PAD;
        for y in 0..heading.h.min(face.h.saturating_sub(40)) {
            for xx in 0..heading.w.min(face.w.saturating_sub(x)) {
                let from = ((y * heading.w + xx) * 4) as usize;
                let to = (((y + 40) * face.w + x + xx) * 4) as usize;
                if heading.rgba[from+3] > 0 { face.rgba[to..to+4].copy_from_slice(&heading.rgba[from..from+4]); }
            }
        }
        face
    }
}
fn time(minutes: u16) -> String { format!("{:02}:{:02}", minutes / 60, minutes % 60) }
