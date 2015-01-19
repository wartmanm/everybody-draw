use core::prelude::*;
use core::fmt;
use core::fmt::Show;
use log::logi;
use glcommon::{GLResult, FillDefaults, Defaults, MString};
use lua_geom::{load_lua_script, destroy_lua_script, push_lua_script};
use collections::str::{IntoMaybeOwned};

static DEFAULT_SCRIPT: &'static str = include_str!("../includes/lua/default_interpolator.lua");

pub struct LuaScript {
    pub registry_id: i32,
    pub source: MString,
}

impl LuaScript {
    pub fn new(source: MString) -> GLResult<LuaScript> {
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

impl FillDefaults<Option<MString>, MString, LuaScript> for LuaScript {
    fn fill_defaults(init: Option<MString>) -> Defaults<MString, LuaScript> {
        Defaults { val: init.unwrap_or_else(|| DEFAULT_SCRIPT.into_maybe_owned()) }
    }
}

