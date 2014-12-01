#![macro_escape]

#[allow(unused)] use core::prelude::*;
use android::log::*;
use libc::{c_char, c_int};
use collections::str::from_c_str;

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
    __android_log_write(level as ::libc::c_int, cstr!("rust"), msg.as_ptr() as *const c_char);
  }
}

#[cfg(not(target_os = "android"))]
pub fn log(rustmsg: &str, _: u32) {
    println!("{}", rustmsg);
}

pub fn loge(rustmsg: &str) {
    log(rustmsg, ANDROID_LOG_ERROR);
}

pub fn logi(rustmsg: &str) {
    log(rustmsg, ANDROID_LOG_INFO);
}

// macros that define entire macro bodies don't seem to be allowed yet
pub macro_rules! logi(
  ($fmt:expr, $($arg:expr),+) => (
    logi(format!($fmt, $($arg, )+).as_slice());
    );
  ($fmt:expr) => (
      logi($fmt);
      )
  )

pub macro_rules! loge(
  ($fmt:expr, $($arg:expr),+) => (
    loge(format!($fmt, $($arg, )+).as_slice());
    );
  ($fmt:expr) => (
      loge($fmt);
      )
  )
