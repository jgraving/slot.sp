//! Review the boot handoff over the real launcher and verify both endpoints.
#[cfg(target_os="linux")]
#[path="../../slot-ui/examples/support/mod.rs"] mod support;
#[cfg(target_os="linux")]
fn main(){
    use slot::{frontend::Frontend,boot_motion::{BootMotion,BOOT_MS}};
    use slot_gfx::{Canvas,Compositor};
    let dir=std::path::PathBuf::from(std::env::args().nth(1).unwrap());std::fs::create_dir_all(&dir).unwrap();
    let width:u32=std::env::args().nth(3).and_then(|s|s.parse().ok()).unwrap_or(640);
    let root=dir.join("card");slot::root::ensure(&root);
    slot_store::write_slot_state(&root,&slot_store::SlotState{clock_set:true,..Default::default()}).unwrap();
    for name in ["Aurora","Borealis","Celeste"] {std::fs::write(root.join("Games/GBA").join(format!("{name}.gba")),[0;256]).unwrap();}
    let surface=support::Egl::new(std::env::args().nth(2).as_deref()!=Some("gl"));
    let mut g=Compositor::new(&surface).unwrap();g.fit((width,480));
    let mut f=Frontend::boot(Box::new(slot_power::SimPlatform::at(root)));f.upload_faces(&mut g);
    f.render(&mut g,(width,480));
    let motion=BootMotion::upload(&mut g).unwrap();let canvas=Canvas{w:width as f32,h:480.0};
    let firmware=slot::bootlogo::decode(include_bytes!("../../../card/System/bootlogo.bmp")).unwrap();
    let firmware=slot::bootlogo::compose(&firmware,(width as usize,480)).unwrap();
    for frame in 0..=66 {
        let ms=frame*1000/60;f.compose(&mut g);let normal=g.read_frame();
        let mut overlay=vec![];motion.draw(ms,canvas,&mut overlay);g.draw_list(&overlay);let pixels=g.read_frame();
        if ms==0 {for (rgba,rgb) in pixels.chunks_exact(4).zip(&firmware.rgb) {assert_eq!(&rgba[..3],rgb,"boot handoff must open on the exact firmware logo");}}
        if ms>=BOOT_MS {assert_eq!(pixels,normal,"the final overlay must leave the normal launcher unchanged");}
        let mut e=png::Encoder::new(std::fs::File::create(dir.join(format!("boot-{frame:03}.png"))).unwrap(),width,480);e.set_color(png::ColorType::Rgba);e.set_depth(png::BitDepth::Eight);e.write_header().unwrap().write_image_data(&pixels).unwrap();
    }
    motion.release(&mut g);
}
#[cfg(not(target_os="linux"))] fn main(){}
