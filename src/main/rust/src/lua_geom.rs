#![allow(non_snake_case)]
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
use log::logi;

use lua_callbacks::LuaCallbackType;

static mut gldraw_lua_key: i32 = 0;
static LUA_FFI_SCRIPT: &'static str = include_str!("../includes/lua/ffi_loader.lua");
static LUA_RUNNER: &'static str = include_str!("../includes/lua/lua_runner.lua");
static DEFAULT_SCRIPT: &'static str = include_str!("../includes/lua/default_interpolator.lua");

static mut STATIC_LUA: Option<*mut lua_State> = None;

type ReaderState<'a> = (&'a str, bool);

#[allow(unused)]
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

    if runstring(L, LUA_FFI_SCRIPT) {
        logi!("ffi init script loaded");
        Ok(L)
    } else {
        lua_close(L);
        Err(format!("ffi init script failed to load: {}\nThis should never happen!", err_to_str(L)))
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

    let script = script.unwrap_or(DEFAULT_SCRIPT);
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
    if !runstring(L, LUA_RUNNER) {
        let err = Err(format!("lua runner failed to load: {}\n This should never happen!", err_to_str(L)));
        lua_pop(L, 1);
        return err;
    }

    lua_getglobal(L, cstr!("runmain"));
    if !lua_isfunction(L, -1) {
        lua_pop(L, 2);
        return Err("runmain not defined.\n  This should never happen!".into_string());
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

pub unsafe fn do_interpolate_lua(dimensions: (i32, i32), output: &mut LuaCallbackType) -> GLResult<()> {
    if let Some(L) = STATIC_LUA {
        lua_getglobal(L, cstr!("runmain"));
        
        let (x, y) = dimensions;
        lua_pushnumber(L, x as f64);
        lua_pushnumber(L, y as f64);
        lua_pushlightuserdata(L, output as *mut LuaCallbackType as *mut c_void);

        match lua_pcall(L, 3, 0, 0) {
            0 => Ok(()),
            _ => Err(format!("script failed to run: {}", err_to_str(L))),
        }
    } else {
        logi!("lua state doesn't exist!");
        Ok(())
    }
}
