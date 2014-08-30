#![feature(globs, macro_rules)]
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

pub mod log;

pub mod macros;
pub mod glcommon;
pub mod rollingaverage;

pub mod glpoint;
pub mod activestate;
pub mod motionevent;
pub mod android;
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
