use core::prelude::*;
use core::{ptr, fmt};
use core::fmt::Show;
use log::{logi, loge};

extern "C" {
    fn loadLuaScript(script: *const u8) -> i32;
    fn unloadLuaScript(key: i32) -> ();
    fn useLuaScript(key: i32) -> ();
}

pub struct LuaScript {
    registry_id: i32,
}

impl LuaScript {
    pub fn new(script: Option<&str>) -> Option<LuaScript> {
        match unsafe { loadLuaScript(script.map_or(ptr::null(), |x| x.as_bytes().as_ptr())) } {
            -1 => None,
            x  => {
                let script = LuaScript { registry_id: x };
                logi!("created {}", script);
                Some(script)
            }
        }
    }

    pub fn prep(&self) {
        unsafe {
            useLuaScript(self.registry_id);
        }
    }
}

impl Drop for LuaScript {
    fn drop(&mut self) {
        logi!("dropping {}", self);
        unsafe {
            unloadLuaScript(self.registry_id);
        }
    }
}

impl Show for LuaScript {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "lua script 0x{:x}", self.registry_id)
    }
}
