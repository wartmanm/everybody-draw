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
use core::mem;
use collections::vec::Vec;
use collections::string::String;
use collections::{Map, MutableMap, MutableSeq};
use collections::hash::Hash;
use collections::hash::sip::SipHasher;
use std::collections::HashMap;
use copyshader::CopyShader;
use gltexture::{PixelFormat, Texture};
use pointshader::PointShader;
use glcommon::Shader;
use luascript::LuaScript;
use arena::TypedArena;

/// Holds GL objects that can be inited using the given keys.
/// The list is to avoid having to pass those keys around, and serialize more easily.
/// The arena doesn't relocate its entries, so we can pass back longer-lived pointers,
/// even if it needs some encouragement to do so.
/// There ought to be a better way.
pub struct DrawObjectList<'a, T: 'a, Init: Eq+Hash> {
    map: HashMap<Init, DrawObjectIndex<T>, SipHasher>,
    list: Vec<&'a T>,
    arena: TypedArena<T>,
}

pub struct DrawObjectIndex<T>(i32);

impl<T> DrawObjectIndex<T> {
    pub fn error() -> DrawObjectIndex<T> {
        unsafe { mem::transmute(-1i) }
    }
}

pub type ShaderInitValues = (Option<String>, Option<String>);
pub type BrushInitValues = (PixelFormat, (i32, i32), Vec<u8>);
pub type LuaInitValues = Option<String>;
pub type PointShaderIndex = DrawObjectIndex<PointShader>;
pub type CopyShaderIndex = DrawObjectIndex<CopyShader>;
pub type BrushIndex = DrawObjectIndex<Texture>;
pub type LuaIndex = DrawObjectIndex<LuaScript>;

pub trait InitFromCache<Init> {
    fn init(&Init) -> Self;
}
pub trait MaybeInitFromCache<Init: Eq+Hash> {
    fn maybe_init(&Init) -> Option<Self>;
}
trait ToOwnedInit<Init> {
    fn to_owned(&Self) -> Init;
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

impl<'a, T: MaybeInitFromCache<Init>, Init: Hash+Eq> DrawObjectList<'a, T, Init> {
    pub fn new() -> DrawObjectList<'a, T, Init> {
        // the default hasher is keyed off of the task-local rng,
        // which would blow up since we don't have a task
        //let mut rng = rand::weak_rng();
        //let hasher = SipHasher::new_with_keys(rng.next_u64(), rng.next_u64());
        // FIXME weak_rng also blows up? can it not find /dev/urandom?
        let hasher = SipHasher::new_with_keys(0, 0);
        let map = HashMap::with_hasher(hasher);
        DrawObjectList {
            map: map,
            list: Vec::new(),
            arena: TypedArena::new(),
        }
    }

    // TODO: avoid allocations just to see if the key is in the map
    pub fn push_object(&mut self, init: Init) -> Option<DrawObjectIndex<T>> {
        // Can't use map.entry() here as it consumes the key
        if self.map.contains_key(&init) {
            Some(*self.map.find(&init).unwrap())
        } else {
            match MaybeInitFromCache::maybe_init(&init) {
                Some(inited) => {
                    // ptr's lifetime is limited to &self's, which is fair but not very useful.
                    // smart ptrs involve individual allocs but are probably better
                    let ptr = self.arena.alloc(inited);
                    unsafe {
                        self.list.push(mem::transmute(ptr));
                    }
                    let index = self.list.len() - 1;
                    let objindex = DrawObjectIndex(index as i32);
                    self.map.insert(init, objindex);
                    Some(objindex)
                },
                None => None,
            }
        }
    }

    pub fn get_object(&self, i: DrawObjectIndex<T>) -> &'a T {
        let DrawObjectIndex(idx) = i;
        self.list[idx as uint]
    }
}

impl<'a, T: InitFromCache<Init>+MaybeInitFromCache<Init>, Init: Hash+Eq> DrawObjectList<'a, T, Init> {
    pub fn safe_push_object(&mut self, init: Init) -> DrawObjectIndex<T> {
        self.push_object(init).unwrap()
    }
}

