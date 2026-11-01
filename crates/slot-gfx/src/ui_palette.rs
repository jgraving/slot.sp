//! Semantic light colours for UI artwork. Game frames and cartridge art never
//! enter this path; the compositor retains each UI texture's original pixels.
pub const BACKDROP:[f32;4]=[232.0/255.0,227.0/255.0,216.0/255.0,1.0];
pub const INK:[u8;3]=[28,42,49];
pub const DIM:[u8;3]=[78,94,103];
fn near(a:[f32;3],b:[f32;3])->bool {a.into_iter().zip(b).all(|(a,b)|(a-b).abs()<0.014)}
pub fn light_colour(mut c:[f32;4])->[f32;4] {
    let rgb=[c[0],c[1],c[2]];
    let next=if near(rgb,[0.92,0.95,0.98]) {Some([0.08,0.20,0.23])}
        else if near(rgb,[0.76,0.84,0.83]) {Some([0.29,0.40,0.43])}
        else if near(rgb,[0.24,0.42,0.43]) {Some([0.69,0.86,0.81])}
        else if near(rgb,[0.20,0.31,0.32]) {Some([0.80,0.91,0.88])}
        else if near(rgb,[0.33,0.76,0.72]) {Some([0.05,0.43,0.38])}
        else if near(rgb,[0.33,0.36,0.40]) {Some([0.43,0.51,0.56])}
        else if near(rgb,[0.09,0.115,0.125]) || near(rgb,[0.08,0.10,0.12]) || near(rgb,[0.07,0.09,0.10]) || near(rgb,[0.10,0.12,0.15]) {Some([0.96,0.975,0.97])}
        else if near(rgb,[0.21,0.26,0.29]) || near(rgb,[0.22,0.25,0.29]) {Some([0.48,0.56,0.60])}
        else {None};
    if let Some(rgb)=next {c[..3].copy_from_slice(&rgb);}
    c
}
pub fn light_texture(rgba:&[u8])->Vec<u8> {
    let mut out=rgba.to_vec();
    let count=rgba.len()/4;
    let opaque=rgba.chunks_exact(4).filter(|p|p[3]>200).count();
    let dark=rgba.chunks_exact(4).filter(|p|p[3]>200&&p[..3].iter().all(|&c|c<55)).count();
    let badge=opaque*3>count*2&&dark*5>opaque*2;
    for p in out.chunks_exact_mut(4) {
        if p[3]==0 {continue;}
        // Preserve antialiasing while reversing keycap fill and lettering.
        let rgb=[p[0],p[1],p[2]];
        if badge && rgb.iter().max().unwrap()-rgb.iter().min().unwrap()<35 {
            let value=rgb.iter().map(|&c|c as f32).sum::<f32>()/3.0;
            let t=((value-26.0)/217.0).clamp(0.0,1.0);
            for i in 0..3 {p[i]=([245,249,247][i]as f32+(INK[i]as f32-[245,249,247][i]as f32)*t).round()as u8;}
            continue;
        }
        if (i16::from(rgb[1])-i16::from(rgb[0])-3).abs()<=1 && (i16::from(rgb[2])-i16::from(rgb[0])-8).abs()<=1 {
            let t=((rgb[0]as f32-34.0)/190.0).clamp(0.0,1.0);
            for i in 0..3 {p[i]=([246,249,248][i]as f32+(INK[i]as f32-[246,249,248][i]as f32)*t).round()as u8;}
            continue;
        }
        if rgb.into_iter().zip([242,247,245]).all(|(a,b)|a.abs_diff(b)<=1) {continue;}
        let c=[p[0]as f32/255.0,p[1]as f32/255.0,p[2]as f32/255.0,1.0];
        let mapped=light_colour(c);
        if mapped!=c {for i in 0..3 {p[i]=(mapped[i]*255.0).round()as u8;}continue;}
        let lo=*rgb.iter().min().unwrap();let hi=*rgb.iter().max().unwrap();
        if hi-lo<30 {
            if lo>230 {p[..3].copy_from_slice(&INK);}
            else if lo>105 {p[..3].copy_from_slice(&DIM);}
        }
    }
    out
}
/// Footer artwork uses semantic keycap colours, so invert the capsule and its
/// lettering together while choosing the caption for the actual backdrop.
/// Original pixels are retained by the compositor for exact palette changes.
pub fn hint_texture(rgba:&[u8],dark_ink:bool)->Vec<u8> {
    let mut out=rgba.to_vec();
    let caption=if dark_ink {INK}else{[246,244,239]};
    let (letter,fill)=if dark_ink {([242,247,245],INK)}else{([34,37,42],[224,227,232])};
    for p in out.chunks_exact_mut(4) {
        if p[3]==0 {continue;}
        let cap=(i16::from(p[1])-i16::from(p[0])-3).abs()<=1
            && (i16::from(p[2])-i16::from(p[0])-8).abs()<=1;
        if cap {
            let t=((p[0]as f32-34.0)/190.0).clamp(0.0,1.0);
            for c in 0..3 {p[c]=(letter[c]as f32+(fill[c]as f32-letter[c]as f32)*t).round()as u8;}
        } else {p[..3].copy_from_slice(&caption);}
    }
    out
}

/// Lift wallpaper tones into a pale range, keeping the original hue, detail and
/// alpha. Normalising to the source peak also gives the dark built-in static
/// gradient its full light palette. Only background textures use this path.
pub fn light_background(rgba: &[u8]) -> Vec<u8> {
    let peak = rgba.chunks_exact(4).filter(|p| p[3] != 0)
        .flat_map(|p| p[..3].iter().copied()).max().unwrap_or(1).max(1) as f32;
    let mut out = rgba.to_vec();
    for p in out.chunks_exact_mut(4) {
        if p[3] == 0 { continue; }
        for i in 0..3 {
            p[i] = ([232,227,216][i] as f32 - (peak-p[i] as f32) * (65.0/peak)).round() as u8;
        }
    }
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    fn luminance(rgb:[u8;3])->f32 {rgb.into_iter().zip([0.2126,0.7152,0.0722]).map(|(v,w)|{let x=v as f32/255.0;w*if x<=0.04045{x/12.92}else{((x+0.055)/1.055).powf(2.4)}}).sum()}
    #[test] fn readable_text_preserves_alpha_and_keycap_labels(){let src=[246,244,239,128,224,227,232,255,34,37,42,255,242,247,245,255];let p=light_texture(&src);assert_eq!(&p[..4],&[28,42,49,128]);assert_eq!(&p[4..8],&[28,42,49,255]);assert_eq!(&p[8..12],&[246,249,248,255]);assert_eq!(&p[12..],&src[12..]);for ink in [INK,DIM] {assert!((luminance([245,249,247])+0.05)/(luminance(ink)+0.05)>4.5);}}
    #[test] fn badge_text_edges_stay_smooth_through_the_keycap_colour_range(){
        let mut rgba=vec![];for _ in 0..250 {rgba.extend_from_slice(&[26,31,36,255]);}
        for n in 0..=100 {for (bg,ink) in [26,31,36].into_iter().zip([246,244,239]) {rgba.push(((bg* (100-n)+ink*n)/100)as u8);}rgba.push(255);}
        let mapped=light_texture(&rgba);let mut previous=255;
        for p in mapped[250*4..].chunks_exact(4){assert!(p[0]<=previous,"antialiasing jumped at a mixed text pixel");previous=p[0];}
    }
    #[test] fn wallpaper_lift_keeps_hue_order_alpha_and_readable_contrast(){
        let src=[15,40,85,255,80,20,100,128,0,0,0,0];let mapped=light_background(&src);
        assert!(mapped[0]<mapped[1]&&mapped[1]<mapped[2]);assert!(mapped[4]>mapped[5]&&mapped[5]<mapped[6]);
        assert_eq!(mapped[3],255);assert_eq!(mapped[7],128);assert_eq!(&mapped[8..],&src[8..]);
        for p in mapped[..8].chunks_exact(4){assert!((luminance([p[0],p[1],p[2]])+0.05)/(luminance(INK)+0.05)>4.5);}
    }
    #[test] fn coloured_badges_are_preserved(){assert_eq!(light_texture(&[249,186,45,255,120,230,150,255]),[249,186,45,255,120,230,150,255]);}
}

/// Choose header ink from a few CPU samples; wallpaper changes never require
/// reading the framebuffer or sampling every frame.
pub fn header_contrast(rgba:&[u8],w:u32,h:u32)->[bool;3] {
    if w==0||h==0||rgba.len()<(w*h*4)as usize {return [false;3];}
    let zones=[(16,w.min(116),40),(w.saturating_sub(230),w.saturating_sub(16),40),(w.saturating_sub(260)/2,(w+260).min(2*w)/2,56)];
    zones.map(|(x0,x1,y1)| {
        let mut luma=0.0;let mut count=0;
        for y in (0..h.min(y1)).step_by(6) {for x in (x0..x1).step_by(8) {
            let p=&rgba[((y*w+x)*4)as usize..];
            luma+=p[..3].iter().zip([0.2126,0.7152,0.0722]).map(|(&v,k)|{let c=v as f32/255.0;k*if c<=0.04045{c/12.92}else{((c+0.055)/1.055).powf(2.4)}}).sum::<f32>();count+=1;
        }}
        count>0&&luma/count as f32>0.24
    })
}
/// Choose clean backdrop ink without changing the original glyph coverage.
pub fn backdrop_text(rgba:&[u8],w:u32,h:u32,dark:bool)->Vec<u8> {
    debug_assert_eq!(rgba.len(),(w*h*4)as usize);
    let mut out=rgba.to_vec();let ink=if dark {INK}else{[246,244,239]};
    for p in out.chunks_exact_mut(4) {if p[3]>0 {p[..3].copy_from_slice(&ink);}}
    out
}

#[cfg(test)]
mod backdrop_tests {
    use super::*;
    #[test]
    fn custom_wallpaper_can_choose_different_ink_for_each_header_region() {
        let (w,h)=(640,80);let mut pixels=vec![0;(w*h*4)as usize];
        for y in 0..h {for x in 0..w {let c=if x<180 {240}else{20};let i=((y*w+x)*4)as usize;pixels[i..i+4].copy_from_slice(&[c,c,c,255]);}}
        assert_eq!(header_contrast(&pixels,w,h),[true,false,false]);
    }
    #[test]
    fn header_text_retains_intermediate_alpha_in_both_contrast_palettes() {
        let pixels=[0,0,0,0,246,244,239,64,246,244,239,128,246,244,239,255,0,0,0,0];
        for light in [false,true] {let p=backdrop_text(&pixels,5,1,light);for (before,after) in pixels.chunks_exact(4).zip(p.chunks_exact(4)) {assert_eq!(before[3],after[3],"adaptive ink added an outline");if after[3]>0 {assert_eq!(&after[..3],if light{&INK}else{&[246,244,239]});}}}
    }
}
