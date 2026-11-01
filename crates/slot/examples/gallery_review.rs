//! Review the production screenshot gallery in both themes and graphics APIs.
#[cfg(target_os="linux")]
#[path="../../slot-ui/examples/support/mod.rs"] mod support;
#[cfg(target_os="linux")]
fn main() {
    use slot_gfx::{Compositor,Draw};
    use slot_ui::{Canvas,ShotViewer};
    let out=std::path::PathBuf::from(std::env::args().nth(1).expect("output directory"));
    std::fs::create_dir_all(&out).unwrap();
    let surface=support::Egl::new(std::env::args().nth(2).as_deref()!=Some("gl"));
    let width:u32=std::env::args().nth(3).and_then(|s|s.parse().ok()).unwrap_or(720);
    let canvas=Canvas{w:width as f32,h:480.0};
    let mut gpu=Compositor::new(&surface).unwrap();gpu.fit((width,480));
    let photo=[54u8,112,168,255].repeat(720*480);
    let tex=gpu.create_texture_nearest(720,480,&photo);
    let hints:Vec<_>=slot_ui::gallery_button_legends().into_iter().map(|f| {
        let tex=gpu.create_ui_texture(f.w,f.h,&f.rgba);gpu.mark_hint_texture(tex,0);(tex,f.w)
    }).collect();
    for light in [false,true] {
        gpu.set_light_ui(light);gpu.set_hint_light([light;2]);
        let mut viewer=ShotViewer::new((0..6).map(|i|slot_store::ShotEntry{path:format!("shot-{i}.png").into(),stamp:format!("2026-11-01_09-01-{i:02}")}).collect());
        viewer.set_hint_faces(hints.clone());
        for entry in viewer.entries.clone(){viewer.set_thumb(&entry.path,Some((tex,720,480)));}
        let f=slot_ui::title_face("Just now");let title=gpu.create_ui_texture(f.w,f.h,&f.rgba);viewer.set_title_face(Some(title));
        for mode in ["grid","select","full","zoom"] {
            match mode {"select"=>{viewer.toggle_selecting();viewer.tick_selected();},"full"=>{viewer.toggle_selecting();viewer.open();viewer.set_face(Some((tex,720,480)));},"zoom"=>viewer.zoom_in(canvas),_=>{}}
            let f=slot_ui::word_face(&viewer.count_text());let count=gpu.create_ui_texture(f.w,f.h,&f.rgba);viewer.set_count_face(Some((count,f.w)));
            let mut draws=vec![];viewer.draw_on(canvas,None,Default::default(),None,Default::default(),light,&mut draws);
            gpu.begin_frame();gpu.draw_list(&draws);let pixels=gpu.read_frame();
            let pixel=|x:u32,y:u32|&pixels[((y*width+x)*4)as usize..((y*width+x)*4+3)as usize];
            let (x,y)=if mode=="grid"||mode=="select" {let (w,h)=slot_ui::grid_cell(canvas);((16.0+w/2.0)as u32,(56.0+h/2.0)as u32)}else{(width/2,240)};
            assert!(pixel(x,y).iter().zip([54,112,168]).all(|(&a,b)|a.abs_diff(b)<=1),"gallery recoloured the screenshot: {light} {mode}");
            assert!(pixel(3,460).iter().all(|&c|if light {c>220}else{c<40}),"footer contrast: {light} {mode}");
            for d in &draws {if let Draw::Tex{x,y,w,h,tex,..}=*d {if hints.iter().any(|&(id,_)|id==tex){assert!(x>=15.0&&x+w<canvas.w-15.0&&y>=440.0&&y+h<=480.0&&h>=28.0,"hint clipped: {width} {light} {mode}");}}}
            let mut e=png::Encoder::new(std::fs::File::create(out.join(format!("{}-{mode}.png",if light{"light"}else{"dark"}))).unwrap(),width,480);
            e.set_color(png::ColorType::Rgba);e.set_depth(png::BitDepth::Eight);e.write_header().unwrap().write_image_data(&pixels).unwrap();
        }
    }
    println!("Gallery colors, screenshot pixels and button hints passed for {width}px");
}
#[cfg(not(target_os="linux"))]
fn main() {}
