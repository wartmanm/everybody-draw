use core::prelude::*;
use core::{mem, ptr, raw};
use collections::str::StrAllocating;
use collections::string::String;
use libc::{c_char, c_void, size_t};

use lua::lib::raw::*;
use lua::aux::raw::*;
use lua::raw::*;
use luajit::*;
use luajit_constants::*;

use glcommon::GLResult;
use log::{logi,loge};

static mut gldraw_lua_key: i32 = 0;
static lua_ffi_script: &'static str = "
ffi = require(\"ffi\")
ffi.cdef[[
  struct ShaderPaintPoint {
    float x;
    float y;
    float time;
    float size;
    float speed;
    float distance;
    float counter;
  };

  void pushrustvec(void *output, int queue, struct ShaderPaintPoint *point);
  char next_point_from_lua(void *output, struct ShaderPaintPoint *points);
  void loglua(const char *message);

]]

pushpoint=ffi.C.pushrustvec
loglua=ffi.C.loglua
ShaderPaintPoint=ffi.typeof(\"struct ShaderPaintPoint\")";

static lua_runner: &'static str = "
local _main = main
local _onframe = onframe
if type(main) ~= \"function\" then
  loglua(\"main not defined for runmain()!!\")
  return
end

function runmain(x, y, output)
  if type(_onframe) == \"function\" then
    onframe(x, y, output)
  end
  if type(_main) ~= \"function\" then
    loglua(\"main doesn't exist!!\")
    return
  end
  local pointpair = ffi.new(\"struct ShaderPaintPoint[2]\")
  while ffi.C.next_point_from_lua(output, pointpair) ~= 0 do
    _main(pointpair[0], pointpair[1], x, y, output)
  end
end";

static default_script: &'static str = "
function main(a, b, x, y, points)
  pushpoint(points, 0, a)
  pushpoint(points, 0, b)
end";

static mut STATIC_LUA: Option<*mut lua_State> = None;

type ReaderState<'a> = (&'a str, bool);

extern "C" fn stringreader(L: *mut lua_State, data: *mut c_void, size: *mut size_t) -> *const c_char {
    unsafe {
        let state: &mut ReaderState = mem::transmute(data);
        let (ref chars, ref mut done) = *state;
        if *done {
            ptr::null()
        } else {
            *done = true;
            *size = chars.len() as u32;
            chars.as_ptr() as *const i8
        }
    }
}

unsafe fn err_to_str(L: *mut lua_State) -> String {
    let mut size: size_t = 0;
    let strptr = lua_tolstring(L, -1, &mut size);
    let luastr: &str = mem::transmute(raw::Slice { data: strptr, len: size as uint });
    let result = luastr.into_string();
    lua_pop(L, -1);
    result
}

unsafe fn runstring(L: *mut lua_State, s: &str) -> bool {
    let mut state: ReaderState = (s, false);
    let stateref: *mut c_void = mem::transmute(&mut state);
    if 0 != lua_load(L, stringreader, stateref, cstr!("loadLuaScript() input")) {
        false
        //Err(format!("script failed to load: {}", err_to_str(L)))
    } else if 0 != lua_pcall(L, 0, MULTRET, 0) {
        false
        //Err(format!("script failed to run: {}", err_to_str(L)));
    } else {
        true
        //Ok(())
    }
}

unsafe fn init_lua() -> GLResult<*mut lua_State> {
    let L = luaL_newstate();
    luaL_openlibs(L);

    luaJIT_setmode(L, 0, LUAJIT_MODE_ENGINE as i32|LUAJIT_MODE_ON as i32);

    if runstring(L, lua_ffi_script) {
        logi!("ffi init script loaded");
        Ok(L)
    } else {
        lua_close(L);
        Err(format!("ffi init script failed to load: {}", err_to_str(L)))
    }
}

unsafe fn get_lua() -> GLResult<*mut lua_State> {
    match STATIC_LUA {
        Some(x) => Ok(x),
        None => {
            STATIC_LUA = Some(try!(init_lua()));
            get_lua()
        }
    }
}

pub unsafe fn load_lua_script(script: Option<&str>) -> GLResult<i32> {
    let L = try!(get_lua());
    logi!("got lua");

    lua_pushnil(L);
    lua_setglobal(L, cstr!("main"));
    lua_pushnil(L);
    lua_setglobal(L, cstr!("onframe"));

    let key = (&gldraw_lua_key) as *const i32 as i32 + gldraw_lua_key;
    lua_pushlightuserdata(L, key as *mut c_void);

    let script = script.unwrap_or(default_script);
    if !runstring(L, script) {
        let err = Err(format!("script failed to load: {}", err_to_str(L)));
        lua_pop(L, 1);
        return err;
    }

    lua_getglobal(L, cstr!("main"));
    if !lua_isfunction(L, -1) {
        lua_pop(L, 2);
        return Err("no main function defined :(".into_string());
    }
    luaJIT_setmode(L, 0, LUAJIT_MODE_ENGINE as i32|LUAJIT_MODE_ON as i32);
    lua_pop(L, 1);

    // FIXME compile runner once
    if !runstring(L, lua_runner) {
        let err = Err(format!("lua runner failed to load: {}", err_to_str(L)));
        lua_pop(L, 1);
        return err;
    }

    lua_getglobal(L, cstr!("runmain"));
    if !lua_isfunction(L, -1) {
        lua_pop(L, 2);
        return Err("runmain not defined :(".into_string());
    }
    luaJIT_setmode(L, 0, LUAJIT_MODE_ENGINE as i32|LUAJIT_MODE_ON as i32);

    lua_settable(L, LUA_REGISTRYINDEX);
    gldraw_lua_key += 1;
    logi!("created script for {}", script);
    Ok(key)
}

pub unsafe fn unload_lua_script(key: i32) {
    let L = get_lua().unwrap();
    lua_pushlightuserdata(L, key as *mut c_void);
    lua_pushnil(L);
    lua_settable(L, LUA_REGISTRYINDEX);
}

pub unsafe fn use_lua_script(key: i32) {
    let L = get_lua().unwrap();
    lua_pushlightuserdata(L, key as *mut c_void);
    lua_gettable(L, LUA_REGISTRYINDEX);
    lua_setglobal(L, cstr!("runmain"));
}

unsafe fn interpolate_lua(L: *mut lua_State, x: i32, y: i32, output: *mut c_void) -> GLResult<()> {
    lua_getglobal(L, cstr!("runmain"));
    
    lua_pushnumber(L, x as f64);
    lua_pushnumber(L, y as f64);
    lua_pushlightuserdata(L, output);

    if lua_pcall(L, 3, 0, 0) != 0 {
        Err(format!("script failed to run: {}", err_to_str(L)))
    } else {
        Ok(())
    }
}

pub unsafe fn do_interpolate_lua(x: i32, y: i32, output: *mut c_void) {
    let L = STATIC_LUA;
    if let Some(L) = STATIC_LUA {
        interpolate_lua(L, x, y, output);
    } else {
        logi!("lua state doesn't exist!");
    }
}
