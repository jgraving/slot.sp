// Surfaceless Mesa context for the production compositor; no window or device.
use std::ffi::{c_void, CString};
use slot_gfx::{Surface, GfxError};
pub struct HeadlessSurface { lib: libloading::Library }
impl HeadlessSurface {
 pub fn new() -> Result<Self, GfxError> { unsafe {
  let lib=libloading::Library::new("libEGL.so.1").unwrap();
  macro_rules! call { ($n:literal, $t:ty, $($a:expr),*) => { lib.get::<$t>(concat!($n,"\0").as_bytes()).unwrap()($($a),*) }; }
  let d=call!("eglGetPlatformDisplay",unsafe extern "C" fn(u32,*mut c_void,*const isize)->*mut c_void,0x31DD,std::ptr::null_mut(),std::ptr::null());
  assert_ne!(call!("eglInitialize",unsafe extern "C" fn(*mut c_void,*mut i32,*mut i32)->u32,d,std::ptr::null_mut(),std::ptr::null_mut()),0);
  assert_ne!(call!("eglBindAPI",unsafe extern "C" fn(u32)->u32,0x30A2),0);
  let attrs=[0x3040,8,0x3033,1,0x3024,8,0x3023,8,0x3022,8,0x3038];
  let mut config=std::ptr::null_mut(); let mut n=0;
  assert_ne!(call!("eglChooseConfig",unsafe extern "C" fn(*mut c_void,*const i32,*mut *mut c_void,i32,*mut i32)->u32,d,attrs.as_ptr(),&mut config,1,&mut n),0); assert!(n>0);
  let ca=[0x3098,3,0x30FB,3,0x30FD,1,0x3038];
  let ctx=call!("eglCreateContext",unsafe extern "C" fn(*mut c_void,*mut c_void,*mut c_void,*const i32)->*mut c_void,d,config,std::ptr::null_mut(),ca.as_ptr()); assert!(!ctx.is_null());
  assert_ne!(call!("eglMakeCurrent",unsafe extern "C" fn(*mut c_void,*mut c_void,*mut c_void,*mut c_void)->u32,d,std::ptr::null_mut(),std::ptr::null_mut(),ctx),0);
  Ok(Self{lib})
 } }
}
impl Surface for HeadlessSurface {
 fn make_current(&mut self)->Result<(),GfxError>{Ok(())}
 fn window_size(&self)->(u32,u32){(720,480)}
 fn swap(&mut self)->Result<(),GfxError>{Ok(())}
 fn proc_address(&self,name:&str)->*const c_void{unsafe{let name=CString::new(name).unwrap();self.lib.get::<unsafe extern "C" fn(*const i8)->*const c_void>(b"eglGetProcAddress\0").unwrap()(name.as_ptr())}}
}
