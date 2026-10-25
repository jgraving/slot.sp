use slot::brand_motion::{frame_at,boot_opacity,FRAMES,REVEAL_MS,BOOT_MS};
#[test]
fn reveal_locks_to_final_frame_and_fade_ends_without_replaying() {
    assert_eq!(frame_at(0),0);
    assert_eq!(frame_at(REVEAL_MS),FRAMES-1);
    assert_eq!(frame_at(u64::MAX),FRAMES-1);
    let mut previous=0;
    for ms in 0..=BOOT_MS {
        let frame=frame_at(ms);
        assert!(frame>=previous);previous=frame;
        assert!((0.0..=1.0).contains(&boot_opacity(ms)));
    }
    assert_eq!(boot_opacity(REVEAL_MS),1.0);
    assert_eq!(boot_opacity(BOOT_MS),0.0);
}

#[test]
fn atlas_opens_transparently_and_locks_to_an_opaque_white_mark() {
    let bytes=include_bytes!("../../../assets/brand-motion.png");
    let mut reader=png::Decoder::new(std::io::Cursor::new(bytes)).read_info().unwrap();
    let mut pixels=vec![0;reader.output_buffer_size()];
    let info=reader.next_frame(&mut pixels).unwrap();
    assert_eq!((info.width,info.height),(1920,1536));
    for y in 0..128 { for x in 0..480 {
        assert_eq!(pixels[(y*1920+x)*4+3],0,"the opening frame has a ghost");
    }}
    let mut solid=0;
    for y in 1408..1536 { for x in 1440..1920 {
        let p=&pixels[(y*1920+x)*4..][..4];
        if p[3]==255 {solid+=1;assert!(p[0]>=245 && p[1]>=246 && p[2]>=241);}
    }}
    assert!(solid>3000,"the final mark disappeared");
}
