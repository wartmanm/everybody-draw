/// DrawObjectList interns shaders, brushes, and scripts, returning references that can be stored
/// in the event queue.
/// TODO: scripts
/// TODO: serialization
/// TODO: further backing store, caching init objs by sha1 or so
/// TODO: free shaders + textures on gl pause
/// TODO: cleanup, deduplication
///
/// more importantly, switch to a map and return keys, rather than using indices

use core::prelude::*;
use core::cell::UnsafeCell;
use collections::vec::Vec;
use collections::string::String;
use collections::MutableSeq;
use copyshader::CopyShader;
use gltexture::{PixelFormat, Texture};
use pointshader::PointShader;
use glcommon::Shader;
use luascript::LuaScript;

pub struct DrawObjectList<T, Init> {
    list: Vec<CachedInit<T, Init>>,
}

pub struct DrawObjectIndex<T>(i32);

pub type ShaderInitValues = (Option<String>, Option<String>);
pub type BrushInitValues = (PixelFormat, (i32, i32), Vec<u8>);
pub type ShaderInit<T> = CachedInit<Option<T>, ShaderInitValues>;
pub type BrushInit = CachedInit<Texture, BrushInitValues>;
pub type LuaInitValues = Option<String>;
pub type LuaInit = CachedInit<Option<LuaScript>, LuaInitValues>;

pub struct CachedInit<T, Init> {
    value: UnsafeCell<Option<T>>,
    init: Init,
}

pub trait InitFromCache<Init> {
    fn init(&Init) -> Self;
}

impl<T: InitFromCache<Init>, Init> CachedInit<T, Init> {
    pub fn get(&self) -> &T {
        match unsafe { &*self.value.get() } {
            &Some(ref x) => x,
            &None => {
                let value: T = InitFromCache::init(&self.init);
                unsafe { *self.value.get() = Some(value); }
                self.get()
            }
        }
    }
}

impl<T, Init> CachedInit<T, Init> {
    pub fn new(init: Init) -> CachedInit<T, Init> {
        CachedInit { value: UnsafeCell::new(None), init: init }
    }
}

// this and the two shader impls were originally a single
// impl<T: Shader> InitFromCache<ShaderInitValues> for Option<T>
// but that counts as the impl for all of Option, not just Option<Shader>
fn _init_copy_shader<T: Shader>(value: &(Option<String>, Option<String>)) -> Option<T> {
    let &(ref frag, ref vert) = value;
    let (vertopt, fragopt) = (vert.as_ref().map(|x|x.as_slice()), frag.as_ref().map(|x|x.as_slice()));
    Shader::new(fragopt, vertopt)
}
impl InitFromCache<ShaderInitValues> for Option<CopyShader> {
    fn init(value: &(Option<String>, Option<String>)) -> Option<CopyShader> { _init_copy_shader(value) }
}
impl InitFromCache<ShaderInitValues> for Option<PointShader> {
    fn init(value: &(Option<String>, Option<String>)) -> Option<PointShader> { _init_copy_shader(value) }
}

impl InitFromCache<BrushInitValues> for Texture {
    fn init(value: &BrushInitValues) -> Texture {
        let &(format, (w, h), ref pixels) = value;
        Texture::with_image(w, h, Some(pixels.as_slice()), format)
    }
}

impl InitFromCache<LuaInitValues> for Option<LuaScript> {
    fn init(value: &Option<String>) -> Option<LuaScript> {
        LuaScript::new(value.as_ref().map(|x|x.as_slice()))
    }
}

impl<T: InitFromCache<Init>, Init> DrawObjectList<T, Init> {
    pub fn new() -> DrawObjectList<T, Init> {
        DrawObjectList {
            list: Vec::new()
        }
    }

    pub fn push_object(&mut self, init: CachedInit<T, Init>) -> DrawObjectIndex<T> {
        self.list.push(init);
        DrawObjectIndex((self.list.len() -1) as i32)
    }

    pub fn get_object(&self, i: DrawObjectIndex<T>) -> &T {
        let DrawObjectIndex(idx) = i;
        self.list[idx as uint].get()
    }
}
