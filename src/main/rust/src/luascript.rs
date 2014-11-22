use core::prelude::*;
use core::fmt;
use core::fmt::Show;
use log::logi;
use glcommon::{GLResult, FillDefaults, Defaults};
use lua_geom::{load_lua_script, destroy_lua_script, push_lua_script};
use collections::str::{MaybeOwned, IntoMaybeOwned, StrAllocating};
use collections::string::String;

static DEFAULT_SCRIPT: &'static str = include_str!("../includes/lua/default_interpolator.lua");

pub struct LuaScript {
    pub registry_id: i32,
    pub source: String,
    //pub source: MaybeOwned<'static>,
}

impl LuaScript {
    pub fn new(source: String) -> GLResult<LuaScript> {
        //let (ptr, len) = script.map_or((ptr::null(), 0), |x| (x.as_bytes().as_ptr(), x.as_bytes().len()));
        //let source = script.map(|x| x.into_maybe_owned())
        let registry_id = unsafe { try!(load_lua_script(source.as_slice())) };
        let script = LuaScript { registry_id: registry_id, source: source };
        logi!("created {}", script);
        Ok(script)
    }

    #[inline]
    pub fn push_self(&self) {
        unsafe {
            push_lua_script(self.registry_id);
        }
    }
}

impl Drop for LuaScript {
    fn drop(&mut self) {
        logi!("dropping {}", self);
        unsafe {
            destroy_lua_script(self.registry_id);
        }
    }
}

impl Show for LuaScript {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "lua script 0x{:x}", self.registry_id)
    }
}

impl FillDefaults<Option<String>, String, LuaScript> for LuaScript {
    fn fill_defaults(init: Option<String>) -> Defaults<String, LuaScript> {
        Defaults { val: init.unwrap_or_else(|| DEFAULT_SCRIPT.into_string()) }
    }
}

