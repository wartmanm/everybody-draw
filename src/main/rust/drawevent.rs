/// Event log for a drawing session
/// Menu selections must pass through here to ensure they can be referenced
/// Motion events can be shoved in wherever, though
/// TODO: replay
/// use timestamps on point events, rather than Frame events, so writing can be done from one
/// thread
/// think about how to make shaders handle accelerated time

use core::prelude::*;
use collections::vec::Vec;
use core::cell::{Ref};
use collections::{Mutable, MutableSeq};
use collections::str::StrAllocating;
use collections::slice::CloneableVector;
use point::PointEntry;
use glstore::{DrawObjectIndex, DrawObjectList, ShaderInit, BrushInit, CachedInit};
use gltexture::{Texture, PixelFormat};
use pointshader::PointShader;
use copyshader::CopyShader;

enum DrawEvent {
    UseAnimShader(DrawObjectIndex<CopyShader>),
    UsePointShader(DrawObjectIndex<PointShader>),
    UseBrush(DrawObjectIndex<Texture>),
    Point(PointEntry),
}

pub struct Events<'a> {
    eventlist: Vec<DrawEvent>,
    objects: DrawObjectList,
    pub pointshader: Option<&'a PointShader>,
    pub animshader: Option<&'a CopyShader>,
    pub brush: Texture,
}

impl<'a> Events<'a> {
    pub fn new(brush: Texture) -> Events<'a> {
        Events {
            eventlist: Vec::new(),
            objects: DrawObjectList::new(),
            pointshader: None,
            animshader: None,
            brush: brush,
        }
    }
    // FIXME: let glstore deal with optionalness
    pub fn load_copyshader(&mut self, vert: Option<&str>, frag: Option<&str>) -> Option<DrawObjectIndex<CopyShader>> {
        let initargs = (vert.map(|x|x.to_owned()), frag.map(|x|x.to_owned()));
        let initopt: ShaderInit<CopyShader> = CachedInit::new(initargs);
        if (initopt.get().is_some()) { Some(self.objects.push_copyshader(initopt)) }
        else { None }
    }
    pub fn use_animshader(&'a mut self, idx: DrawObjectIndex<CopyShader>) -> &CopyShader {
        self.eventlist.push(UseAnimShader(idx));
        let shader = self.objects.get_copyshader(idx);
        self.animshader = Some(shader);
        shader
    }
    pub fn load_pointshader(&mut self, vert: Option<&str>, frag: Option<&str>) -> Option<DrawObjectIndex<PointShader>> {
        let initargs = (vert.map(|x|x.to_owned()), frag.map(|x|x.to_owned()));
        let initopt: ShaderInit<PointShader> = CachedInit::new(initargs);
        if (initopt.get().is_some()) { Some(self.objects.push_pointshader(initopt)) }
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
        let init = CachedInit::new((format, (w, h), ownedpixels));
        self.objects.push_brush(init)
    }
    pub fn use_brush(&mut self, idx: DrawObjectIndex<Texture>) -> &Texture {
        self.eventlist.push(UseBrush(idx));
        self.objects.get_brush(idx)
    }
    pub fn pushpoint(&mut self, event: PointEntry) {
        self.eventlist.push(Point(event));
    }
    pub fn clear(&mut self) {
        self.eventlist.clear();
    }
}
