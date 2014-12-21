use core::prelude::*;
use core::fmt;
use core::fmt::Show;
use glcommon::{GLResult, FillDefaults, Defaults, MString};
use lua_geom::{load_lua_script, destroy_lua_script};
use core::borrow::IntoCow;

static DEFAULT_SCRIPT: &'static str = include_str!("../includes/lua/default_interpolator.lua");

pub struct LuaScript {
    pub registry_id: i32,
    pub source: (MString, (i32, i32)),
}

impl LuaScript {
    pub fn new(source: MString, dimensions: (i32, i32)) -> GLResult<LuaScript> {
        let registry_id = unsafe { try!(load_lua_script(source.as_slice(), dimensions)) };
        let script = LuaScript { registry_id: registry_id, source: (source, dimensions) };
        logi!("created {}", script);
        Ok(script)
    }

    #[inline]
    pub fn get_key(&self) -> i32 {
        self.registry_id
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

impl FillDefaults<(Option<MString>, (i32, i32)), (MString, (i32, i32)), LuaScript> for LuaScript {
    fn fill_defaults(init: (Option<MString>, (i32, i32))) -> Defaults<(MString, (i32, i32)), LuaScript> {
        let (script, dimensions) = init;
        Defaults { val: (script.unwrap_or_else(|| DEFAULT_SCRIPT.into_cow()), dimensions) }
    }
}

