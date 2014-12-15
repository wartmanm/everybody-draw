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
use core::fmt::Show;
use collections::vec::Vec;
use collections::hash::Hash;
use collections::hash::sip::SipHasher;

use std::collections::HashMap;
use std::collections::hash_map::{Vacant, Occupied};
use copyshader::CopyShader;
use gltexture::{PixelFormat, Texture, BrushTexture};
use pointshader::PointShader;
use glcommon::Shader;
use luascript::LuaScript;
use arena::TypedArena;
use glcommon::GLResult;
use glcommon::{FillDefaults, MString};

use core::borrow::IntoCow;

/// Holds GL objects that can be inited using the given keys.
/// The list is to avoid having to pass those keys around, and serialize more easily.
/// The arena doesn't relocate its entries, so we can pass back longer-lived pointers,
/// even if it needs some encouragement to do so.
/// There ought to be a better way.
pub struct DrawObjectList<'a, T: 'a, Init: Eq+Hash+'a> {
    map: HashMap<&'a Init, DrawObjectIndex<T>, SipHasher>,
    list: Vec<&'a T>,
    arena: TypedArena<T>,
}

// copy doesn't work, wtf
#[deriving(Show, Copy)]
pub struct DrawObjectIndex<T>(i32);

impl<T> DrawObjectIndex<T> {
    pub fn error() -> DrawObjectIndex<T> {
        unsafe { mem::transmute(-1i32) }
    }
}

// again, wtf, why is this needed
impl<T> Clone for DrawObjectIndex<T> {
    fn clone(&self) -> DrawObjectIndex<T> {
        let DrawObjectIndex(idx) = *self;
        DrawObjectIndex(idx)
    }
}

pub type ShaderInitValues = (MString, MString);
pub type BrushInitValues = (PixelFormat, (i32, i32), Vec<u8>);
pub type LuaInitValues = MString;
//pub type ShaderKeyValues = &(String, String);
//pub type BrushKeyValues = (PixelFormat, (i32, i32), Vec<u8>>);
//pub type LuaKeyValues = &String;
pub type ShaderUnfilledValues = (Option<MString>, Option<MString>);
pub type BrushUnfilledValues = BrushInitValues;
pub type LuaUnfilledValues = Option<MString>;
pub type PointShaderIndex = DrawObjectIndex<PointShader>;
pub type CopyShaderIndex = DrawObjectIndex<CopyShader>;
pub type BrushIndex = DrawObjectIndex<Texture>;
pub type LuaIndex = DrawObjectIndex<LuaScript>;

pub trait InitFromCache<Init> {
    fn init(Init) -> Self;
}
pub trait MaybeInitFromCache<Init> {
    fn maybe_init(Init) -> GLResult<Self>;
    fn get_source(&self) -> &Init;
}

// this and the two shader impls were originally a single
// impl<T: Shader> InitFromCache<ShaderInitValues> for Option<T>
// but that counts as the impl for all of Option, not just Option<Shader>
fn _init_copy_shader<T: Shader>(value: (MString, MString)) -> GLResult<T> {
    let (vert, frag) = value;
    Shader::new(vert, frag)
}
impl MaybeInitFromCache<ShaderInitValues> for CopyShader {
    fn maybe_init(value: (MString, MString)) -> GLResult<CopyShader> { _init_copy_shader(value) }
    fn get_source(&self) -> &(MString, MString) { &self.source }
}
impl MaybeInitFromCache<ShaderInitValues> for PointShader {
    fn maybe_init(value: (MString, MString)) -> GLResult<PointShader> { _init_copy_shader(value) }
    fn get_source(&self) -> &(MString, MString) { &self.source }
}
// TODO: use this as the impl for all InitFromCache<Init>
impl MaybeInitFromCache<BrushInitValues> for BrushTexture {
    fn maybe_init(value: BrushInitValues) -> GLResult<BrushTexture> {
        Ok(InitFromCache::init(value))
    }
    fn get_source(&self) -> &BrushInitValues { &self.source }
}

pub fn init_from_defaults<Unfilled, T: MaybeInitFromCache<Init> + FillDefaults<Unfilled, Init, T>, Init>(init: Unfilled) -> GLResult<T> {
    let filled = FillDefaults::fill_defaults(init).val;
    MaybeInitFromCache::<Init>::maybe_init(filled)
}

// FIXME maybe a BrushTexture wrapper?
impl InitFromCache<BrushInitValues> for BrushTexture {
    fn init(value: BrushInitValues) -> BrushTexture {
        let tex = {
            let (ref format, (w, h), ref pixels) = value;
            Texture::with_image(w, h, Some(pixels.as_slice()), *format)
        };
        BrushTexture { texture: tex, source: value }
    }
}

impl MaybeInitFromCache<LuaInitValues> for LuaScript {
    fn maybe_init(value: MString) -> GLResult<LuaScript> {
        LuaScript::new(value)
    }
    fn get_source(&self) -> &MString { &self.source }
}

impl<'a, Unfilled, T: MaybeInitFromCache<Init> + FillDefaults<Unfilled, Init, T>, Init: Hash+Eq+Show> DrawObjectList<'a, T, Init> {
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

    pub fn push_object(&mut self, init: Unfilled) -> GLResult<DrawObjectIndex<T>> {
        // Can't use map.entry() here as it consumes the key
        let filled = FillDefaults::fill_defaults(init).val;
        let filledref: &'a Init = unsafe { mem::transmute(&filled) };
        // see below -- these are safe, we just can't prove it
        match self.map.entry(filledref) {
            Occupied(entry) => Ok(entry.get().clone()),
            Vacant(entry) => {
                let inited: T = try!(MaybeInitFromCache::<Init>::maybe_init(filled));
                // ptr's lifetime is limited to &self's, which is fair but not very useful.
                // smart ptrs involve individual allocs but are probably better
                let ptr = self.arena.alloc(inited);
                unsafe {
                    self.list.push(mem::transmute(ptr));
                }
                let index = self.list.len() - 1;
                let objindex = DrawObjectIndex(index as i32);
                entry.set(objindex.clone());
                Ok(objindex)
            }
        }
    }

    pub fn get_object(&self, i: DrawObjectIndex<T>) -> &'a T {
        let DrawObjectIndex(idx) = i;
        self.list[idx as uint]
    }

    pub fn maybe_get_object(&self, i: DrawObjectIndex<T>) -> GLResult<&'a T> {
        let DrawObjectIndex(idx) = i;
        match self.list.get(idx as uint) {
            Some(x) => Ok(*x),
            None => Err(format!("tried to get glstore index {} of {}", idx, self.list.len()).into_cow()),
        }
    }
}

impl<'a, Unfilled, T: InitFromCache<Init> + MaybeInitFromCache<Init> + FillDefaults<Unfilled, Init, T>, Init: Hash+Eq+Show> DrawObjectList<'a, T, Init> {
    pub fn safe_push_object(&mut self, init: Unfilled) -> DrawObjectIndex<T> {
        self.push_object(init).unwrap()
    }
}

#[test]
fn equal_keys_match() {
    let mut list: DrawObjectList<LuaScript, LuaInitValues> = DrawObjectList::new();
    let script_1 = "function main() end".into_cow();
    let script_2 = "function main() end".into_cow();
    let idx_1 = list.push_object(Some(script_1)).unwrap();
    let idx_2 = list.push_object(Some(script_2)).unwrap();
    let (DrawObjectIndex(i1), DrawObjectIndex(i2)) = (idx_1, idx_2);
    assert_eq!(i1, i2);
    println!("test");
}

#[test]
fn different_keys_differ() {
    let mut list: DrawObjectList<LuaScript, LuaInitValues> = DrawObjectList::new();
    let script_1 = "function main() end".into_cow();
    let script_2 = "function main() end \n-- hello world".into_cow();
    let idx_1 = list.push_object(Some(script_1)).unwrap();
    let idx_2 = list.push_object(Some(script_2)).unwrap();
    let (DrawObjectIndex(i1), DrawObjectIndex(i2)) = (idx_1, idx_2);
    assert!(i1 != i2);
    println!("test");
}
