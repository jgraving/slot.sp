//! Shared expressive indeterminate indicator: rounded shapes morph while rotating.
//! Rasterised once into a bounded cache, never on the animation/frame thread.
use crate::UndoFace;
pub const LOADING_FRAMES: usize = 72;
pub const LOADING_FRAME_MS: u128 = 20;
pub fn loading_face(frame: usize) -> UndoFace {
    loading_face_at(frame,32)
}
pub fn loading_face_at(frame:usize,size:u32)->UndoFace {
    let (w,h)=(size,size);
    let t=(frame%LOADING_FRAMES) as f32/LOADING_FRAMES as f32;
    let turn=std::f32::consts::TAU;
    let phase=t*4.0;
    let shape=phase.floor() as usize;
    let u=phase.fract();
    let mix=u*u*(3.0-2.0*u);
    let radii=|angle:f32,shape:usize| match shape%4 {
        0=>11.0,
        1=>10.5+1.4*(angle*4.0).cos(),
        2=>10.5+1.5*(angle*3.0).cos(),
        _=>10.8+1.0*(angle*6.0).cos(),
    };
    let mut rgba=vec![0;(w*h*4) as usize];
    for y in 0..h {for x in 0..w {
        let mut coverage=0u32;
        for sy in 0..4 {for sx in 0..4 {
            let dx=(x as f32+(sx as f32+0.5)/4.0)*32.0/size as f32-16.0;
            let dy=(y as f32+(sy as f32+0.5)/4.0)*32.0/size as f32-16.0;
            let angle=dy.atan2(dx)-t*turn;
            let r=radii(angle,shape)*(1.0-mix)+radii(angle,shape+1)*mix;
            if dx*dx+dy*dy<=r*r {coverage+=1;}
        }}
        let at=((y*w+x)*4) as usize;
        rgba[at..at+4].copy_from_slice(&[164,220,211,(coverage*255/16) as u8]);
    }}
    UndoFace{rgba,w,h}
}
#[cfg(test)]mod tests {
    use super::*;
    #[test]fn morphs_are_bounded_antialiased_and_seamless(){
        let first=loading_face(0);
        assert_eq!(first.rgba,loading_face(LOADING_FRAMES).rgba);
        assert_ne!(first.rgba,loading_face(LOADING_FRAMES/4).rgba);
        for i in 0..LOADING_FRAMES {
            let f=loading_face(i);
            assert!(f.rgba.chunks_exact(4).any(|p|p[3]>0&&p[3]<255));
            for x in 0..32 {assert_eq!(f.rgba[(x*4+3) as usize],0);}
        }
    }
}
