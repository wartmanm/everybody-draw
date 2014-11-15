use core::prelude::*;
use core::mem;
use collections::vec::Vec;

use android::log::{ANDROID_LOG_INFO, __android_log_write};
use libc::{c_char, c_int};
use point::ShaderPaintPoint;
use point::{Move, Down, Up, NoEvent};
use glpoint;
use glpoint::MotionEventConsumer;
use glinit::GLInit;
use drawevent::Events;
use log::loge;
use lua_geom::rust_raise_lua_err;
use rustjni::JNICallbackClosure;

static MOVE: u8 = 0u8;
static DONE: u8 = 1u8;
static DOWN: u8 = 2u8;
static UP:   u8 = 3u8;

//pub type UndoCallback<'a> = ::rustjni::JNICallbackClosure<'a>;

pub struct LuaCallbackType<'a, 'b, 'c: 'b, 'd> {
    consumer: &'a mut MotionEventConsumer,
    events: &'c mut Events<'c>,
    glinit: &'b mut GLInit<'c>,
    undo_callback: &'d JNICallbackClosure<'d>,
}

pub trait LuaCallback { }
impl<'a,'b,'c,'d> LuaCallback for LuaCallbackType<'a,'b,'c,'d> { }

impl<'a, 'b, 'c, 'd> LuaCallbackType<'a, 'b, 'c, 'd> {
    pub fn new(glinit: &'b mut GLInit<'c>, events: &'c mut Events<'c>, s: &'a mut MotionEventConsumer, undo_callback: &'d JNICallbackClosure) -> LuaCallbackType<'a, 'b, 'c, 'd> {
        LuaCallbackType {
            consumer: s,
            events: events,
            glinit: glinit,
            undo_callback: undo_callback,
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

macro_rules! rust_raise_lua_err(
    ($L:expr, $fmt:expr, $($arg:tt)*) => ({
        let formatted = format!(concat!($fmt, "\0"), $($arg)*);
        rust_raise_lua_err($L, formatted.as_slice().as_ptr() as *const ::libc::c_char);
    })
)

fn get_queue_or_raise_err<'a, 'b, 'c, 'd>(data: &'d mut LuaCallbackType, queue: i32) -> &'d mut Vec<ShaderPaintPoint> {
    let points = &mut data.glinit.points;
    if (queue as uint) >= points.len() {
        unsafe {
            loge!("tried to push point to queue {} of {}", queue + 1, points.len());
            rust_raise_lua_err!(None, "tried to push point to queue {} of {}", queue + 1, points.len());
        }
    }
    unsafe { points.as_mut_slice().unsafe_mut(queue as uint) }
}

#[no_mangle]
pub unsafe extern "C" fn lua_pushpoint(data: &mut LuaCallbackType, queue: i32, point: *const ShaderPaintPoint) {
    let points = get_queue_or_raise_err(data, queue);
    glpoint::push_point(points, &*point);
}

#[no_mangle]
pub unsafe extern "C" fn lua_pushline(data: &mut LuaCallbackType, queue: i32, a: *const ShaderPaintPoint, b: *const ShaderPaintPoint) {
    let points = get_queue_or_raise_err(data, queue);
    glpoint::push_line(points, &*a, &*b);
}

#[no_mangle]
pub unsafe extern "C" fn lua_log(message: *const c_char) {
    __android_log_write(ANDROID_LOG_INFO as c_int, cstr!("luascript"), message);
}

#[no_mangle]
pub unsafe extern "C" fn lua_clearlayer(data: &mut LuaCallbackType, layer: i32) {
    if let Err(mut msg) = data.glinit.erase_layer(layer) {
        loge!(msg.as_slice());
        msg.push('\0');
        rust_raise_lua_err(None, msg.as_slice().as_ptr() as *const i8);
    }
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

#[no_mangle]
pub unsafe extern "C" fn lua_saveundobuffer(data: &mut LuaCallbackType) -> () {
    let result = data.glinit.push_undo_frame();
    (*data.undo_callback)(result);
}


