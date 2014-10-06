use core::prelude::*;
use core::{ptr, fmt};
use core::fmt::Show;
use collections::str::StrAllocating;
use log::logi;
use glcommon::GLResult;

extern "C" {
    fn loadLuaScript(script: *const u8, len: i32) -> i32;
    fn unloadLuaScript(key: i32) -> ();
    fn useLuaScript(key: i32) -> ();
}

pub struct LuaScript {
    registry_id: i32,
}

impl LuaScript {
    pub fn new(script: Option<&str>) -> GLResult<LuaScript> {
        let (ptr, len) = script.map_or((ptr::null(), 0), |x| (x.as_bytes().as_ptr(), x.as_bytes().len()));
        match unsafe { loadLuaScript(ptr, len as i32) } {
            -1 => Err("something went wrong loading the script!".into_string()),
            x  => {
                let script = LuaScript { registry_id: x };
                logi!("created {}", script);
                Ok(script)
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
