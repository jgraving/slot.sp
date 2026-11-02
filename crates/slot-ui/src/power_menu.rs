use crate::plate::UndoFace;
use crate::text;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PowerChoice {
    Restart,
    PowerOff,
    Repair,
}

impl PowerChoice {
    pub const ALL: [PowerChoice; 3] = [
        PowerChoice::Restart,
        PowerChoice::PowerOff,
        PowerChoice::Repair,
    ];

    pub fn index(self) -> usize {
        self as usize
    }
}

pub(crate) const MENU_PX: f32 = 30.0;
const MENU_MIN_PX: f32 = 18.0;
pub(crate) const MENU_H: u32 = 40;
pub const MENU_PAD: u32 = 18;
pub(crate) const MENU_INK: [u8; 3] = [0xf6, 0xf4, 0xef];

pub fn menu_face(label: &str) -> UndoFace {
    let Some(font) = text::font(text::Role::Display) else {
        return UndoFace {
            rgba: Vec::new(),
            w: 0,
            h: 0,
        };
    };
    let ink = text::line_width(font, label, MENU_PX, 0.0).ceil() as u32;
    let w = ink + 2 * MENU_PAD;
    let mut rgba = vec![0u8; (w * MENU_H * 4) as usize];
    let layout = text::fit_preserving_case(font, label, w as f32, 1, MENU_PX, MENU_MIN_PX);
    text::draw_centred(&mut rgba, w, MENU_H, &layout, MENU_INK);
    UndoFace { rgba, w, h: MENU_H }
}

/// A short physical ON-to-OFF stroke, followed by a fade to black.
pub const POWER_SWITCH_MS: u64 = 600;
/// Time at which the slider reaches OFF (including the initial ON hold).
pub const POWER_SWITCH_SLIDE_MS: u64 = 330;

/// Compatibility entry point for the automatic dark/light shell selection.
pub fn power_switch_faces(light: bool) -> [UndoFace; 2] {
    power_switch_faces_for(if light {
        slot_store::ShellColour::Platinum
    } else {
        slot_store::ShellColour::Graphite
    })
}

/// The switch shares the carousel's extracted paint grain and shell palette.
pub fn power_switch_faces_for(colour: slot_store::ShellColour) -> [UndoFace; 2] {
    let mut rgba = crate::art::render_svg(include_str!("../assets/power_switch.svg"), 1000, 400)
        .expect("embedded power switch housing");
    let base = crate::material::shell_rgb(colour);
    for y in 0..400usize {
        for x in 0..1000usize {
            let i = (y * 1000 + x) * 4;
            let r = rgba[i] as f32;
            if r > rgba[i + 1] as f32 * 1.5 && rgba[i + 3] != 0 {
                let relief = (r - 174.0) * 0.68;
                let grain = crate::material::paint_grain(x, y);
                for c in 0..3 {
                    rgba[i + c] = (base[c] as f32 + grain + relief).clamp(0.0, 255.0) as u8;
                }
            } else if rgba[i] > rgba[i + 1] {
                let neutral = rgba[i + 1].max(rgba[i + 2]);
                rgba[i..i + 3].fill(neutral);
            }
        }
    }
    [
        UndoFace {
            rgba,
            w: 1000,
            h: 400,
        },
        UndoFace {
            rgba: crate::art::render_svg(
                include_str!("../assets/power_switch_slider.svg"),
                180,
                78,
            )
            .expect("embedded power switch slider"),
            w: 180,
            h: 78,
        },
    ]
}

/// Draw on the current canvas, including narrow and framed panels.
pub fn draw_power_switch(
    canvas: crate::Canvas,
    progress: f32,
    faces: [crate::TexId; 2],
    out: &mut Vec<crate::Draw>,
) {
    let p = progress.clamp(0.0, 1.0);
    // 150 ms ON, 180 ms snap, 170 ms OFF, 100 ms fade; total remains 600 ms.
    let elapsed = p * POWER_SWITCH_MS as f32;
    let t = ((elapsed - 150.0) / 180.0).clamp(0.0, 1.0);
    let off = t * t * (3.0 - 2.0 * t);
    let alpha = ((POWER_SWITCH_MS as f32 - elapsed) / 100.0).clamp(0.0, 1.0);
    let scale = canvas.w / 300.0;
    let w = 1000.0 * scale;
    let h = 400.0 * scale;
    let x = canvas.centred(w);
    let y = (crate::OUT_H as f32 - h) * 0.5;
    out.push(crate::Draw::Tex {
        x,
        y,
        w,
        h,
        tex: faces[0],
        alpha,
    });
    out.push(crate::Draw::Tex {
        x: x + (432.0 + 48.0 * (1.0 - off)) * scale,
        y: y + 189.0 * scale,
        w: 90.0 * scale,
        h: 39.0 * scale,
        tex: faces[1],
        alpha,
    });
}

const DETAIL_PX: f32 = 20.0;
const DETAIL_H: u32 = 28;

pub fn menu_detail_face(label: &str) -> UndoFace {
    let Some(font) = text::font(text::Role::Detail) else {
        return UndoFace {
            rgba: Vec::new(),
            w: 0,
            h: 0,
        };
    };
    let shown = label.to_string();
    let ink =
        text::line_width(font, &shown, DETAIL_PX, text::tracking_for(DETAIL_PX)).ceil() as u32;
    let w = ink + 2 * MENU_PAD;
    let mut rgba = vec![0u8; (w * DETAIL_H * 4) as usize];
    let layout = text::fit_preserving_case(font, label, w as f32, 1, DETAIL_PX, DETAIL_PX);
    text::draw_centred(&mut rgba, w, DETAIL_H, &layout, MENU_INK);
    UndoFace {
        rgba,
        w,
        h: DETAIL_H,
    }
}
