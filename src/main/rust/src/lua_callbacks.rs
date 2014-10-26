use core::prelude::*;
use core::mem;
use collections::MutableSeq;

use android::log::{ANDROID_LOG_INFO, __android_log_write};
use libc::{c_char, c_int};
use point::ShaderPaintPoint;
use point::{Move, Down, Up, NoEvent};
use glpoint;
use glpoint::MotionEventConsumer;
use glinit::GLInit;
use drawevent::Events;
use log::{logi};

static MOVE: u8 = 0u8;
static DONE: u8 = 1u8;
static DOWN: u8 = 2u8;
static UP:   u8 = 3u8;

pub struct LuaCallbackType<'a, 'b, 'c: 'b> {
    consumer: &'a mut MotionEventConsumer,
    events: &'c mut Events<'c>,
    glinit: &'b mut GLInit<'c>,
}

impl<'a, 'b, 'c> LuaCallbackType<'a, 'b, 'c> {
    pub fn new(glinit: &'b mut GLInit<'c>, events: &'c mut Events<'c>, s: &'a mut MotionEventConsumer) -> LuaCallbackType<'a, 'b, 'c> {
        LuaCallbackType {
            consumer: s,
            events: events,
            glinit: glinit,
        }
    }
}

#[no_mangle]
pub extern "C" fn lua_nextpoint(data: &mut LuaCallbackType, points: &mut (ShaderPaintPoint, ShaderPaintPoint)) -> u16 {
    let events: &mut Events = data.events;
    let (state, pointer) = glpoint::next_point(data.consumer, events);
    let (newpoints, luastate) = match state {
        Move(a, b) => ((a,b), MOVE),
        Down(a) => unsafe { ((a, mem::uninitialized()), DOWN) },
        Up => unsafe { (mem::uninitialized(), UP) },
        NoEvent => unsafe { (mem::uninitialized(), DONE) },
    };
    *points = newpoints;
    ((luastate as u16) << 8) | (pointer as u16)
}

#[no_mangle]
pub unsafe extern "C" fn lua_pushpoint(data: &mut LuaCallbackType, queue: i32, point: *const ShaderPaintPoint) {
    glpoint::push_point(&mut data.glinit.points.as_mut_slice()[queue as uint], &*point);
}

#[no_mangle]
pub unsafe extern "C" fn lua_pushline(data: &mut LuaCallbackType, queue: i32, a: *const ShaderPaintPoint, b: *const ShaderPaintPoint) {
    glpoint::push_line(&mut data.glinit.points.as_mut_slice()[queue as uint], &*a, &*b);
}

#[no_mangle]
pub unsafe extern "C" fn lua_log(message: *const c_char) {
    __android_log_write(ANDROID_LOG_INFO as c_int, cstr!("luascript"), message);
}

#[no_mangle]
pub unsafe extern "C" fn lua_clearlayer(data: &mut LuaCallbackType, layer: i32) {
    data.glinit.erase_layer(layer);
}

#[no_mangle]
pub unsafe extern "C" fn lua_savelayers(data: &mut LuaCallbackType) {
    data.glinit.copy_layers_down();
}

#[no_mangle]
pub unsafe extern "C" fn lua_pushcatmullrom(data: &mut LuaCallbackType, queue: i32, points: &[ShaderPaintPoint, ..4]) {
    glpoint::push_catmullrom(&mut data.glinit.points.as_mut_slice()[queue as uint], points);
}

#[no_mangle]
pub unsafe extern "C" fn lua_pushcubicbezier(data: &mut LuaCallbackType, queue: i32, points: &[ShaderPaintPoint, ..4]) {
    glpoint::push_cubicbezier(&mut data.glinit.points.as_mut_slice()[queue as uint], points);
}
