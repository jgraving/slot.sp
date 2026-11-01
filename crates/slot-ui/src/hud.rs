use slot_gfx::{Canvas, Draw, TexId, OUT_H};
use slot_store::{BLUE_LIGHT_MAX, BRIGHTNESS_MAX, VOLUME_MAX};

use crate::icon::icon_box;
use crate::toast::toast_rect;
use crate::{Badge, Icon, Toast};

pub type Millis = u64;

pub const HUD_MS: Millis = 1500;
const FADE_MS: Millis = 250;
const PULSE_MS: Millis = 500;

pub const PLATE_H: f32 = 40.0;
pub(crate) const PLATE: [f32; 4] = [0.0, 0.0, 0.0, 0.72];
pub const ALARM_BAND_H: f32 = 120.0;
const ALARM_BRIGHT: [f32; 4] = [0.82, 0.10, 0.10, 0.94];
const ALARM_DIM: [f32; 4] = [0.40, 0.03, 0.03, 0.94];

pub const HUD_ICON_PX: f32 = 24.0;
pub const HUD_INK: [u8; 3] = [0xf5, 0xf2, 0xef];

pub const FOOTER_INK: [u8; 3] = [0xff, 0xff, 0xff];
pub const FOOTER_INK_LIGHT: [u8; 3] = [0x00, 0x00, 0x00];

pub fn footer_ink(light: bool) -> [u8; 3] {
    if light {
        FOOTER_INK_LIGHT
    } else {
        FOOTER_INK
    }
}

const BADGE_MARGIN: f32 = 12.0;

pub fn badge_at(canvas: Canvas, w: f32, h: f32) -> (f32, f32) {
    (canvas.w - BADGE_MARGIN - w, (PLATE_H - h) / 2.0)
}

const BAR_W: f32 = 280.0;
pub(crate) const TRACK: [f32; 4] = [1.0, 1.0, 1.0, 0.18];
pub(crate) const FILL: [f32; 4] = [
    HUD_INK[0] as f32 / 255.0,
    HUD_INK[1] as f32 / 255.0,
    HUD_INK[2] as f32 / 255.0,
    1.0,
];

#[derive(Copy, Clone, Default, PartialEq, Eq, Debug)]
pub enum HudKind {
    #[default]
    Brightness,
    BlueLight,
    Volume,
    Rewind,
}

impl HudKind {
    pub fn icon(self, value: u8, muted: bool, headphones: bool) -> Icon {
        match self {
            HudKind::Brightness => Icon::Brightness,
            HudKind::BlueLight => Icon::BlueLight,
            HudKind::Volume if muted || value == 0 => Icon::VolumeMuted,
            HudKind::Volume if headphones => Icon::Headphones,
            HudKind::Volume => Icon::Volume,
            HudKind::Rewind => Icon::Rewind,
        }
    }

    fn max(self) -> u8 {
        match self {
            HudKind::Brightness => BRIGHTNESS_MAX,
            HudKind::BlueLight => BLUE_LIGHT_MAX,
            HudKind::Volume => VOLUME_MAX,
            HudKind::Rewind => 100,
        }
    }
}

#[derive(Copy, Clone, Default, PartialEq, Eq, Debug)]
pub enum FfState {
    #[default]
    Off,
    Held,
    Latched,
}

pub fn ff_badge(state: FfState) -> Option<Icon> {
    match state {
        FfState::Off => None,
        FfState::Held => Some(Icon::FastForward),
        FfState::Latched => Some(Icon::FastForwardLatched),
    }
}

pub const LINK_HOST_INK: [u8; 3] = [0x8a, 0x74, 0xcf];
pub const LINK_JOIN_INK: [u8; 3] = [0xb2, 0xb2, 0xb8];

#[derive(Copy, Clone, Default, PartialEq, Eq, Debug)]
pub enum LinkBadge {
    #[default]
    Off,
    Hosting,
    Joined,
    HostingLost,
    JoinedLost,
}

impl LinkBadge {
    pub const FACES: [LinkBadge; 4] = [
        LinkBadge::Hosting,
        LinkBadge::Joined,
        LinkBadge::HostingLost,
        LinkBadge::JoinedLost,
    ];

    pub fn face_index(self) -> Option<usize> {
        LinkBadge::FACES.iter().position(|b| *b == self)
    }

    pub fn badge(self) -> Option<Badge> {
        match self {
            LinkBadge::Off => None,
            LinkBadge::Hosting | LinkBadge::Joined => Some(Badge::Link),
            LinkBadge::HostingLost | LinkBadge::JoinedLost => Some(Badge::LinkBroken),
        }
    }

    pub fn colour(self) -> Option<[u8; 3]> {
        match self {
            LinkBadge::Off => None,
            LinkBadge::Hosting | LinkBadge::HostingLost => Some(LINK_HOST_INK),
            LinkBadge::Joined | LinkBadge::JoinedLost => Some(LINK_JOIN_INK),
        }
    }
}

#[derive(Default)]
pub struct Hud {
    pub kind: HudKind,
    pub value: u8,
    pub shown_at: Option<Millis>,
    entered_at: Option<Millis>,
    value_at: Millis,
    from_offset: f32,
    offset_velocity: f32,
    from_alpha: f32,
    from_fraction: f32,
    from_velocity: f32,
    held: bool,
    muted: bool,
    headphones: bool,
    bluetooth: bool,
    usb: bool,
    ff: FfState,
    said: Option<(Toast, Millis)>,
    alarm: Option<Millis>,
    alarm_toast: Option<Toast>,
    icons: Vec<TexId>,
    toasts: Vec<TexId>,
    link: LinkBadge,
    link_faces: Vec<TexId>,
}

impl Hud {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_icons(&mut self, icons: Vec<TexId>) {
        self.icons = icons;
    }

    pub fn set_toasts(&mut self, toasts: Vec<TexId>) {
        self.toasts = toasts;
    }

    pub fn toast(&mut self, toast: Toast, now: Millis) {
        self.said = Some((toast, now));
    }

    pub fn hush(&mut self) {
        self.said = None;
    }

    pub fn set_alarm(&mut self, on: bool, now: Millis) {
        self.alarm = match (on, self.alarm) {
            (true, Some(since)) => Some(since),
            (true, None) => Some(now),
            (false, _) => None,
        };
    }

    pub fn set_alarm_toast(&mut self, toast: Toast) {
        self.alarm_toast = Some(toast);
    }

    pub fn alarm_toast(&self) -> Option<Toast> {
        self.alarm
            .map(|_| self.alarm_toast.unwrap_or(Toast::CardUnwritable))
    }

    pub fn alarmed(&self) -> bool {
        self.alarm.is_some()
    }

    pub fn toast_visible(&self, now: Millis) -> bool {
        self.toast_alpha(now) > 0.0
    }

    pub fn said(&self, now: Millis) -> Option<Toast> {
        self.said.filter(|(t, shown)| now.saturating_sub(*shown) < t.hold_ms()).map(|(t, _)| t)
    }

    pub fn show(&mut self, kind: HudKind, value: u8, muted: bool, now: Millis) {
        let active = self.visible(now);
        let previous = self.visual_state(now);
        let position = self.slide_state(now);
        self.from_alpha = if active { self.alpha(now) } else { 1.0 };
        self.from_offset = if active { position.0 } else { -60.0 };
        self.offset_velocity = if active { position.1 } else { 0.0 };
        self.entered_at = Some(now);
        let target = value.min(kind.max()) as f32 / kind.max().max(1) as f32;
        self.from_fraction = if active { previous.0 } else { target };
        self.from_velocity = if active { previous.1 } else { 0.0 };
        self.value_at = now;
        self.kind = kind;
        self.value = value;
        self.muted = muted;
        self.shown_at = Some(now);
        self.held = kind == HudKind::Rewind;
    }

    pub fn release_rewind(&mut self) {
        if self.kind == HudKind::Rewind {
            self.shown_at = None;
        }
        self.held = false;
    }

    pub fn glyph(&self) -> Icon {
        if self.kind == HudKind::Volume && (self.muted || self.value == 0) {
            return Icon::VolumeMuted;
        }
        if self.kind == HudKind::Volume && self.usb {
            return Icon::Headphones;
        }
        if self.kind == HudKind::Volume && self.bluetooth {
            return Icon::Bluetooth;
        }
        self.kind.icon(self.value, self.muted, self.headphones)
    }

    pub fn set_usb(&mut self, on: bool) {
        self.usb = on;
    }

    pub fn set_bluetooth(&mut self, on: bool) {
        self.bluetooth = on;
    }

    pub fn set_headphones(&mut self, on: bool) {
        self.headphones = on;
    }

    pub fn badge(&self) -> Option<Icon> {
        ff_badge(self.ff)
    }

    pub fn set_ff(&mut self, ff: FfState) {
        self.ff = ff;
    }

    pub fn set_link(&mut self, badge: LinkBadge) {
        self.link = badge;
    }

    pub fn link(&self) -> LinkBadge {
        self.link
    }

    pub fn set_link_faces(&mut self, faces: Vec<TexId>) {
        self.link_faces = faces;
    }

    pub fn visible(&self, now: Millis) -> bool {
        self.alpha(now) > 0.0
    }

    pub fn draw(&self,canvas:Canvas,now:Millis,out:&mut Vec<Draw>){self.draw_with_bottom_inset(canvas,now,0.0,out);}

    pub fn draw_with_bottom_inset(&self, canvas: Canvas, now: Millis, inset:f32, out: &mut Vec<Draw>) {
        let alpha = self.alpha(now);
        let toast = self.toast_alpha(now);
        let audio_notice = toast > 0.0 && self.said.is_some_and(|(t, _)| t.audio_change());
        if toast > 0.0 {
            let begin=out.len();self.draw_toast(canvas,now,out);
            if self.said.is_some_and(|(t,_)|!t.alarm()) {
                for draw in &mut out[begin..] {match draw {Draw::Rect{y,..}|Draw::RoundRect{y,..}|Draw::Tex{y,..}|Draw::Part{y,..}=>*y-=inset,_=>{}}}
            }
        }
        if alpha > 0.0 && (toast == 0.0 || audio_notice) {
            self.draw_bar(
                canvas,
                1.0,
                0.0,
                now,
                out,
            );
        }
        let link = self
            .link
            .face_index()
            .and_then(|i| self.link_faces.get(i).copied());
        let ff = ff_badge(self.ff).and_then(|i| self.icons.get(i.index()).copied());
        if let Some(tex) = link.or(ff) {
            self.place_badge(canvas, tex, out);
        }
        if let Some(since) = self.alarm {
            self.draw_alarm(canvas, now.saturating_sub(since), out);
        }
    }

    fn draw_bar(&self, canvas: Canvas, alpha: f32, offset: f32, now: Millis, out: &mut Vec<Draw>) {
        let y = 8.0 + offset + self.slide_state(now).0;
        let x = canvas.centred(360.0);
        crate::expressive::rounded(
            out,
            x,
            y,
            360.0,
            48.0,
            24.0,
            [0.07, 0.085, 0.10, 0.97 * alpha],
        );
        if let Some(tex) = self.icon() {
            let (w, h) = match self.glyph() {Icon::Volume|Icon::VolumeZero|Icon::VolumeMuted|Icon::Brightness|Icon::BlueLight|Icon::Bluetooth|Icon::BluetoothMuted|Icon::Headphones|Icon::HeadphonesMuted=>(24,24),icon=>crate::icon::glyph_box(icon,HUD_ICON_PX)};
            out.push(Draw::Tex {
                x: x + 16.0,
                y: y + (48.0 - h as f32) / 2.0,
                w: w as f32,
                h: h as f32,
                tex,
                alpha,
            });
        }
        crate::draw_slider(
            out,
            x + 58.0,
            y + 9.0,
            BAR_W,
            self.visual_state(now).0,
            alpha,
        );
    }
    fn slide_state(&self, now: Millis) -> (f32, f32) {
        let t = now.saturating_sub(self.entered_at.unwrap_or(now)) as f32 / 1000.0;
        let omega = 30.0;
        let b = self.offset_velocity + omega * self.from_offset;
        let decay = (-omega * t).exp();
        let base = (self.from_offset + b * t) * decay;
        let velocity = (self.offset_velocity - omega * b * t) * decay;
        let progress = if self.held {
            0.0
        } else {
            1.0 - self.shown_at.map_or(0.0, |shown| fade(shown, now))
        };
        let travel = -60.0 * progress * progress * (3.0 - 2.0 * progress);
        let speed = if progress > 0.0 && progress < 1.0 {
            -60.0 * 6.0 * progress * (1.0 - progress) * 1000.0 / FADE_MS as f32
        } else {
            0.0
        };
        (base + travel, velocity + speed)
    }
    fn visual_state(&self, now: Millis) -> (f32, f32) {
        let target = self.fraction();
        let t = now.saturating_sub(self.value_at) as f32 / 1000.0;
        let omega = 40.0;
        let x = self.from_fraction - target;
        let b = self.from_velocity + omega * x;
        let decay = (-omega * t).exp();
        (
            (target + (x + b * t) * decay).clamp(0.0, 1.0),
            (self.from_velocity - omega * b * t) * decay,
        )
    }

    fn place_badge(&self, canvas: Canvas, tex: TexId, out: &mut Vec<Draw>) {
        let (w, h) = icon_box(HUD_ICON_PX);
        let (w, h) = (w as f32, h as f32);
        let (x, y) = badge_at(canvas, w, h);
        out.push(Draw::Tex {
            x,
            y,
            w,
            h,
            tex,
            alpha: 1.0,
        });
    }

    fn draw_alarm(&self, canvas: Canvas, age: Millis, out: &mut Vec<Draw>) {
        out.push(Draw::Rect {
            x: 0.0,
            y: (OUT_H as f32 - ALARM_BAND_H) / 2.0,
            w: canvas.w,
            h: ALARM_BAND_H,
            colour: pulse(age),
        });
        let alarm = self.alarm_toast.unwrap_or(Toast::CardUnwritable);
        let Some(tex) = self.toasts.get(alarm.index()).copied() else {
            return;
        };
        let (x, y, w, h) = toast_rect(canvas, alarm);
        out.push(Draw::Tex {
            x,
            y,
            w,
            h,
            tex,
            alpha: 1.0,
        });
    }

    fn draw_toast(&self, canvas: Canvas, now: Millis, out: &mut Vec<Draw>) {
        let alpha = self.toast_alpha(now);
        if alpha <= 0.0 {
            return;
        }
        let Some(tex) = self
            .said
            .and_then(|(t, _)| self.toasts.get(t.index()))
            .copied()
        else {
            return;
        };
        let Some((said, _)) = self.said else {
            return;
        };
        let (x, base, w, h) = toast_rect(canvas, said);
        let y=base+(1.0-alpha)*(canvas.h-base+h+32.0);
        let alpha=1.0;
        crate::expressive::rounded(out, x - 16.0, y - 10.0, w + 32.0, h + 20.0,
            (h + 20.0) / 2.0, [0.10, 0.12, 0.14, 0.97 * alpha]);
        out.push(Draw::Tex {
            x,
            y,
            w,
            h,
            tex,
            alpha,
        });
    }

    fn icon(&self) -> Option<TexId> {
        self.icons.get(self.glyph().index()).copied()
    }

    fn fraction(&self) -> f32 {
        let max = self.kind.max();
        if max == 0 {
            return 0.0;
        }
        self.value.min(max) as f32 / max as f32
    }

    fn alpha(&self, now: Millis) -> f32 {
        let Some(shown) = self.shown_at else {
            return 0.0;
        };
        if self.held {
            return 1.0;
        }
        let t = now.saturating_sub(shown) as f32 / 1000.0;
        let arrival = 1.0 - (1.0 - self.from_alpha) * (1.0 + 40.0 * t) * (-40.0 * t).exp();
        fade(shown, now) * arrival
    }

    fn toast_alpha(&self, now: Millis) -> f32 {
        match self.said {
            Some((toast, shown)) => {
                let arrival = (now.saturating_sub(shown) as f32 / 140.0).clamp(0.0, 1.0);
                fade_over(shown, now, toast.hold_ms()) * arrival * arrival * (3.0 - 2.0 * arrival)
            },
            None => 0.0,
        }
    }
}

fn fade(shown: Millis, now: Millis) -> f32 {
    fade_over(shown, now, HUD_MS)
}

fn fade_over(shown: Millis, now: Millis, hold: Millis) -> f32 {
    let age = now.saturating_sub(shown);
    if age >= hold {
        return 0.0;
    }
    ((hold - age) as f32 / FADE_MS as f32).min(1.0)
}

fn pulse(age: Millis) -> [f32; 4] {
    let phase = (age % PULSE_MS) as f32 / PULSE_MS as f32;
    let t = 0.5 + 0.5 * (phase * std::f32::consts::TAU).cos();
    std::array::from_fn(|i| ALARM_DIM[i] + (ALARM_BRIGHT[i] - ALARM_DIM[i]) * t)
}
