use slot_gfx::{Canvas, Draw, TexId, OUT_H};
use slot_store::{
    civil_from_days, days_from_civil, days_in_month, parse_stamp, UTC_OFFSET_MAX, UTC_OFFSET_MIN,
};

use crate::plate::{blit, UndoFace};
use crate::text;

const DAY: i64 = 86_400;

const YEAR_MIN: i64 = 2000;
const YEAR_MAX: i64 = 2099;

const INK: [u8; 3] = [0xf6, 0xf4, 0xef];
const BACKDROP: [f32; 4] = [0.06, 0.06, 0.07, 1.0];

const OFFSET_STEP: i64 = 30;

pub fn clock_label(stamp: &str) -> String {
    parse_stamp(stamp).map(hhmm).unwrap_or_default()
}

pub fn hhmm(secs: i64) -> String {
    let rem = secs.rem_euclid(DAY);
    format!("{:02}:{:02}", rem / 3600, rem / 60 % 60)
}

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

pub fn date_time_text(secs: i64) -> String {
    let (_, month, day) = civil_from_days(secs.div_euclid(DAY));
    let name = MONTHS
        .get((month - 1) as usize)
        .copied()
        .unwrap_or_default();
    format!("{name} {day} {}", hhmm(secs))
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Field {
    Year,
    Month,
    Day,
    Hour,
    Minute,
    Offset,
}

impl Field {
    pub const ALL: [Field; 6] = [
        Field::Year,
        Field::Month,
        Field::Day,
        Field::Hour,
        Field::Minute,
        Field::Offset,
    ];
}

#[derive(Clone, Debug)]
pub struct ClockPicker {
    year: i64,
    month: i64,
    day: i64,
    hour: i64,
    minute: i64,
    offset_min: i64,
    cursor: usize,
    editing: bool,
}

impl ClockPicker {
    pub fn from_secs(secs: i64) -> Self {
        let (year, month, day) = civil_from_days(secs.div_euclid(DAY));
        let rem = secs.rem_euclid(DAY);
        ClockPicker {
            year: year.clamp(YEAR_MIN, YEAR_MAX),
            month,
            day,
            hour: rem / 3600,
            minute: rem / 60 % 60,
            offset_min: 0,
            cursor: 0,
            editing: false,
        }
    }

    pub fn local(utc: i64, offset_min: i16) -> Self {
        let offset = i64::from(offset_min);
        ClockPicker {
            offset_min: offset,
            ..ClockPicker::from_secs(utc + offset * 60)
        }
    }

    pub fn from_ymd(year: i64, month: i64, day: i64) -> Self {
        ClockPicker {
            year: year.clamp(YEAR_MIN, YEAR_MAX),
            month,
            day,
            hour: 0,
            minute: 0,
            offset_min: 0,
            cursor: 0,
            editing: false,
        }
    }

    pub fn secs(&self) -> i64 {
        let local = days_from_civil(self.year, self.month, self.day) * DAY
            + self.hour * 3600
            + self.minute * 60;
        local - self.offset_min * 60
    }

    pub fn offset_min(&self) -> i64 {
        self.offset_min
    }

    pub fn month(&self) -> i64 {
        self.month
    }

    pub fn day(&self) -> i64 {
        self.day
    }

    pub fn cursor(&self) -> Field {
        Field::ALL[self.cursor]
    }

    pub fn field(&mut self, field: Field) {
        self.cursor = Field::ALL.iter().position(|f| *f == field).unwrap_or(0);
    }

    pub fn editing(&self) -> bool { self.editing }

    pub fn toggle_editing(&mut self) { self.editing = !self.editing; }

    pub fn navigate(&mut self, dx: i32, dy: i32) {
        if self.editing {
            self.step(if dx != 0 { i64::from(dx) } else { -i64::from(dy) });
        } else {
            let col = (self.cursor as i32 % 3 + dx).clamp(0, 2);
            let row = (self.cursor as i32 / 3 + dy).clamp(0, 1);
            self.cursor = (row * 3 + col) as usize;
        }
    }

    pub fn left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub fn right(&mut self) {
        self.cursor = (self.cursor + 1).min(Field::ALL.len() - 1);
    }

    pub fn up(&mut self) {
        self.step(1);
    }

    pub fn down(&mut self) {
        self.step(-1);
    }

    fn step(&mut self, by: i64) {
        match self.cursor() {
            Field::Year => self.year = wrap(self.year + by, YEAR_MIN, YEAR_MAX),
            Field::Month => self.month = wrap(self.month + by, 1, 12),
            Field::Day => self.day = wrap(self.day + by, 1, days_in_month(self.year, self.month)),
            Field::Hour => self.hour = wrap(self.hour + by, 0, 23),
            Field::Minute => self.minute = wrap(self.minute + by, 0, 59),
            Field::Offset => {
                self.offset_min = (self.offset_min + by * OFFSET_STEP)
                    .clamp(i64::from(UTC_OFFSET_MIN), i64::from(UTC_OFFSET_MAX))
            }
        }
        self.day = self.day.min(days_in_month(self.year, self.month));
    }

    pub fn text(&self) -> String {
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02} {}",
            self.year,
            self.month,
            self.day,
            self.hour,
            self.minute,
            self.offset_text()
        )
    }

    pub fn face(&self) -> UndoFace {
        self.face_on(Canvas::FRAMED)
    }
    pub fn field_rect(&self, canvas: Canvas) -> [f32; 4] {
        let w = (canvas.w - 112.0 - 40.0) / 3.0;
        [
            56.0 + (self.cursor % 3) as f32 * (w + 20.0),
            104.0 + (self.cursor / 3) as f32 * 112.0,
            w,
            88.0,
        ]
    }
    pub fn face_on(&self, canvas: Canvas) -> UndoFace {
        let (w, h) = (canvas.w as u32, OUT_H);
        let mut rgba = vec![0; (w * h * 4) as usize];
        let mut put = |f: UndoFace, x: u32, y: u32| blit(&mut rgba, w, &f.rgba, f.w, f.h, x, y);
        let heading = crate::quick_heading_face("Date & time");
        put(heading, crate::QUICK_LABEL_X as u32 - crate::MENU_PAD, 40);
        let cell_w = (canvas.w - 152.0) / 3.0;
        for (i, (label, value)) in [
            ("Year", format!("{:04}", self.year)),
            ("Month", ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"][(self.month - 1) as usize].to_string()),
            ("Day", format!("{:02}", self.day)),
            ("Hour", format!("{:02}", self.hour)),
            ("Minute", format!("{:02}", self.minute)),
            ("UTC offset", self.offset_text()),
        ]
        .into_iter()
        .enumerate()
        {
            let x = 56.0 + (i % 3) as f32 * (cell_w + 20.0);
            let y = 104.0 + (i / 3) as f32 * 112.0;
            for (text, role, px, dy, hh, ink) in [
                (
                    label.to_string(),
                    text::Role::Detail,
                    16.0,
                    8,
                    24,
                    [154, 154, 164],
                ),
                (value, text::Role::Value, 30.0, 36, 40, INK),
            ] {
                let mut f = UndoFace {
                    rgba: vec![0; (cell_w as u32 * hh * 4) as usize],
                    w: cell_w as u32,
                    h: hh,
                };
                if let Some(font) = text::font(role) {
                    let layout = text::fit_preserving_case(
                        font,
                        &text,
                        cell_w - 24.0,
                        1,
                        px,
                        18.0_f32.min(px),
                    );
                    text::draw_centred(&mut f.rgba, f.w, f.h, &layout, ink);
                }
                put(f, x as u32, y as u32 + dy);
            }
        }

        UndoFace { rgba, w, h }
    }

    fn offset_text(&self) -> String {
        let sign = if self.offset_min < 0 { '-' } else { '+' };
        let mins = self.offset_min.abs();
        format!("{sign}{:02}:{:02}", mins / 60, mins % 60)
    }

    pub fn draw(
        &self,
        canvas: Canvas,
        line: Option<TexId>,
        hint: Option<TexId>,
        back: Option<(TexId, u32)>,
        out: &mut Vec<Draw>,
    ) {
        self.draw_animated(canvas, line, hint, back, self.field_rect(canvas), out);
    }
    pub fn draw_animated(
        &self,
        canvas: Canvas,
        line: Option<TexId>,
        hint: Option<TexId>,
        back: Option<(TexId, u32)>,
        focus: [f32; 4],
        out: &mut Vec<Draw>,
    ) {
        out.push(Draw::Rect {
            x: 0.0,
            y: 0.0,
            w: canvas.w,
            h: OUT_H as f32,
            colour: BACKDROP,
        });
        crate::expressive::rounded(
            out,
            focus[0],
            focus[1],
            focus[2],
            focus[3],
            12.0,
            if self.editing { [0.24, 0.42, 0.43, 1.0] } else { crate::expressive::SELECTED },
        );
        if let Some(tex) = line {
            out.push(Draw::Tex {
                x: 0.0,
                y: 0.0,
                w: canvas.w,
                h: OUT_H as f32,
                tex,
                alpha: 1.0,
            });
        }
        if let Some((tex, w)) = back {
            out.push(Draw::Tex {
                x: 16.0,
                y: 378.0,
                w: w as f32,
                h: crate::MENU_HINT_H as f32,
                tex,
                alpha: 1.0,
            });
        }
        if let Some(tex) = hint {
            let w = *CLOCK_HINT_WIDTH.get_or_init(|| set_clock_hint_face().w);
            out.push(Draw::Tex {
                x: canvas.w - 16.0 - w as f32 + crate::HINT_EDGE as f32,
                y: 378.0,
                w: w as f32,
                h: crate::MENU_HINT_H as f32,
                tex,
                alpha: 1.0,
            });
        }
    }
}

static CLOCK_HINT_WIDTH: std::sync::OnceLock<u32> = std::sync::OnceLock::new();
pub fn set_clock_hint_face() -> UndoFace {
    crate::quick_menu::clock_legend()
}

fn wrap(v: i64, lo: i64, hi: i64) -> i64 {
    (v - lo).rem_euclid(hi - lo + 1) + lo
}
