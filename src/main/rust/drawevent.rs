/// Event log for a drawing session
/// Menu selections must pass through here to ensure they can be referenced
/// Motion events can be shoved in wherever, though
/// TODO: replay
/// use timestamps on point events, rather than Frame events, so writing can be done from one
/// thread
/// think about how to make shaders handle accelerated time
///
// TODO: remove all this duplication

use core::prelude::*;
use collections::vec::Vec;
use collections::{Mutable, MutableSeq};
use collections::slice::CloneableVector;
use point::PointEntry;
use glstore::{DrawObjectIndex, DrawObjectList, CachedInit, ShaderInit, BrushInit, LuaInit};
use glstore::{ShaderInitValues, BrushInitValues, LuaInitValues};
use gltexture::{Texture, PixelFormat};
use pointshader::PointShader;
use copyshader::CopyShader;
use collections::str::StrAllocating;
use luascript::LuaScript;

enum DrawEvent {
    UseAnimShader(DrawObjectIndex<Option<CopyShader>>),
    UseCopyShader(DrawObjectIndex<Option<CopyShader>>),
    UsePointShader(DrawObjectIndex<Option<PointShader>>),
    UseBrush(DrawObjectIndex<Texture>),
    UseInterpolator(DrawObjectIndex<Option<LuaScript>>),
    Point(PointEntry),
}

pub struct Events<'a> {
    eventlist: Vec<DrawEvent>,
    pointshaders: DrawObjectList<Option<PointShader>, ShaderInitValues>,
    copyshaders: DrawObjectList<Option<CopyShader>, ShaderInitValues>,
    textures: DrawObjectList<Texture, BrushInitValues>,
    luascripts: DrawObjectList<Option<LuaScript>, LuaInitValues>,
    pub pointshader: Option<&'a PointShader>,
    pub animshader: Option<&'a CopyShader>,
    pub copyshader: Option<&'a CopyShader>,
    pub brush: Option<&'a Texture>,
    pub interpolator: Option<&'a LuaScript>,
}

impl<'a> Events<'a> {
    pub fn new() -> Events<'a> {
        Events {
            eventlist: Vec::new(),
            pointshaders: DrawObjectList::new(),
            copyshaders: DrawObjectList::new(),
            textures: DrawObjectList::new(),
            luascripts: DrawObjectList::new(),
            pointshader: None,
            animshader: None,
            copyshader: None,
            brush: None,
            interpolator: None,
        }
    }


    // FIXME: let glstore deal with optionalness
    pub fn load_copyshader(&mut self, vert: Option<&str>, frag: Option<&str>) -> DrawObjectIndex<Option<CopyShader>> {
        let initargs = (vert.map(|x|x.into_string()), frag.map(|x|x.into_string()));
        let initopt: ShaderInit<CopyShader> = CachedInit::new(initargs);
        self.copyshaders.push_object(initopt)
    }
    pub fn use_copyshader(&'a mut self, idx: DrawObjectIndex<Option<CopyShader>>) -> Option<&CopyShader> {
        self.eventlist.push(UseCopyShader(idx));
        let shader = self.copyshaders.get_object(idx).as_ref();
        self.copyshader = shader;
        shader
    }

    pub fn use_animshader(&'a mut self, idx: DrawObjectIndex<Option<CopyShader>>) -> Option<&CopyShader> {
        self.eventlist.push(UseAnimShader(idx));
        let shader = self.copyshaders.get_object(idx).as_ref();
        self.animshader = shader;
        shader
    }
    pub fn load_pointshader(&mut self, vert: Option<&str>, frag: Option<&str>) -> DrawObjectIndex<Option<PointShader>> {
        let initargs = (vert.map(|x|x.into_string()), frag.map(|x|x.into_string()));
        let initopt: ShaderInit<PointShader> = CachedInit::new(initargs);
        self.pointshaders.push_object(initopt)
    }
    pub fn use_pointshader(&'a mut self, idx: DrawObjectIndex<Option<PointShader>>) -> Option<&PointShader> {
        self.eventlist.push(UsePointShader(idx));
        let shader = self.pointshaders.get_object(idx).as_ref();
        self.pointshader = shader;
        shader
    }
    pub fn load_brush(&mut self, w: i32, h: i32, pixels: &[u8], format: PixelFormat) -> DrawObjectIndex<Texture> {
        let ownedpixels = pixels.to_vec();
        let init: BrushInit = CachedInit::new((format, (w, h), ownedpixels));
        self.textures.push_object(init)
    }
    pub fn use_brush(&'a mut self, idx: DrawObjectIndex<Texture>) -> &Texture {
        self.eventlist.push(UseBrush(idx));
        let brush = self.textures.get_object(idx);
        self.brush = Some(brush);
        brush
    }
    pub fn load_interpolator(&mut self, script: Option<&str>) -> DrawObjectIndex<Option<LuaScript>> {
        let initopt: LuaInit = CachedInit::new(script.map(|x|x.into_string()));
        self.luascripts.push_object(initopt)
    }

    pub fn use_interpolator(&'a mut self, idx: DrawObjectIndex<Option<LuaScript>>) -> Option<&LuaScript> {
        self.eventlist.push(UseInterpolator(idx));
        let interpolator = self.luascripts.get_object(idx).as_ref();
        self.interpolator = interpolator;
        interpolator
    }

    pub fn pushpoint(&mut self, event: PointEntry) {
        self.eventlist.push(Point(event));
    }
    pub fn clear(&mut self) {
        self.eventlist.clear();
    }
}
