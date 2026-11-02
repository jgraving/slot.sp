//! Surfaceless Mesa harness for production-compositor UI review (Linux only).
use libloading::Library;
use slot_gfx::{GfxError, Surface};
use std::ffi::{c_void, CString};
type Ptr = *mut c_void;
pub struct Egl {
    lib: Library,
    display: Ptr,
    surface: Ptr,
    context: Ptr,
}
impl Egl {
    pub fn new(es: bool) -> Self {
        unsafe {
            let lib = Library::new("libEGL.so.1").expect("Mesa EGL is required for UI previews");
            let platform = lib
                .get::<unsafe extern "C" fn(u32, Ptr, *const isize) -> Ptr>(
                    b"eglGetPlatformDisplay\0",
                )
                .unwrap();
            let display = platform(0x31DD, std::ptr::null_mut(), std::ptr::null());
            let init = lib
                .get::<unsafe extern "C" fn(Ptr, *mut i32, *mut i32) -> u32>(b"eglInitialize\0")
                .unwrap();
            assert_ne!(
                init(display, std::ptr::null_mut(), std::ptr::null_mut()),
                0,
                "EGL initialisation"
            );
            let api = lib
                .get::<unsafe extern "C" fn(u32) -> u32>(b"eglBindAPI\0")
                .unwrap();
            assert_ne!(api(if es { 0x30A0 } else { 0x30A2 }), 0);
            let choose = lib
                .get::<unsafe extern "C" fn(Ptr, *const i32, *mut Ptr, i32, *mut i32) -> u32>(
                    b"eglChooseConfig\0",
                )
                .unwrap();
            let attrs = [
                0x3033,
                1,
                0x3040,
                if es { 4 } else { 8 },
                0x3024,
                8,
                0x3023,
                8,
                0x3022,
                8,
                0x3021,
                8,
                0x3038,
            ];
            let mut config = std::ptr::null_mut();
            let mut count = 0;
            assert_ne!(
                choose(display, attrs.as_ptr(), &mut config, 1, &mut count),
                0
            );
            assert!(count > 0);
            let pbuffer = lib
                .get::<unsafe extern "C" fn(Ptr, Ptr, *const i32) -> Ptr>(
                    b"eglCreatePbufferSurface\0",
                )
                .unwrap();
            let surface = pbuffer(display, config, [0x3057, 720, 0x3056, 480, 0x3038].as_ptr());
            let create = lib
                .get::<unsafe extern "C" fn(Ptr, Ptr, Ptr, *const i32) -> Ptr>(
                    b"eglCreateContext\0",
                )
                .unwrap();
            let es_attrs = [0x3098, 2, 0x3038];
            let core_attrs = [0x3098, 3, 0x30FB, 3, 0x30FD, 1, 0x3038];
            let context = create(
                display,
                config,
                std::ptr::null_mut(),
                if es {
                    es_attrs.as_ptr()
                } else {
                    core_attrs.as_ptr()
                },
            );
            assert!(!context.is_null() && !surface.is_null(), "EGL context");
            let mut s = Self {
                lib,
                display,
                surface,
                context,
            };
            s.make_current().unwrap();
            s
        }
    }
}
impl Surface for Egl {
    fn make_current(&mut self) -> Result<(), GfxError> {
        unsafe {
            let f = self
                .lib
                .get::<unsafe extern "C" fn(Ptr, Ptr, Ptr, Ptr) -> u32>(b"eglMakeCurrent\0")
                .unwrap();
            if f(self.display, self.surface, self.surface, self.context) != 0 {
                Ok(())
            } else {
                Err(GfxError::Context("EGL make current".into()))
            }
        }
    }
    fn window_size(&self) -> (u32, u32) {
        (720, 480)
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
impl Drop for Egl {
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
            let destroy = self
                .lib
                .get::<unsafe extern "C" fn(Ptr, Ptr) -> u32>(b"eglDestroyContext\0")
                .unwrap();
            destroy(self.display, self.context);
            let destroy = self
                .lib
                .get::<unsafe extern "C" fn(Ptr, Ptr) -> u32>(b"eglDestroySurface\0")
                .unwrap();
            destroy(self.display, self.surface);
            let end = self
                .lib
                .get::<unsafe extern "C" fn(Ptr) -> u32>(b"eglTerminate\0")
                .unwrap();
            end(self.display);
        }
    }
}
