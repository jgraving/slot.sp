//! Day/night display presets, with read-only migration from the earlier fork.
use slot_store::SlotState;
use std::{io, path::Path};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisplaySettings {
    pub night: bool,
    pub day: (u8, u8),
    pub bedtime: (u8, u8),
    pub schedule: bool,
    pub day_start: u16,
    pub night_start: u16,
    pub override_until: Option<i64>,
}
impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            night: false,
            day: (9, 0),
            bedtime: (2, 9),
            schedule: false,
            day_start: 7 * 60,
            night_start: 20 * 60,
            override_until: None,
        }
    }
}
impl DisplaySettings {
    pub fn read(root: &Path, state: &SlotState) -> Self {
        let mut s = Self {
            day: (state.brightness, state.blue_light),
            ..Self::default()
        };
        let text = match std::fs::read_to_string(root.join("Config/display.txt")) {
            Ok(text) => text,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                std::fs::read_to_string(root.join("Config/comfort.txt")).unwrap_or_default()
            }
            Err(_) => String::new(),
        };
        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            if key == "night" {
                s.night = value == "1";
            }
            match key {
                "schedule" => s.schedule = value == "1",
                "day_start" | "night_start" => {
                    if let Ok(minutes @ 0..=1439) = value.parse::<u16>() {
                        if key == "day_start" { s.day_start = minutes; } else { s.night_start = minutes; }
                    }
                }
                "override_until" => s.override_until = value.parse().ok(),
                _ => {}
            }
            if key == "day" || key == "bedtime" {
                if let Some((b, w)) = value.split_once(',') {
                    if let (Ok(b @ 0..=9), Ok(w @ 0..=9)) = (b.parse::<u8>(), w.parse::<u8>()) {
                        if key == "day" {
                            s.day = (b, w);
                        } else {
                            s.bedtime = (b, w);
                        }
                    }
                }
            }
        }
        s
    }
    pub fn next_change(&self, wall: i64) -> i64 {
        let midnight = wall.div_euclid(86_400) * 86_400;
        [self.day_start, self.night_start].map(|m| {
            let at = midnight + i64::from(m) * 60;
            if at > wall { at } else { at + 86_400 }
        }).into_iter().min().unwrap()
    }
    pub fn overridden(&self, wall: i64) -> bool {
        self.override_until.is_some_and(|until| (1..=86_400).contains(&until.saturating_sub(wall)))
    }
    pub fn scheduled_night(&self, wall: i64) -> Option<bool> {
        if !self.schedule || self.day_start == self.night_start || self.overridden(wall) { return None; }
        let minute = (wall.rem_euclid(86_400) / 60) as u16;
        let day = if self.day_start < self.night_start {
            minute >= self.day_start && minute < self.night_start
        } else { minute >= self.day_start || minute < self.night_start };
        Some(!day)
    }
    pub fn current(&self) -> (u8, u8) {
        if self.night {
            self.bedtime
        } else {
            self.day
        }
    }
    pub fn remember(&mut self, b: u8, w: u8) {
        if self.night {
            self.bedtime = (b, w);
        } else {
            self.day = (b, w);
        }
    }
    pub fn save(&self, root: &Path) -> io::Result<()> {
        std::fs::create_dir_all(root.join("Config"))?;
        slot_store::atomic_write(
            &root.join("Config/display.txt"),
            format!(
                "night={}\nday={},{}\nbedtime={},{}\nschedule={}\nday_start={}\nnight_start={}\noverride_until={}\n",
                u8::from(self.night),
                self.day.0,
                self.day.1,
                self.bedtime.0,
                self.bedtime.1,
                u8::from(self.schedule), self.day_start, self.night_start,
                self.override_until.map(|v| v.to_string()).unwrap_or_default()
            )
            .as_bytes(),
        )
    }
}
