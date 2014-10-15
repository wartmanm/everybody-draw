use core::prelude::*;
use core::mem;
use collections::vec::Vec;
use collections::MutableSeq;

use android::log::{ANDROID_LOG_INFO, __android_log_write};
use libc::{c_char, c_int};
use point::ShaderPaintPoint;
use point::{Move, Down, Up, NoEvent};
use glpoint;
use glpoint::MotionEventConsumer;
use drawevent::Events;
use glinit::GLInit;

static MOVE: u8 = 0u8;
static DONE: u8 = 1u8;
static DOWN: u8 = 2u8;
static UP:   u8 = 3u8;

pub struct LuaCallbackType<'a, 'b, 'c, 'd: 'b> {
    consumer: &'a mut MotionEventConsumer,
    events: &'b mut Events<'d>,
    drawvecs: &'c mut [Vec<ShaderPaintPoint>],
}

impl<'a, 'b, 'c, 'd> LuaCallbackType<'a, 'b, 'c, 'd> {
    pub fn new<'e: 'b+'c>(glinit: &'e mut GLInit<'d>, s: &'a mut MotionEventConsumer) -> LuaCallbackType<'a, 'b, 'c, 'd> {
        LuaCallbackType {
            consumer: s,
            events: &mut glinit.events,
            drawvecs: glinit.points.as_mut_slice(),
        }
    }
}

#[no_mangle]
pub extern "C" fn lua_nextpoint(data: &mut LuaCallbackType, points: &mut (ShaderPaintPoint, ShaderPaintPoint)) -> u16 {
    let (state, pointer) = glpoint::next_point(data.consumer, data.events);
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
    data.drawvecs[queue as uint].push(*point);
}

#[no_mangle]
pub unsafe extern "C" fn lua_pushline(data: &mut LuaCallbackType, queue: i32, a: *const ShaderPaintPoint, b: *const ShaderPaintPoint) {
    glpoint::push_line(&mut data.drawvecs[queue as uint], &*a, &*b);
}

#[no_mangle]
pub unsafe extern "C" fn lua_log(message: *const c_char) {
    __android_log_write(ANDROID_LOG_INFO as c_int, cstr!("luascript"), message);
}
