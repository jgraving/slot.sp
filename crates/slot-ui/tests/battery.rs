use slot_power::{Battery, Charge};
use slot_ui::{draw_gauge, Draw, Printed, TexId, GAUGE_H, GAUGE_W, HUD_INK, WALL};

fn percent_face()->Printed {Printed::new(TexId::from_raw(10),60)}
fn at(percent:u8,charge:Charge)->Option<Battery> {Some(Battery{percent,charge})}
fn gauge(percent:u8,charge:Charge)->Vec<Draw> {
    let mut out=vec![];
    draw_gauge(24.0,100.0,at(percent,charge),percent_face(),Some(TexId::from_raw(7)),HUD_INK,&mut out);
    out
}
#[test]
fn percentage_and_charging_bolt_stay_inside_the_battery() {
    for charge in [Charge::Charging,Charge::Discharging,Charge::Full,Charge::Unknown] {
        let out=gauge(100,charge);
        for draw in &out {
            if let Draw::Tex{x,y,w,h,..}=draw {
                assert!(*x>=24.0+WALL && x+w<=24.0+GAUGE_W-WALL);
                assert!(*y>=100.0+WALL && y+h<=100.0+GAUGE_H-WALL);
            }
        }
        assert_eq!(out.iter().any(|d|matches!(d,Draw::Tex{tex,..}if *tex==TexId::from_raw(7))),charge==Charge::Charging);
        assert!(out.iter().any(|d|matches!(d,Draw::Tex{tex,..}if *tex==TexId::from_raw(10))));
    }
}
#[test]
fn charge_states_keep_the_same_body_and_use_state_colours() {
    let body=|charge,percent|match gauge(percent,charge)[0] {
        Draw::RoundRect{x,y,w,h,colour,..}=>(x,y,w,h,colour),_=>panic!()
    };
    let normal=body(Charge::Discharging,50);
    let low=body(Charge::Discharging,20);
    let charging=body(Charge::Charging,20);
    assert_eq!((normal.0,normal.1,normal.2,normal.3),(charging.0,charging.1,charging.2,charging.3));
    assert!(low.4[0]>low.4[1]*2.0);
    assert!(charging.4[1]>charging.4[0]);
    assert_ne!(normal.4,low.4);
}
#[test]
fn fill_grows_with_reading_and_stays_inside_the_body() {
    let mut widths=vec![];
    for percent in [0,1,50,99,100] {
        let out=gauge(percent,Charge::Discharging);
        let width=out.iter().find_map(|d|match d {
            Draw::RoundRect{x,w,..}if (*x-(24.0+2.0*WALL)).abs()<0.01=>Some(*w),_=>None
        }).unwrap_or(0.0);
        assert!(width<=GAUGE_W-4.0*WALL);
        widths.push(width);
    }
    assert!(widths.windows(2).all(|p|p[1]>p[0]));
}

#[test]
fn enlarged_status_glyphs_and_text_stay_inside_the_status_band() {
    let mut out=vec![];
    let canvas=slot_ui::Canvas{w:640.0,h:480.0};
    slot_ui::draw_status(canvas,at(100,Charge::Charging),Printed::new(TexId::from_raw(1),60),Some(TexId::from_raw(2)),Printed::new(TexId::from_raw(3),70),HUD_INK,&mut out);
    slot_ui::draw_corner_audio(at(100,Charge::Charging),Printed::new(TexId::from_raw(1),60),Some(TexId::from_raw(4)),Some(TexId::from_raw(5)),&mut out);
    slot_ui::draw_corner_kid(at(100,Charge::Charging),Printed::new(TexId::from_raw(1),60),TexId::from_raw(6),2,&mut out);
    assert!(out.iter().all(|d|matches!(d,Draw::Rect{y,h,..}|Draw::RoundRect{y,h,..}|Draw::Tex{y,h,..}if *y>=0.0&&y+h<=slot_ui::PLATE_H)));
    assert!(out.iter().any(|d|matches!(d,Draw::Tex{w,h,..}if *w==32.0&&*h==32.0)));
}

#[test]
fn no_reading_draws_nothing() {
    let mut out = Vec::new();
    draw_gauge(24.0, 400.0, None, percent_face(), None, HUD_INK, &mut out);
    assert!(out.is_empty());
}
