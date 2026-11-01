use slot_power::{Battery,Charge};
use slot_ui::{Canvas,Draw,Printed,TexId,status_battery_face,draw_status,draw_corner_audio,draw_corner_kid,GAUGE_W,GAUGE_H};
fn at(percent:u8,charge:Charge)->Battery{Battery{percent,charge}}
fn pixel(f:&slot_ui::UndoFace,x:u32,y:u32)->[u8;4]{f.rgba[((y*f.w+x)*4)as usize..((y*f.w+x)*4+4)as usize].try_into().unwrap()}
#[test]
fn native_battery_has_a_capsule_and_visible_level_at_every_percentage(){
    let mut fills=vec![];
    for n in [0,10,50,82,100]{let f=status_battery_face(at(n,Charge::Discharging));assert_eq!((f.w,f.h),(GAUGE_W as u32,GAUGE_H as u32));assert_eq!(pixel(&f,0,0)[3],0);let width=(0..70).filter(|&x|pixel(&f,x,4)[0]>200).count();fills.push(width);}
    assert!(fills.windows(2).all(|p|p[0]<p[1]));
}
#[test]
fn battery_state_colours_and_bolt_are_part_of_the_same_native_asset(){
    let regular=status_battery_face(at(82,Charge::Discharging));let charging=status_battery_face(at(82,Charge::Charging));let low=status_battery_face(at(20,Charge::Discharging));
    let g=pixel(&charging,10,4);assert!(g[1]>g[0]);let red=pixel(&low,4,10);assert!(red[0]>red[1]);
    assert_ne!(regular.rgba,charging.rgba);
    for percent in [0,7,10,99,100] {for light in [false,true] {let f=slot_ui::status_battery_face_on(at(percent,Charge::Charging),light);for y in 0..f.h {for x in 0..f.w {assert_eq!(pixel(&f,x,y)[3],pixel(&regular,x,y)[3],"bolt escaped capsule at {percent}%");}}}}
}
#[test]
fn time_is_left_and_the_icon_cluster_and_battery_are_right_without_overlap(){
    for width in [640.0,720.0]{let canvas=Canvas{w:width,h:480.0};let b=Some(at(100,Charge::Charging));let p=Printed::new(TexId::from_raw(1),70);let mut out=vec![];
        draw_status(canvas,b,p,None,Printed::new(TexId::from_raw(2),100),slot_ui::HUD_INK,&mut out);
        draw_corner_audio(canvas,b,p,Some(TexId::from_raw(3)),Some(TexId::from_raw(4)),&mut out);
        draw_corner_kid(canvas,b,p,TexId::from_raw(5),2,&mut out);
        let rect=|id|out.iter().find_map(|d|match d{Draw::Tex{x,y,w,h,tex,..}if *tex==TexId::from_raw(id)=>Some([*x,*y,*w,*h]),_=>None}).unwrap();
        assert_eq!(rect(2)[0],16.0);assert_eq!(rect(1)[0]+rect(1)[2],width-16.0);
        for id in 3..=5{let [x,y,w,h]=rect(id);assert_eq!((w,h),(24.0,24.0));assert!(y>=0.0&&y+h<=slot_ui::PLATE_H);assert!(x>116.0&&x+w<rect(1)[0]);}
    }
}

#[test]
fn light_battery_has_distinct_readable_ink_and_a_visible_empty_outline(){
    use slot_ui::status_battery_face_on;
    for charge in [Charge::Charging,Charge::Full,Charge::Discharging] {
        for percent in [0,10,50,82,100] {
            let f=status_battery_face_on(at(percent,charge),true);
            assert_eq!(pixel(&f,0,0)[3],0);
            assert!(pixel(&f,0,13)[..3].iter().all(|&v|v<160),"empty battery edge vanished");
            let colours:std::collections::BTreeSet<_>=f.rgba.chunks_exact(4).filter(|p|p[3]>240).map(|p|[p[0],p[1],p[2]]).collect();
            assert!(colours.len()>20,"percentage lost its antialiased text edges");
            assert!(colours.iter().any(|p|p.iter().all(|&v|v<130)));
            if percent>20 {assert!(colours.iter().any(|p|p.iter().all(|&v|v>235)),"filled battery lost its light percentage");}
        }
    }
}
