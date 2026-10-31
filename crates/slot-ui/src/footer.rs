use slot_gfx::{Canvas, Draw, TexId, OUT_H};
use slot_power::Battery;

use crate::battery::{draw_gauge, GAUGE_H};
use crate::hud::{HUD_INK, PLATE_H};
use crate::plate::HINT_H;
use crate::slot_chrome::MOUTH_H;

pub const FOOTER_Y: f32 = OUT_H as f32 - MOUTH_H + (MOUTH_H - HINT_H as f32) / 2.0;
const FOOTER_MARGIN: f32 = 24.0;
const CORNER_MARGIN: f32 = 16.0;
const STATUS_STEP:f32=34.0;

#[derive(Copy, Clone, Default, PartialEq, Eq, Debug)]
pub struct Printed {
    pub face: Option<TexId>,
    pub w: u32,
}

impl Printed {
    pub fn new(face: TexId, w: u32) -> Self {
        Printed {
            face: Some(face),
            w,
        }
    }
}

pub fn draw_footer(
    canvas: Canvas,
    battery: Option<Battery>,
    percent: Printed,
    bolt: Option<TexId>,
    clock: Printed,
    ink: [u8; 3],
    out: &mut Vec<Draw>,
) {
    let y = FOOTER_Y + (HINT_H as f32 - GAUGE_H) / 2.0;
    draw_gauge(FOOTER_MARGIN, y, battery, percent, bolt, ink, out);
    printed(canvas.w - FOOTER_MARGIN - clock.w as f32, clock, out);
}

pub fn draw_corners(
    canvas: Canvas,
    battery: Option<Battery>,
    percent: Printed,
    bolt: Option<TexId>,
    clock: Printed,
    out: &mut Vec<Draw>,
) {
    draw_status(canvas, battery, percent, bolt, clock, HUD_INK, out);
}

/// Persistent status at the same top corners used by the in-game overlay.
pub fn draw_status(
    canvas: Canvas,
    battery: Option<Battery>,
    percent: Printed,
    bolt: Option<TexId>,
    clock: Printed,
    ink: [u8; 3],
    out: &mut Vec<Draw>,
) {
    let y = (PLATE_H - GAUGE_H) / 2.0;
    crate::battery::draw_gauge_sized(CORNER_MARGIN,y,battery,percent,bolt,ink,crate::STATUS_TEXT_H,out);
    let x = canvas.w - CORNER_MARGIN - clock.w as f32;
    draw_printed_sized(x,(PLATE_H-crate::STATUS_TEXT_H as f32)/2.0,clock,crate::STATUS_TEXT_H,out);
}

pub(crate) fn draw_printed(x:f32,y:f32,p:Printed,out:&mut Vec<Draw>) {draw_printed_sized(x,y,p,HINT_H,out);}
fn draw_printed_sized(x:f32,y:f32,p:Printed,height:u32,out:&mut Vec<Draw>) {
    if p.w == 0 {
        return;
    }
    let (w, h) = (p.w as f32, height as f32);
    out.push(match p.face {
        Some(tex) => Draw::Tex {
            x,
            y,
            w,
            h,
            tex,
            alpha: 1.0,
        },
        None => Draw::Rect {
            x,
            y,
            w,
            h,
            colour: [1.0, 1.0, 1.0, 0.08],
        },
    });
}

fn printed(x: f32, p: Printed, out: &mut Vec<Draw>) {
    draw_printed(x, FOOTER_Y, p, out);
}

/// Place a connection icon after the complete battery gauge and percentage.
pub fn draw_footer_connection(
    battery: Option<Battery>,
    percent: Printed,
    icon: Option<TexId>,
    out: &mut Vec<Draw>,
) {
    draw_connection(
        battery,
        percent,
        icon,
        FOOTER_MARGIN,
        FOOTER_Y + (HINT_H as f32 - crate::STATUS_ICON_SIZE as f32) / 2.0,
        out,
    );
}

pub fn draw_corner_connection(
    battery: Option<Battery>,
    percent: Printed,
    icon: Option<TexId>,
    out: &mut Vec<Draw>,
) {
    draw_corner_audio(battery, percent, icon, None, out);
}

/// Keep route and mute visible together, after the entire battery readout.
pub fn draw_corner_audio(
    battery: Option<Battery>,
    percent: Printed,
    connection: Option<TexId>,
    mute: Option<TexId>,
    out: &mut Vec<Draw>,
) {
    draw_connection(
        battery,
        percent,
        connection,
        CORNER_MARGIN,
        (PLATE_H - crate::STATUS_ICON_SIZE as f32) / 2.0,
        out,
    );
    draw_connection(
        battery,
        percent,
        mute,
        CORNER_MARGIN + if connection.is_some() { STATUS_STEP } else { 0.0 },
        (PLATE_H - crate::STATUS_ICON_SIZE as f32) / 2.0,
        out,
    );
}

pub fn draw_corner_kid(battery:Option<Battery>,percent:Printed,icon:TexId,preceding:usize,out:&mut Vec<Draw>) {
    draw_connection(battery,percent,Some(icon),CORNER_MARGIN+STATUS_STEP*preceding as f32,(PLATE_H-crate::STATUS_ICON_SIZE as f32)/2.0,out);
}

fn draw_connection(
    battery: Option<Battery>,
    percent: Printed,
    icon: Option<TexId>,
    margin: f32,
    y: f32,
    out: &mut Vec<Draw>,
) {
    if let Some(tex) = icon {
        let x = margin
            + if battery.is_some() {
                crate::battery::gauge_width(percent) + 10.0
            } else {
                0.0
            };
        out.push(Draw::Tex {
            x,
            y,
            w: crate::STATUS_ICON_SIZE as f32,
            h: crate::STATUS_ICON_SIZE as f32,
            tex,
            alpha: 1.0,
        });
    }
}

#[cfg(test)]
mod connection_tests {
    use super::*;
    #[test]
    fn mute_follows_connection_without_overlapping_the_battery_or_clock() {
        let connection = TexId::from_raw(2);
        let mute = TexId::from_raw(3);
        for battery in [
            None,
            Some(Battery {
                percent: 100,
                charge: slot_power::Charge::Charging,
            }),
        ] {
            for connected in [false, true] {
                let mut draws = vec![];
                let percent = Printed::new(TexId::from_raw(1), 60);
                draw_corner_audio(
                    battery,
                    percent,
                    connected.then_some(connection),
                    Some(mute),
                    &mut draws,
                );
                let xs: Vec<f32> = draws
                    .iter()
                    .map(|d| match d {
                        Draw::Tex { x, .. } => *x,
                        _ => unreachable!(),
                    })
                    .collect();
                assert_eq!(xs.len(), if connected { 2 } else { 1 });
                if connected {
                    assert!(xs[1] >= xs[0] + 30.0);
                }
                let start = CORNER_MARGIN
                    + if battery.is_some() {
                        crate::battery::gauge_width(percent) + 10.0
                    } else {
                        0.0
                    };
                assert_eq!(xs[0], start);
                assert!(xs.last().unwrap() + 20.0 < 640.0 - CORNER_MARGIN - 100.0);
            }
        }
    }

    #[test]
    fn bluetooth_sits_after_the_percentage_and_is_absent_without_a_texture() {
        let mut draws = vec![];
        let percent = Printed::new(TexId::from_raw(1), 40);
        let battery = Some(Battery {
            percent: 75,
            charge: slot_power::Charge::Discharging,
        });
        draw_footer_connection(battery, percent, None, &mut draws);
        assert!(draws.is_empty());
        draw_footer_connection(battery, percent, Some(TexId::from_raw(2)), &mut draws);
        assert!(
            matches!(draws[0], Draw::Tex { x, w, h, .. } if w==crate::STATUS_ICON_SIZE as f32 && h==crate::STATUS_ICON_SIZE as f32 && x > FOOTER_MARGIN + 40.0 + 40.0)
        );
    }
}
