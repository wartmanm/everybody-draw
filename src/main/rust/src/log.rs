#[allow(unused)] use core::prelude::*;
use android::log::*;
use libc::{c_char, c_int};
#[cfg(not(target_os = "android"))] use collections::str::from_c_str;

#[cfg(target_os = "android")]
pub unsafe fn raw_log(level: c_int, tag: *const c_char, msg: *const c_char) {
    __android_log_write(level, tag, msg);
}

#[cfg(not(target_os = "android"))]
pub unsafe fn raw_log(_: c_int, tag: *const c_char, msg: *const c_char) {
    println!("{}: {}", from_c_str(tag), from_c_str(msg));
}

#[cfg(target_os = "android")]
pub fn log(msg: &str, level: u32) {
  unsafe {
      let cmsg = format!("{}\0", msg);
      __android_log_write(level as ::libc::c_int, cstr!("rust"), cmsg.as_ptr() as *const c_char);
  }
}

#[cfg(not(target_os = "android"))]
pub fn log(rustmsg: &str, _: u32) {
    println!("{}", rustmsg);
}

pub macro_rules! debug_logi {
    ($($arg:tt)*) => (if cfg!(debug) { logi!($($arg)*); })
}

/// wrapper for unsafe calls to raw_log, don't use directly
pub fn _log(level: c_int, msg: *const c_char) {
    unsafe { raw_log(level, cstr!("everybody-draws"), msg); }
}

macro_rules! log(
    ($lvl:expr, $fmt:expr, $($arg:expr),+) => (
        ::log::_log($lvl, format!(concat!("native: ", $fmt, "\0"), $($arg, )+).as_slice().as_ptr() as *const ::libc::c_char);
    );
    ($lvl:expr, $fmt:expr) => (
        ::log::_log($lvl, concat!("native: ", $fmt, "\0").as_ptr() as *const ::libc::c_char);
    );
);

// macros that define entire macro bodies don't seem to be allowed yet
pub macro_rules! logi {
    ($($arg:tt)*) => ( log!(::android::log::ANDROID_LOG_INFO as i32, $($arg)*); )
}

pub macro_rules! loge {
    ($($arg:tt)*) => ( log!(::android::log::ANDROID_LOG_ERROR as i32, $($arg)*); )
}
