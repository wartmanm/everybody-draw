#![feature(globs, macro_rules, default_type_params, if_let)]
#![crate_name = "rustgl"]
#![crate_type = "staticlib"]
#![no_std]

extern crate libc;
extern crate opengles;
extern crate egl;
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

pub mod macros;
pub mod log;
pub mod glcommon;
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
