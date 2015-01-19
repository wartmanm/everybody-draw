use core::prelude::*;
use core::fmt;
use core::fmt::Show;
use log::logi;
use glcommon::GLResult;
use lua_geom::{load_lua_script, unload_lua_script, push_lua_script};

pub struct LuaScript {
    registry_id: i32,
}

impl LuaScript {
    pub fn new(script: Option<&str>) -> GLResult<LuaScript> {
        //let (ptr, len) = script.map_or((ptr::null(), 0), |x| (x.as_bytes().as_ptr(), x.as_bytes().len()));
        let registry_id = unsafe { try!(load_lua_script(script)) };
        let script = LuaScript { registry_id: registry_id };
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
            unload_lua_script(self.registry_id);
        }
    }
}

impl Show for LuaScript {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "lua script 0x{:x}", self.registry_id)
    }
}
