#![allow(non_snake_case)]
use core::prelude::*;
use core::{mem, ptr, raw};
use core::str;
use collections::str::{StrAllocating, IntoMaybeOwned};
use collections::string::String;
use libc::{c_char, c_void, size_t};

use lua::lib::raw::*;
use lua::aux::raw::*;
use lua::raw::*;
use luajit::*;
use luajit_constants::*;

use glcommon::{GLResult, MString};
use log::{logi, loge};

use lua_callbacks::LuaCallback;

use lua_geom::SandboxMode::{Sandboxed, Unsandboxed};
use lua_geom::LuaValue::{RegistryValue, IndexValue};

static mut GLDRAW_LUA_CREATE_SANDBOX: *mut c_void = 0 as *mut c_void;
static mut GLDRAW_LUA_STOPFNS: *mut c_void = 0 as *mut c_void;
static mut gldraw_lua_key: i32 = 0;
static LUA_FFI_SCRIPT: &'static str = include_str!("../includes/lua/ffi_loader.lua");
static LUA_RUNNER: &'static str = include_str!("../includes/lua/lua_runner.lua");
static LUA_INTERPOLATOR_DEFAULTS: &'static str = include_str!("../includes/lua/init_defaults.lua");

static mut STATIC_LUA: Option<*mut lua_State> = None;

macro_rules! assert_stacksize {
    ($L:expr, $body:expr) => (
        {
            let stacksize = lua_gettop($L);
            let result = $body;
            assert_eq!(stacksize, lua_gettop($L));
            result
        }
    )
}

macro_rules! safe_pop {
    ($L:expr, $body:expr) => (
        {
            assert!(lua_gettop($L) as u32 >= $body as u32);
            lua_pop($L, $body);
        }
    )
}

type ReaderState<'a> = (&'a str, bool);

enum SandboxMode {
    Sandboxed(LuaValue),
    Unsandboxed,
}

enum LuaValue {
    #[allow(dead_code)]
    RegistryValue(*mut c_void),
    IndexValue(i32),
}

impl LuaValue {
    unsafe fn push_self(&self, L: *mut lua_State) {
        match self {
            &RegistryValue(key) => {
                lua_pushlightuserdata(L, key);
                lua_gettable(L, LUA_REGISTRYINDEX);
            },
            &IndexValue(idx) => {
                lua_pushvalue(L, idx);
            },
        }
    }
}

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
    safe_pop!(L, 1);
    result
}

unsafe fn runstring(L: *mut lua_State, s: &str, filename: *const i8, env: SandboxMode) -> bool {
    let mut state: ReaderState = (s, false);
    let stateref: *mut c_void = mem::transmute(&mut state);
    if 0 != lua_load(L, stringreader, stateref, filename) {
        false
        //Err(format!("script failed to load: {}", err_to_str(L)))
    } else {
        if let Sandboxed(key) = env {
            key.push_self(L);
            lua_setfenv(L, -2);
        }
        let result = lua_pcall(L, 0, 0, 0);
        if 0 != result {
            false
            //Err(format!("script failed to run: {}", err_to_str(L)));
        } else {
            true
            //Ok(())
        }
    }
}

static mut LUA_ORIGINAL_PANICFN: *mut c_void = 0 as *mut c_void;

#[no_mangle]
unsafe extern "C" fn panic_wrapper(L: *mut lua_State) -> i32 {
    loge!("inside lua panic handler!");
    let errorcstr = lua_tostring(L, -1);
    let errorstr = if errorcstr.is_null() { "" } else { str::from_c_str(errorcstr) };
    loge!("error is {}", errorstr);
    let panicfn: lua_CFunction = mem::transmute(LUA_ORIGINAL_PANICFN);
    panicfn(L); // should never return
    -1
}

unsafe fn init_lua() -> GLResult<*mut lua_State> {
    GLDRAW_LUA_STOPFNS = &mut GLDRAW_LUA_STOPFNS as *mut *mut c_void as *mut c_void;

    let L = luaL_newstate();
    let stacksize = lua_gettop(L);
    luaL_openlibs(L);

    luaJIT_setmode(L, 0, LUAJIT_MODE_ENGINE as i32|LUAJIT_MODE_ON as i32);

    if runstring(L, LUA_FFI_SCRIPT, cstr!("built-in ffi init script"), Unsandboxed) {
        logi!("ffi init script loaded");
        lua_pushlightuserdata(L, GLDRAW_LUA_CREATE_SANDBOX);
        lua_getglobal(L, cstr!("create_sandbox"));
        lua_settable(L, LUA_REGISTRYINDEX);

        lua_pushlightuserdata(L, GLDRAW_LUA_STOPFNS);
        lua_newtable(L);
        lua_settable(L, LUA_REGISTRYINDEX);

        LUA_ORIGINAL_PANICFN = lua_atpanic(L, panic_wrapper) as *mut c_void;
        assert_eq!(stacksize, lua_gettop(L));
        Ok(L)
    } else {
        let err = format!("ffi init script failed to load: {}\nThis should never happen!", err_to_str(L)).into_maybe_owned();
        lua_close(L);
        log_err(err)
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

#[no_mangle]
pub unsafe fn rust_raise_lua_err(L: Option<*mut lua_State>, msg: &str) -> ! {
    let L = L.unwrap_or(get_existing_lua().unwrap());
    ::lua::raw::lua_pushlstring(L, msg.as_ptr() as *const i8, msg.len() as u32);
    ::lua::raw::lua_error(L);
    panic!("luaL_error() returned, this should never happen!");
}

#[inline(always)]
pub unsafe fn get_existing_lua() -> Option<*mut lua_State> {
    STATIC_LUA
}

pub unsafe fn get_existing_lua_or_err() -> GLResult<*mut lua_State> {
    match get_existing_lua() {
        Some(lua) => Ok(lua),
        None => Err("couldn't get lua state!".into_maybe_owned()),
    }
}

unsafe fn create_sandbox(L: *mut lua_State) {
    lua_pushlightuserdata(L, GLDRAW_LUA_CREATE_SANDBOX);
    lua_gettable(L, LUA_REGISTRYINDEX);
    lua_pcall(L, 0, 1, 0);
}

unsafe fn save_ondone(L: *mut lua_State, key: i32, sandbox: LuaValue) -> GLResult<()> {
    logi!("saving ondone() method");
    let stacksize = lua_gettop(L);
    sandbox.push_self(L); {
        lua_pushlightuserdata(L, GLDRAW_LUA_STOPFNS); {
            lua_gettable(L, LUA_REGISTRYINDEX);
            logi!("type of gldraw_lua_stopfns is {}", str::from_c_str(lua_typename(L, lua_type(L, -1))));
            // stack holds sandbox -- stopfns
            lua_pushlightuserdata(L, key as *mut c_void); {
                // stack holds sandbox -- stopfns -- stopidx
                lua_getfield(L, -3, cstr!("ondone")); {
                    // stack holds sandbox -- stopfns -- stopidx -- ondone
                    if !lua_isfunction(L, -1) {
                        safe_pop!(L, 4);
                        assert_eq!(stacksize, lua_gettop(L));
                        return log_err("ondone not defined.\nThis should never happen!".into_maybe_owned());
                    }
                    logi!("saving ondone method to 0x{:x} in gldraw_lua_stopfns", key);
                    lua_settable(L, -3);
                }
            }
            // stack holds sandbox -- stopfns
            safe_pop!(L, 2);
            assert_eq!(stacksize, lua_gettop(L));
            Ok(())
        }
    }
}

pub unsafe fn load_lua_script(script: &str) -> GLResult<i32> {
    let L = try!(get_lua());
    logi!("got lua");
    lua_pushstring(L, cstr!("assert filler")); // lua_pop() on an empty stack is a no-op
    lua_pushstring(L, cstr!("assert filler"));
    lua_pushstring(L, cstr!("assert filler"));
    let stacksize = lua_gettop(L);
    let result = load_lua_script_internal(L, script);
    assert_eq!(stacksize, lua_gettop(L));
    safe_pop!(L, 3);
    result
}


unsafe fn load_lua_script_internal(L: *mut lua_State, script: &str) -> GLResult<i32> {
    let stacksize = lua_gettop(L);

    let key = (&gldraw_lua_key) as *const i32 as i32 + gldraw_lua_key;

    create_sandbox(L); {
        let sandbox_idx = IndexValue(lua_gettop(L));
        if !runstring(L, LUA_INTERPOLATOR_DEFAULTS, cstr!("interpolator defaults"), Sandboxed(sandbox_idx)) {
            let err = format!("default loader failed to load: {}\nThis should never happen!", err_to_str(L)).into_maybe_owned();
            safe_pop!(L, 1);
            assert_eq!(stacksize, lua_gettop(L));
            return log_err(err);
        }

        lua_pushlightuserdata(L, key as *mut c_void); {

            if !runstring(L, script, cstr!("interpolator script"), Sandboxed(sandbox_idx)) {
                let err = format!("script failed to load: {}", err_to_str(L)).into_maybe_owned();
                safe_pop!(L, 2);
                assert_eq!(stacksize, lua_gettop(L));
                return log_err(err);
            }

            sandbox_idx.push_self(L); {
                lua_getfield(L, -1, cstr!("main")); {
                    if !lua_isfunction(L, -1) {
                        safe_pop!(L, 4);
                        assert_eq!(stacksize, lua_gettop(L));
                        return log_err("no main function defined :(".into_maybe_owned());
                    }
                    luaJIT_setmode(L, 0, LUAJIT_MODE_ENGINE as i32|LUAJIT_MODE_ON as i32);
                    safe_pop!(L, 1);
                }

                // make values defined in script available to lua_runner
                lua_setglobal(L, cstr!("callbacks"));
            }

            // FIXME compile runner once
            if !runstring(L, LUA_RUNNER, cstr!("built-in lua_runner script"), Unsandboxed) {
                let err = format!("lua runner failed to load: {}\n This should never happen!", err_to_str(L)).into_maybe_owned();
                safe_pop!(L, 2);
                assert_eq!(stacksize, lua_gettop(L));
                return log_err(err);
            }

            lua_getglobal(L, cstr!("runmain")); {
                if !lua_isfunction(L, -1) {
                    safe_pop!(L, 3);
                    assert_eq!(stacksize, lua_gettop(L));
                    return log_err("runmain not defined.\nThis should never happen!".into_maybe_owned());
                }
                luaJIT_setmode(L, 0, LUAJIT_MODE_ENGINE as i32|LUAJIT_MODE_ON as i32);

                if let Err(msg) = save_ondone(L, key, sandbox_idx) {
                    safe_pop!(L, 3);
                    assert_eq!(stacksize, lua_gettop(L));
                    return Err(msg);
                }

                lua_settable(L, LUA_REGISTRYINDEX);
            }
        }

        safe_pop!(L, 1);
    }

    gldraw_lua_key += 1;
    logi!("created script for {}", script);
    assert_eq!(stacksize, lua_gettop(L));
    Ok(key)
}

pub unsafe fn finish_lua_script<T: LuaCallback>(output: &mut T, script: &::luascript::LuaScript) -> GLResult<()> {
    let L = get_lua().unwrap();
    lua_pushstring(L, cstr!("assert filler")); // lua_pop() on an empty stack is a no-op
    lua_pushstring(L, cstr!("assert filler"));
    lua_pushstring(L, cstr!("assert filler"));
    let stacksize = lua_gettop(L);
    lua_pushlightuserdata(L, GLDRAW_LUA_STOPFNS);
    let result = {
        lua_gettable(L, LUA_REGISTRYINDEX);
        logi!("type of gldraw_lua_stopfns is {}", str::from_c_str(lua_typename(L, lua_type(L, -1))));

        // stack is stopfns
        script.push_self();
        lua_gettable(L, -2);
        logi!("type of stopfn for {} is {}", script, str::from_c_str(lua_typename(L, lua_type(L, -1))));
        // stack is stopfns -- stopfn
        lua_pushlightuserdata(L, output as *mut T as *mut c_void);
        logi!("calling lua ondone()");
        let result = match lua_pcall(L, 1, 0, 0) {
            0 => Ok(()),
            _ => {
                logi!("finish_lua_script failed, stack size is {}", lua_gettop(L));
                log_err(format!("ondone() script failed to run: {}", err_to_str(L)).into_maybe_owned())
            }
        };
        safe_pop!(L, 1);
        result
    };
    assert_eq!(stacksize, lua_gettop(L));
    safe_pop!(L, 3);
    result
}

pub unsafe fn destroy_lua_script(key: i32) {
    let L = get_lua().unwrap();
    let stacksize = lua_gettop(L);
    lua_pushlightuserdata(L, key as *mut c_void);
    lua_pushnil(L);
    lua_settable(L, LUA_REGISTRYINDEX);
    assert_eq!(stacksize, lua_gettop(L));
}

fn log_err<T>(message: MString) -> GLResult<T> {
    loge(message.as_slice());
    Err(message)
}

#[inline]
pub unsafe fn push_lua_script(key: i32) {
    let L = get_lua().unwrap();
    lua_pushlightuserdata(L, key as *mut c_void);
}

pub unsafe fn do_interpolate_lua<T: LuaCallback>(script: &::luascript::LuaScript, dimensions: (i32, i32), output: &mut T) -> GLResult<()> {
    let L = try!(get_existing_lua_or_err());
    lua_pushstring(L, cstr!("assert filler")); // lua_pop() on an empty stack is a no-op
    lua_pushstring(L, cstr!("assert filler"));
    lua_pushstring(L, cstr!("assert filler"));
    let stacksize = lua_gettop(L);
    script.push_self();
    lua_gettable(L, LUA_REGISTRYINDEX);

    let (x, y) = dimensions;
    lua_pushnumber(L, x as f64);
    lua_pushnumber(L, y as f64);
    lua_pushlightuserdata(L, output as *mut T as *mut c_void);

    let result = match lua_pcall(L, 3, 0, 0) {
        0 => Ok(()),
        _ => log_err(format!("script failed to run: {}", err_to_str(L)).into_maybe_owned()),
    };
    assert_eq!(stacksize, lua_gettop(L));
    safe_pop!(L, 3);
    result
}
