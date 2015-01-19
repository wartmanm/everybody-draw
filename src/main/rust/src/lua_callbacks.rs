use core::prelude::*;
use core::mem;
use collections::MutableSeq;
use collections::vec::Vec;
use collections::str::StrAllocating;

use android::log::{ANDROID_LOG_INFO, __android_log_write};
use libc::{c_char, c_int};
use point::ShaderPaintPoint;
use point::{Move, Down, Up, NoEvent};
use glpoint;
use glpoint::MotionEventConsumer;
use glcommon::GLResult;
use glinit::GLInit;
use drawevent::Events;
use lua::raw::lua_State;

static MOVE: u8 = 0u8;
static DONE: u8 = 1u8;
static DOWN: u8 = 2u8;
static UP:   u8 = 3u8;

pub struct LuaCallbackType<'a, 'b, 'c: 'b> {
    consumer: &'a mut MotionEventConsumer,
    events: &'c mut Events<'c>,
    glinit: &'b mut GLInit<'c>,
    pub lua: *mut lua_State,
}

impl<'a, 'b, 'c> LuaCallbackType<'a, 'b, 'c> {
    pub fn new(glinit: &'b mut GLInit<'c>, events: &'c mut Events<'c>, s: &'a mut MotionEventConsumer) -> GLResult<LuaCallbackType<'a, 'b, 'c>> {
        match unsafe { ::lua_geom::get_existing_lua() } {
            Some(lua) => Ok(LuaCallbackType {
                consumer: s,
                events: events,
                glinit: glinit,
                lua: lua,
            }),
            None => Err("couldn't get lua state!".into_string()),
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
#[allow(non_snake_case)]
pub unsafe fn rust_raise_lua_err(L: *mut lua_State, msg: *const i8) -> ! {
    ::lua::aux::raw::luaL_error(L, msg);
    fail!("luaL_error() returned, this should never happen!");
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
            rust_raise_lua_err!(data.lua, "tried to push point to queue {} of {}", queue + 1, points.len());
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
        msg.push('\0');
        rust_raise_lua_err(data.lua, msg.as_slice().as_ptr() as *const i8);
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
