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

pub fn raw_loge(rustmsg: *const c_char) {
    unsafe {
        raw_log(ANDROID_LOG_ERROR as i32, cstr!("rust"), rustmsg);
    }
}

pub fn raw_logi(rustmsg: *const c_char) {
    unsafe {
        raw_log(ANDROID_LOG_INFO as i32, cstr!("rust"), rustmsg);
    }
}

// macros that define entire macro bodies don't seem to be allowed yet
pub macro_rules! logi(
    ($fmt:expr, $($arg:expr),+) => (
        ::log::raw_logi(format!(concat!($fmt, "\0"), $($arg, )+).as_slice().as_ptr() as *const ::libc::c_char);
    );
    ($fmt:expr) => (
        ::log::raw_logi(concat!($fmt, "\0").as_ptr() as *const ::libc::c_char);
    )
);

pub macro_rules! loge(
    ($fmt:expr, $($arg:expr),+) => (
        ::log::raw_loge(format!(concat!($fmt, "\0"), $($arg, )+).as_slice().as_ptr() as *const ::libc::c_char);
    );
    ($fmt:expr) => (
        ::log::raw_loge(concat!($fmt, "\0").as_ptr() as *const ::libc::c_char);
    )
);
