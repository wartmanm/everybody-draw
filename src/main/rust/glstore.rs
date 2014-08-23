/// DrawObjectList interns shaders, brushes, and scripts, returning references that can be stored
/// in the event queue.
/// TODO: scripts
/// TODO: serialization
/// TODO: further backing store, caching init objs by sha1 or so
/// TODO: free shaders + textures on gl pause
/// TODO: cleanup, deduplication

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

pub struct DrawObjectList {
    copyshaderlist: Vec<ShaderInit<CopyShader>>,
    pointshaderlist: Vec<ShaderInit<PointShader>>,
    brushlist: Vec<BrushInit>,
    interplist: Vec<LuaInit>,
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

impl DrawObjectList {
    pub fn new() -> DrawObjectList {
        DrawObjectList {
            copyshaderlist: Vec::new(),
            pointshaderlist: Vec::new(),
            brushlist: Vec::new(),
            interplist: Vec::new(),
        }
    }

    pub fn push_copyshader(&mut self, shader: ShaderInit<CopyShader>) -> DrawObjectIndex<CopyShader> {
        self.copyshaderlist.push(shader);
        DrawObjectIndex((self.copyshaderlist.len() - 1) as i32)
    }
    pub fn push_pointshader(&mut self, shader: ShaderInit<PointShader>) -> DrawObjectIndex<PointShader> {
        self.pointshaderlist.push(shader);
        DrawObjectIndex((self.copyshaderlist.len() - 1) as i32)
    }
    pub fn push_brush(&mut self, brush: BrushInit) -> DrawObjectIndex<Texture> {
        self.brushlist.push(brush);
        DrawObjectIndex((self.brushlist.len() - 1) as i32)
    }
    pub fn push_interpolator(&mut self, interpolator: LuaInit) -> DrawObjectIndex<LuaScript> {
        self.interplist.push(interpolator);
        DrawObjectIndex((self.interplist.len() -1) as i32)
    }

    // FIXME: push optionalness out elsewhere
    pub fn get_copyshader(&self, i: DrawObjectIndex<CopyShader>) -> &CopyShader {
        let DrawObjectIndex(idx) = i;
        self.copyshaderlist[idx as uint].get().as_ref().unwrap()
    }
    pub fn get_pointshader(&self, i: DrawObjectIndex<PointShader>) -> &PointShader {
        let DrawObjectIndex(idx) = i;
        self.pointshaderlist[idx as uint].get().as_ref().unwrap()
    }
    pub fn get_brush(&self, i: DrawObjectIndex<Texture>) -> &Texture {
        let DrawObjectIndex(idx) = i;
        self.brushlist[idx as uint].get()
    }
    pub fn get_interpolator(&self, i: DrawObjectIndex<LuaScript>) -> &LuaScript {
        let DrawObjectIndex(idx) = i;
        self.interplist[idx as uint].get().as_ref().unwrap()
    }
}
