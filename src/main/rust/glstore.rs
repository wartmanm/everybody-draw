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
impl<T> PartialEq for DrawObjectIndex<T> {
    fn eq(&self, other: &DrawObjectIndex<T>) -> bool {
        let (DrawObjectIndex(a), DrawObjectIndex(b)) = (*self, *other);
        a == b
    }
}


pub type ShaderInitValues = (Option<String>, Option<String>);
pub type BrushInitValues = (PixelFormat, (i32, i32), Vec<u8>);
pub type ShaderInit<T> = CachedInit<T, ShaderInitValues>;
pub type BrushInit = CachedInit<Texture, BrushInitValues>;
pub type LuaInitValues = Option<String>;
pub type LuaInit = CachedInit<LuaScript, LuaInitValues>;

pub struct CachedInit<T, Init> {
    value: UnsafeCell<Option<T>>,
    init: Init,
}

pub trait InitFromCache<Init> {
    fn init(&Init) -> Self;
}
pub trait MaybeInitFromCache<Init> {
    fn maybe_init(&Init) -> Option<Self>;
}

impl<T: MaybeInitFromCache<Init>, Init> CachedInit<T, Init> {
    pub fn get(&self) -> &T {
        match unsafe { &*self.value.get() } {
            &Some(ref x) => x,
            &None => {
                let value: Option<T> = MaybeInitFromCache::maybe_init(&self.init);
                unsafe { *self.value.get() = Some(value).unwrap(); }
                self.get()
            }
        }
    }
    pub fn new(init: Init) -> Option<CachedInit<T, Init>> {
        let value: Option<T> = MaybeInitFromCache::maybe_init(&init);
        match value {
            Some(v) => Some(CachedInit { value: UnsafeCell::new(Some(v)), init: init }),
            None => None
        }
    }
}

trait RunInit<T, Init> {
    fn get_inited(&Init) -> Option<T>;
}

impl<T: InitFromCache<Init>, Init> CachedInit<T, Init> {
    pub fn safe_new(init: Init) -> CachedInit<T, Init> {
        let value: T = InitFromCache::init(&init);
        CachedInit { value: UnsafeCell::new(Some(value)), init: init }
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
impl MaybeInitFromCache<ShaderInitValues> for CopyShader {
    fn maybe_init(value: &(Option<String>, Option<String>)) -> Option<CopyShader> { _init_copy_shader(value) }
}
impl MaybeInitFromCache<ShaderInitValues> for PointShader {
    fn maybe_init(value: &(Option<String>, Option<String>)) -> Option<PointShader> { _init_copy_shader(value) }
}
// TODO: use this as the impl for all InitFromCache<Init>
impl MaybeInitFromCache<BrushInitValues> for Texture {
    fn maybe_init(value: &BrushInitValues) -> Option<Texture> {
        Some(InitFromCache::init(value))
    }
}

impl InitFromCache<BrushInitValues> for Texture {
    fn init(value: &BrushInitValues) -> Texture {
        let &(format, (w, h), ref pixels) = value;
        Texture::with_image(w, h, Some(pixels.as_slice()), format)
    }
}

impl MaybeInitFromCache<LuaInitValues> for LuaScript {
    fn maybe_init(value: &Option<String>) -> Option<LuaScript> {
        LuaScript::new(value.as_ref().map(|x|x.as_slice()))
    }
}

impl<T: MaybeInitFromCache<Init>, Init> DrawObjectList<T, Init> {
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
