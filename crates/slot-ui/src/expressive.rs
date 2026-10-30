//! Small, interruptible spatial springs and tonal surfaces. Everything here
//! changes existing quads; text is still rasterised only when its content changes.
use crate::{quick_row_top_at, Draw, QuickRow};

pub const ACCENT: [f32; 4] = [0.33, 0.76, 0.72, 1.0];
pub const SURFACE: [f32; 4] = [0.065, 0.078, 0.095, 0.97];
pub const CONTAINER: [f32; 4] = [0.12, 0.14, 0.17, 1.0];
pub const SELECTED: [f32; 4] = [0.20, 0.30, 0.32, 1.0];

pub fn rounded(out: &mut Vec<Draw>, x: f32, y: f32, w: f32, h: f32, radius: f32, colour: [f32; 4]) {
    if w > 0.0 && h > 0.0 && colour[3] > 0.0 {
        out.push(Draw::RoundRect {
            x,
            y,
            w,
            h,
            radius,
            colour,
        });
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct Spring {
    value: f32,
    velocity: f32,
}
impl Spring {
    fn at(value: f32) -> Self {
        Self {
            value,
            velocity: 0.0,
        }
    }
    fn step(&mut self, target: f32, dt: f32, stiffness: f32, damping: f32) {
        if !dt.is_finite() || dt <= 0.0 {
            return;
        }
        let dt = dt.min(1.0);
        let x = self.value - target;
        if x == 0.0 && self.velocity == 0.0 {
            return;
        }
        let omega = stiffness.sqrt();
        let (x, velocity) = if damping >= 1.0 {
            let b = self.velocity + omega * x;
            let decay = (-omega * dt).exp();
            (
                (x + b * dt) * decay,
                (self.velocity - omega * b * dt) * decay,
            )
        } else {
            let decay_rate = damping * omega;
            let frequency = omega * (1.0 - damping * damping).sqrt();
            let (sin, cos) = (frequency * dt).sin_cos();
            let decay = (-decay_rate * dt).exp();
            (
                (x * cos + (self.velocity + decay_rate * x) / frequency * sin) * decay,
                (self.velocity * cos
                    - (decay_rate * self.velocity + stiffness * x) / frequency * sin)
                    * decay,
            )
        };
        if x.abs() < 0.01 && velocity.abs() < 0.05 {
            *self = Self::at(target);
        } else {
            self.value = target + x;
            self.velocity = velocity;
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MenuFrame {
    pub focus_y: f32,
    pub entry: f32,
    pub opacity: f32,
    pub switch: Option<f32>,
    pub closing: bool,
}
impl MenuFrame {
    pub fn settled(row: QuickRow, hidden: &[QuickRow]) -> Self {
        Self {
            focus_y: quick_row_top_at(row, row, hidden),
            entry: 1.0,
            opacity: 1.0,
            switch: None,
            closing: false,
        }
    }
}

#[derive(Default, Debug)]
pub struct MenuMotion {
    active: bool,
    page: Option<QuickRow>,
    row: Option<QuickRow>,
    focus: Spring,
    entry: Spring,
    opacity: Spring,
    switch: Spring,
}
impl MenuMotion {
    pub fn update(
        &mut self,
        row: Option<QuickRow>,
        hidden: &[QuickRow],
        on: Option<bool>,
        dt: f32,
    ) {
        let Some(row) = row else {
            self.active = false;
            self.entry.step(0.0, dt, 800.0, 1.0);
            self.opacity.step(0.0, dt, 1600.0, 1.0);
            if self.opacity.value < 0.004 {
                self.page = None;
                self.row = None;
            }
            return;
        };
        self.active = true;
        let target = quick_row_top_at(row, row, hidden);
        if self.page != Some(row.page()[0]) {
            self.page = Some(row.page()[0]);
            self.focus = Spring::at(target);
            self.entry = Spring::default();
            self.opacity = Spring::default();
        }
        if self.row != Some(row) {
            self.switch = Spring::at(if on == Some(true) { 1.0 } else { 0.0 });
        }
        self.row = Some(row);
        // Expressive fast spatial for focus and switches, default spatial for
        // page arrival. Critically damp opacity: it must never bounce.
        self.focus.step(target, dt, 800.0, 0.76);
        self.entry.step(1.0, dt, 380.0, 0.82);
        self.opacity.step(1.0, dt, 1600.0, 1.0);
        if let Some(on) = on {
            self.switch
                .step(if on { 1.0 } else { 0.0 }, dt, 800.0, 0.76);
        }
    }
    pub fn closing_row(&self) -> Option<QuickRow> {
        self.row
            .filter(|_| !self.active && self.opacity.value > 0.004)
    }
    pub fn frame(&self, row: QuickRow, hidden: &[QuickRow]) -> MenuFrame {
        if self.page != Some(row.page()[0]) {
            return MenuFrame::settled(row, hidden);
        }
        MenuFrame {
            focus_y: self.focus.value,
            entry: self.entry.value,
            opacity: self.opacity.value.clamp(0.0, 1.0),
            switch: (self.row == Some(row)).then_some(self.switch.value.clamp(0.0, 1.0)),
            closing: !self.active,
        }
    }
}

/// Cached menu pages (Bluetooth and parental controls) use the same shapes.
/// Called only while rebuilding their existing text texture, never per frame.
pub fn raster_menu_rows(
    face: &mut crate::UndoFace,
    top: f32,
    pitch: f32,
    count: usize,
    selected: Option<usize>,
) {
    raster_round(
        face,
        24.0,
        top - 8.0,
        face.w as f32 - 48.0,
        count as f32 * pitch + 16.0,
        24.0,
        SURFACE,
    );
    for i in 0..count {
        let active = selected == Some(i);
        raster_round(
            face,
            32.0,
            top + i as f32 * pitch,
            face.w as f32 - 64.0,
            pitch - 2.0,
            if active { 20.0 } else { 10.0 },
            if active { SELECTED } else { CONTAINER },
        );
        if active {
            raster_round(
                face,
                40.0,
                top + i as f32 * pitch + 13.0,
                3.0,
                14.0,
                1.5,
                ACCENT,
            );
        }
    }
}
fn raster_round(
    face: &mut crate::UndoFace,
    left: f32,
    top: f32,
    w: f32,
    h: f32,
    r: f32,
    ink: [f32; 4],
) {
    for y in top.max(0.0) as u32..((top + h).ceil() as u32).min(face.h) {
        for x in left.max(0.0) as u32..((left + w).ceil() as u32).min(face.w) {
            let qx = ((x as f32 + 0.5 - left) - w / 2.0).abs() - w / 2.0 + r;
            let qy = ((y as f32 + 0.5 - top) - h / 2.0).abs() - h / 2.0 + r;
            let d = (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt() + qx.max(qy).min(0.0) - r;
            let a = (0.5 - d).clamp(0.0, 1.0) * ink[3];
            if a == 0.0 {
                continue;
            }
            let at = ((y * face.w + x) * 4) as usize;
            let old = face.rgba[at + 3] as f32 / 255.0;
            let alpha = a + old * (1.0 - a);
            for c in 0..3 {
                face.rgba[at + c] =
                    ((ink[c] * a + face.rgba[at + c] as f32 / 255.0 * old * (1.0 - a)) / alpha
                        * 255.0)
                        .round() as u8;
            }
            face.rgba[at + 3] = (alpha * 255.0).round() as u8;
        }
    }
}

#[derive(Default, Debug)]
pub struct StatusMotion {
    key: Option<u8>,
    scale: Spring,
    opacity: Spring,
}
impl StatusMotion {
    pub fn update(&mut self, key: u8, dt: f32) {
        if self.key != Some(key) {
            let initial = self.key.is_none();
            self.key = Some(key);
            self.scale = Spring::at(if initial { 1.0 } else { 0.0 });
            self.opacity = Spring::at(if initial { 1.0 } else { 0.0 });
        }
        self.scale.step(1.0, dt, 800.0, 0.8);
        self.opacity.step(1.0, dt, 1600.0, 1.0);
    }
    pub fn apply(&self, draws: &mut [Draw]) {
        let scale = 0.8 + 0.2 * self.scale.value;
        for d in draws {
            if let Draw::Tex {
                x, y, w, h, alpha, ..
            } = d
            {
                *x += *w * (1.0 - scale) / 2.0;
                *y += *h * (1.0 - scale) / 2.0;
                *w *= scale;
                *h *= scale;
                *alpha *= self.opacity.value.clamp(0.0, 1.0);
            }
        }
    }
}
