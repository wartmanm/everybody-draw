use core::prelude::*;
use core::fmt;
use core::fmt::Show;
use log::logi;
use glcommon::GLResult;
use lua_geom::{load_lua_script, destroy_lua_script, push_lua_script};
use collections::str::{MaybeOwned, IntoMaybeOwned};

static DEFAULT_SCRIPT: &'static str = include_str!("../includes/lua/default_interpolator.lua");

pub struct LuaScript {
    pub registry_id: i32,
    pub source: String,
    //pub source: MaybeOwned<'static>,
}

impl LuaScript {
    pub fn new(script: Option<String>) -> GLResult<LuaScript> {
        //let (ptr, len) = script.map_or((ptr::null(), 0), |x| (x.as_bytes().as_ptr(), x.as_bytes().len()));
        //let source = script.map(|x| x.into_maybe_owned())
        let source = script.unwrap_or_else(|| DEFAULT_SCRIPT.to_string());
        let registry_id = unsafe { try!(load_lua_script(script.as_slice())) };
        let script = LuaScript { registry_id: registry_id, source: script };
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
