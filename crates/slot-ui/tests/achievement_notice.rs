use slot_ui::{achievement_notice_face, Canvas};

#[test]
fn complete_badges_fit_separately_from_long_copy_at_both_widths() {
    for width in [640.0,720.0] {
        for (sw,sh) in [(64,64),(64,128),(128,64),(16,16)] {
            let rgba=[220,80,40,255].repeat((sw*sh) as usize);
            let face=achievement_notice_face("A long achievement title that needs wrapping without touching the badge (+25)",
                "Complete an unusually demanding challenge without taking damage or collecting power-ups.",
                Some((&rgba,sw,sh)),Canvas{w:width,h:480.0},false);
            assert!(face.w<=width as u32-64 && face.h<=126);
            let mut xs=vec![];let mut ys=vec![];
            for y in 0..face.h {for x in 0..88 {
                if face.rgba[((y*face.w+x)*4)as usize..][..4]==[220,80,40,255] {xs.push(x);ys.push(y);}
            }}
            let bw=xs.iter().max().unwrap()-xs.iter().min().unwrap()+1;
            let bh=ys.iter().max().unwrap()-ys.iter().min().unwrap()+1;
            assert!(bw<=48 && bh<=48 && bw<=sw && bh<=sh);
            assert_eq!(bw*sh,bh*sw,"aspect ratio must survive fitting");
            assert!(*xs.iter().min().unwrap()>=24 && *ys.iter().min().unwrap()>=16);
        }
    }
}

#[test]
fn reduction_averages_pixels_and_transparent_rgb_cannot_dirty_edges() {
    let mut rgba=vec![];
    for y in 0..64 {for x in 0..64 {rgba.extend_from_slice(if (x+y)%2==0 {&[255,0,0,255]}else{&[0,0,255,0]});}}
    let face=achievement_notice_face("Smooth badge (+5)","Test",Some((&rgba,64,64)),Canvas::FRAMED,false);
    let badge:Vec<_>=(0..face.h).flat_map(|y|(24..72).map(move|x|(y,x))).map(|(y,x)| &face.rgba[((y*face.w+x)*4)as usize..][..4])
        .filter(|p|p[0]==255 && p[1]==0 && p[2]==0 && p[3]>0).collect();
    assert!(!badge.is_empty());assert!(badge.iter().any(|p|p[3]>0 && p[3]<255));
}

#[test]
fn missing_artwork_keeps_readable_text_and_padding() {
    let face=achievement_notice_face("Offline unlock (+5)","The notification still works.",None,Canvas::FRAMED,false);
    assert!(face.rgba.chunks_exact(4).any(|p|p[3]>0));
    for y in 0..face.h {for x in 0..24 {assert_eq!(face.rgba[((y*face.w+x)*4+3)as usize],0);}}
}
