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
use glstore::{DrawObjectIndex, DrawObjectList, ShaderInit, BrushInit, CachedInit, LuaInit};
use gltexture::{Texture, PixelFormat};
use pointshader::PointShader;
use copyshader::CopyShader;
use std::to_string::ToString;
use luascript::LuaScript;

enum DrawEvent {
    UseAnimShader(DrawObjectIndex<CopyShader>),
    UseCopyShader(DrawObjectIndex<CopyShader>),
    UsePointShader(DrawObjectIndex<PointShader>),
    UseBrush(DrawObjectIndex<Texture>),
    UseInterpolator(DrawObjectIndex<LuaScript>),
    Point(PointEntry),
}

pub struct Events<'a> {
    eventlist: Vec<DrawEvent>,
    objects: DrawObjectList,
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
            objects: DrawObjectList::new(),
            pointshader: None,
            animshader: None,
            copyshader: None,
            brush: None,
            interpolator: None,
        }
    }
    // FIXME: let glstore deal with optionalness
    pub fn load_copyshader(&mut self, vert: Option<&str>, frag: Option<&str>) -> Option<DrawObjectIndex<CopyShader>> {
        let initargs = (vert.map(|x|x.to_string()), frag.map(|x|x.to_string()));
        let initopt: ShaderInit<CopyShader> = CachedInit::new(initargs);
        if initopt.get().is_some() { Some(self.objects.push_copyshader(initopt)) }
        else { None }
    }
    pub fn use_copyshader(&'a mut self, idx: DrawObjectIndex<CopyShader>) -> &CopyShader {
        self.eventlist.push(UseCopyShader(idx));
        let shader = self.objects.get_copyshader(idx);
        self.copyshader = Some(shader);
        shader
    }

    pub fn use_animshader(&'a mut self, idx: DrawObjectIndex<CopyShader>) -> &CopyShader {
        self.eventlist.push(UseAnimShader(idx));
        let shader = self.objects.get_copyshader(idx);
        self.animshader = Some(shader);
        shader
    }
    pub fn load_pointshader(&mut self, vert: Option<&str>, frag: Option<&str>) -> Option<DrawObjectIndex<PointShader>> {
        let initargs = (vert.map(|x|x.to_string()), frag.map(|x|x.to_string()));
        let initopt: ShaderInit<PointShader> = CachedInit::new(initargs);
        if initopt.get().is_some() { Some(self.objects.push_pointshader(initopt)) }
        else { None }
    }
    pub fn use_pointshader(&'a mut self, idx: DrawObjectIndex<PointShader>) -> &PointShader {
        self.eventlist.push(UsePointShader(idx));
        let shader = self.objects.get_pointshader(idx);
        self.pointshader = Some(shader);
        shader
    }
    pub fn load_brush(&mut self, w: i32, h: i32, pixels: &[u8], format: PixelFormat) -> DrawObjectIndex<Texture> {
        let ownedpixels = pixels.to_vec();
        let init: BrushInit = CachedInit::new((format, (w, h), ownedpixels));
        self.objects.push_brush(init)
    }
    pub fn use_brush(&'a mut self, idx: DrawObjectIndex<Texture>) -> &Texture {
        self.eventlist.push(UseBrush(idx));
        let brush = self.objects.get_brush(idx);
        self.brush = Some(brush);
        brush
    }
    pub fn load_interpolator(&mut self, script: Option<&str>) -> Option<DrawObjectIndex<LuaScript>> {
        let initopt: LuaInit = CachedInit::new(script.map(|x|x.to_string()));
        //let initopt: LuaInit = CachedInit::new(initargs);
        if initopt.get().is_some() { Some(self.objects.push_interpolator(initopt)) }
        else { None }
    }

    pub fn use_interpolator(&'a mut self, idx: DrawObjectIndex<LuaScript>) -> &LuaScript {
        self.eventlist.push(UseInterpolator(idx));
        let interpolator = self.objects.get_interpolator(idx);
        self.interpolator = Some(interpolator);
        interpolator
    }

    pub fn pushpoint(&mut self, event: PointEntry) {
        self.eventlist.push(Point(event));
    }
    pub fn clear(&mut self) {
        self.eventlist.clear();
    }
}
