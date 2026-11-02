#![cfg(target_os = "linux")]
use libloading::Library;
use slot::video_mode::{self, VideoMode};
use slot_gfx::{Compositor, Draw, GfxError, ScreenEffect, Surface};
use slot_store::Platform;
use std::ffi::{CString, c_void};
type Ptr = *mut c_void;
struct Offscreen {
    lib: Library,
    display: Ptr,
    context: Ptr,
    surface: Ptr,
    size: (u32, u32),
}
impl Offscreen {
    fn new(size: (u32, u32)) -> Self {
        unsafe {
            let lib = Library::new("libEGL.so.1").unwrap();
            let get = lib
                .get::<unsafe extern "C" fn(u32, Ptr, *const isize) -> Ptr>(
                    b"eglGetPlatformDisplay\0",
                )
                .unwrap();
            let display = get(0x31DD, std::ptr::null_mut(), std::ptr::null());
            assert!(!display.is_null());
            let init = lib
                .get::<unsafe extern "C" fn(Ptr, *mut i32, *mut i32) -> u32>(b"eglInitialize\0")
                .unwrap();
            let (mut a, mut b) = (0, 0);
            assert_ne!(init(display, &mut a, &mut b), 0, "EGL unavailable");
            let bind = lib
                .get::<unsafe extern "C" fn(u32) -> u32>(b"eglBindAPI\0")
                .unwrap();
            assert_ne!(bind(0x30A2), 0);
            let choose = lib
                .get::<unsafe extern "C" fn(Ptr, *const i32, *mut Ptr, i32, *mut i32) -> u32>(
                    b"eglChooseConfig\0",
                )
                .unwrap();
            let attrs = [
                0x3033, 1, 0x3040, 8, 0x3024, 8, 0x3023, 8, 0x3022, 8, 0x3038,
            ];
            let mut config = std::ptr::null_mut();
            let mut n = 0;
            assert_ne!(choose(display, attrs.as_ptr(), &mut config, 1, &mut n), 0);
            assert!(n > 0);
            let create = lib
                .get::<unsafe extern "C" fn(Ptr, Ptr, Ptr, *const i32) -> Ptr>(
                    b"eglCreateContext\0",
                )
                .unwrap();
            let context = create(
                display,
                config,
                std::ptr::null_mut(),
                [0x3098, 3, 0x30FB, 3, 0x3038].as_ptr(),
            );
            assert!(!context.is_null());
            let pb = lib
                .get::<unsafe extern "C" fn(Ptr, Ptr, *const i32) -> Ptr>(
                    b"eglCreatePbufferSurface\0",
                )
                .unwrap();
            let surface = pb(
                display,
                config,
                [0x3057, size.0 as i32, 0x3056, size.1 as i32, 0x3038].as_ptr(),
            );
            assert!(!surface.is_null());
            let mut s = Self {
                lib,
                display,
                context,
                surface,
                size,
            };
            s.make_current().unwrap();
            s
        }
    }
}
impl Surface for Offscreen {
    fn make_current(&mut self) -> Result<(), GfxError> {
        unsafe {
            let f = self
                .lib
                .get::<unsafe extern "C" fn(Ptr, Ptr, Ptr, Ptr) -> u32>(b"eglMakeCurrent\0")
                .unwrap();
            assert_ne!(f(self.display, self.surface, self.surface, self.context), 0);
        }
        Ok(())
    }
    fn window_size(&self) -> (u32, u32) {
        self.size
    }
    fn swap(&mut self) -> Result<(), GfxError> {
        Ok(())
    }
    fn proc_address(&self, name: &str) -> *const c_void {
        unsafe {
            let f = self
                .lib
                .get::<unsafe extern "C" fn(*const i8) -> *const c_void>(b"eglGetProcAddress\0")
                .unwrap();
            f(CString::new(name).unwrap().as_ptr())
        }
    }
}
impl Drop for Offscreen {
    fn drop(&mut self) {
        unsafe {
            let current = self
                .lib
                .get::<unsafe extern "C" fn(Ptr, Ptr, Ptr, Ptr) -> u32>(b"eglMakeCurrent\0")
                .unwrap();
            current(
                self.display,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            let ctx = self
                .lib
                .get::<unsafe extern "C" fn(Ptr, Ptr) -> u32>(b"eglDestroyContext\0")
                .unwrap();
            ctx(self.display, self.context);
            let surf = self
                .lib
                .get::<unsafe extern "C" fn(Ptr, Ptr) -> u32>(b"eglDestroySurface\0")
                .unwrap();
            surf(self.display, self.surface);
            let term = self
                .lib
                .get::<unsafe extern "C" fn(Ptr) -> u32>(b"eglTerminate\0")
                .unwrap();
            term(self.display);
        }
    }
}
#[test]
fn fit_and_integer_render_at_native_panel_resolution_without_stretching_or_pause_shift() {
    for panel in [(640, 480), (720, 480)] {
        let surface = Offscreen::new(panel);
        let mut c = Compositor::new(&surface).unwrap();
        c.fit(panel);
        c.set_screen_power(1.0);
        c.set_screen_effect(ScreenEffect::None);
        for platform in [Platform::Gba, Platform::Gb] {
            for mode in [VideoMode::Fit, VideoMode::Integer] {
                let (sw, sh) = platform.picture();
                let mut frame = vec![0u8; 240 * 160 * 4];
                for y in 0..sh {
                    for x in 0..sw {
                        let i = (((y + (160 - sh) / 2) * 240 + x + (240 - sw) / 2) * 4) as usize;
                        frame[i..i + 4].copy_from_slice(if (x + y) % 2 == 0 {
                            &[20, 80, 240, 255]
                        } else {
                            &[240, 140, 40, 255]
                        });
                    }
                }
                c.set_game_source_rect(video_mode::source_rect(platform, mode));
                let rect = mode.destination(platform, panel).unwrap();
                c.set_game_destination(Some(rect));
                c.upload_game(&frame);
                // A saved-state preview must use the same crop and placement as live play.
                let still = c.create_texture(240, 160, &frame);
                let mut boxes = Vec::new();
                for foot in [0.0, 80.0] {
                    c.set_screen_footer(foot);
                    c.begin_frame();
                    c.draw_list(&[Draw::Game]);
                    let live = c.read_frame();
                    c.begin_frame();
                    c.draw_list(&[Draw::Shot { tex: still }]);
                    let shot = c.read_frame();
                    for (a, b) in live.chunks_exact(4).zip(shot.chunks_exact(4)) {
                        assert_eq!(
                            a[..3].iter().any(|v| *v > 10),
                            b[..3].iter().any(|v| *v > 10),
                            "saved-state preview placement differs"
                        );
                    }
                    assert_eq!(c.frame_size(), panel);
                    let pixels = c.read_frame();
                    let (mut left, mut top, mut right, mut bottom) = (panel.0, panel.1, 0, 0);
                    for y in 0..panel.1 {
                        for x in 0..panel.0 {
                            let i = ((y * panel.0 + x) * 4) as usize;
                            if pixels[i..i + 3].iter().any(|c| *c > 10) {
                                left = left.min(x);
                                top = top.min(y);
                                right = right.max(x + 1);
                                bottom = bottom.max(y + 1);
                            }
                        }
                    }
                    boxes.push((left, top, right, bottom));
                    assert!((left as f32 - rect[0] / 720.0 * panel.0 as f32).abs() < 1.1);
                    assert!((top as f32 - rect[1] / 480.0 * panel.1 as f32).abs() < 1.1);
                    if foot == 0.0 {
                        assert!(
                            ((right - left) as f32 - rect[2] / 720.0 * panel.0 as f32).abs() < 1.1
                        );
                        assert!(
                            ((bottom - top) as f32 - rect[3] / 480.0 * panel.1 as f32).abs() < 1.1
                        );
                    }
                    if let Ok(dir) = std::env::var("SLOT_FEATURE_PREVIEW") {
                        let name = format!(
                            "{dir}/scaling-{}-{}-{}-{foot}.png",
                            panel.0,
                            platform as u8,
                            mode.as_str()
                        );
                        let f = std::fs::File::create(name).unwrap();
                        let mut e = png::Encoder::new(f, panel.0, panel.1);
                        e.set_color(png::ColorType::Rgba);
                        e.set_depth(png::BitDepth::Eight);
                        e.write_header().unwrap().write_image_data(&pixels).unwrap();
                    }
                }
                assert_eq!(boxes[0].0, boxes[1].0);
                assert_eq!(boxes[0].1, boxes[1].1);
                assert_eq!(boxes[0].2, boxes[1].2);
                assert!(boxes[1].3 <= boxes[0].3);
            }
        }
    }
}
