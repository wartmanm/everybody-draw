#![macro_escape]

use android::log::*;
use core::prelude::*;
use std::c_str::ToCStr;

pub fn log(rustmsg: &str, level: u32) {
  let msg = rustmsg.to_c_str();
  unsafe {
    __android_log_write(level as ::libc::c_int, cstr!("rust"), msg.as_ptr());
  }
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
