#![feature(unboxed_closures, unsafe_destructor)]
#![feature(no_std, core, hash, std_misc, rustc_private, alloc, collections, libc)]
#![crate_name = "rustgl"]
#![crate_type = "staticlib"]
#![no_std]

extern crate libc;
extern crate opengles;
extern crate egl;
//#[macro_use(format, write, println, try, assert, debug_assert, assert_eq, debug_assert_eq)]
extern crate core;
extern crate collections;
extern crate alloc;
extern crate std;
extern crate arena;
extern crate lua;

pub mod bindgen_builtins;
pub mod android;
pub mod jni;
pub mod jni_constants;
pub mod luajit;
pub mod luajit_constants;

#[macro_use]
pub mod macros;
#[macro_use]
pub mod log;
#[macro_use]
pub mod glcommon;
#[macro_use]
pub mod rollingaverage;

pub mod glpoint;
pub mod activestate;
pub mod motionevent;
pub mod pointshader;
pub mod glinit;
pub mod eglinit;
pub mod copyshader;
pub mod gltexture;
pub mod point;
pub mod matrix;
pub mod drawevent;
pub mod glstore;
pub mod luascript;
pub mod paintlayer;
pub mod rustjni;
pub mod lua_geom;
pub mod lua_callbacks;
pub mod jni_helpers;
