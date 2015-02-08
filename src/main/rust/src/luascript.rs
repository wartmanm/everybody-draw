use core::prelude::*;
use core::fmt;
use core::fmt::Debug;
use glcommon::{GLResult, UsingDefaults, MString};
use lua_geom::{load_lua_script, destroy_lua_script};
use core::borrow::IntoCow;

static DEFAULT_SCRIPT: &'static str = include_str!("../includes/lua/default_interpolator.lua");

pub struct LuaScript {
    pub registry_id: i32,
    pub source: MString,
}

impl LuaScript {
    pub fn new(source: MString) -> GLResult<LuaScript> {
        let registry_id = unsafe { try!(load_lua_script(source.as_slice())) };
        let script = LuaScript { registry_id: registry_id, source: source };
        debug_logi!("created {:?}", script);
        Ok(script)
    }

    #[inline]
    pub fn get_key(&self) -> i32 {
        self.registry_id
    }
}

impl Drop for LuaScript {
    fn drop(&mut self) {
        debug_logi!("dropping {:?}", self);
        unsafe {
            destroy_lua_script(self.registry_id);
        }
    }
}

impl Debug for LuaScript {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "lua script 0x{:x}", self.registry_id)
    }
}

impl UsingDefaults<Option<MString>> for LuaScript {
    type Defaults = MString;
    fn maybe_init(script: Option<MString>) -> GLResult<LuaScript> {
        LuaScript::new(fill_defaults(script))
    }
    fn get_source(&self) -> &MString { &self.source }
}

fn fill_defaults(script: Option<MString>) -> MString {
    let script = script.unwrap_or_else(|| DEFAULT_SCRIPT.into_cow());
    script
}
