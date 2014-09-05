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

#[deriving(PartialEq)]
enum DrawEvent {
    UseAnimShader(DrawObjectIndex<CopyShader>),
    UseCopyShader(DrawObjectIndex<CopyShader>),
    UsePointShader(DrawObjectIndex<PointShader>),
    UseBrush(DrawObjectIndex<Texture>),
    UseInterpolator(DrawObjectIndex<LuaScript>),
    Point(PointEntry),
    Frame,
}

pub struct Events<'a> {
    eventlist: Vec<DrawEvent>,
    pointshaders: DrawObjectList<PointShader, ShaderInitValues>,
    copyshaders: DrawObjectList<CopyShader, ShaderInitValues>,
    textures: DrawObjectList<Texture, BrushInitValues>,
    luascripts: DrawObjectList<LuaScript, LuaInitValues>,
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
    pub fn load_copyshader(&mut self, vert: Option<&str>, frag: Option<&str>) -> Option<DrawObjectIndex<CopyShader>> {
        let initargs = (vert.map(|x|x.into_string()), frag.map(|x|x.into_string()));
        let initopt: Option<ShaderInit<CopyShader>> = CachedInit::new(initargs);
        initopt.map(|x| self.copyshaders.push_object(x))
    }

    pub fn use_copyshader(&'a mut self, idx: DrawObjectIndex<CopyShader>) -> &CopyShader {
        self.eventlist.push(UseCopyShader(idx));
        let shader = self.copyshaders.get_object(idx);
        self.copyshader = Some(shader);
        shader
    }

    pub fn use_animshader(&'a mut self, idx: DrawObjectIndex<CopyShader>) -> &CopyShader {
        self.eventlist.push(UseAnimShader(idx));
        let shader = self.copyshaders.get_object(idx);
        self.animshader = Some(shader);
        shader
    }
    pub fn load_pointshader(&mut self, vert: Option<&str>, frag: Option<&str>) -> Option<DrawObjectIndex<PointShader>> {
        let initargs = (vert.map(|x|x.into_string()), frag.map(|x|x.into_string()));
        let initopt: Option<ShaderInit<PointShader>> = CachedInit::new(initargs);
        initopt.map(|x| self.pointshaders.push_object(x))
    }
    pub fn use_pointshader(&'a mut self, idx: DrawObjectIndex<PointShader>) -> &PointShader {
        self.eventlist.push(UsePointShader(idx));
        let shader = self.pointshaders.get_object(idx);
        self.pointshader = Some(shader);
        shader
    }
    pub fn load_brush(&mut self, w: i32, h: i32, pixels: &[u8], format: PixelFormat) -> DrawObjectIndex<Texture> {
        let ownedpixels = pixels.to_vec();
        let init: BrushInit = CachedInit::safe_new((format, (w, h), ownedpixels));
        self.textures.push_object(init)
    }
    pub fn use_brush(&'a mut self, idx: DrawObjectIndex<Texture>) -> &Texture {
        self.eventlist.push(UseBrush(idx));
        let brush = self.textures.get_object(idx);
        self.brush = Some(brush);
        brush
    }
    pub fn load_interpolator(&mut self, script: Option<&str>) -> Option<DrawObjectIndex<LuaScript>> {
        let initopt: Option<LuaInit> = CachedInit::new(script.map(|x|x.into_string()));
        initopt.map(|x| self.luascripts.push_object(x))
    }

    pub fn use_interpolator(&'a mut self, idx: DrawObjectIndex<LuaScript>) -> &LuaScript {
        self.eventlist.push(UseInterpolator(idx));
        let interpolator = self.luascripts.get_object(idx);
        self.interpolator = Some(interpolator);
        interpolator
    }

    pub fn pushpoint(&mut self, event: PointEntry) {
        self.eventlist.push(Point(event));
    }
    pub fn pushframe(&mut self) {
        self.eventlist.push(Frame);
    }
    pub fn clear(&mut self) {
        self.eventlist.clear();
    }
    fn get_event(&self, idx: uint) -> DrawEvent {
        self.eventlist[idx]
    }
    pub fn get_eventcount(&self) -> uint {
        self.eventlist.len()
    }
}

pub struct EventStream {
    position: uint,
}

impl EventStream {
    pub fn new() -> EventStream {
        EventStream { position: 0 }
    }
    pub fn advance<'a>(&mut self, events: &'a mut Events<'a>, mut framecount: u32, playback: bool, m: &mut ::point::PointProducer, gl: &mut ::glinit::Data) {
        if framecount == 0 || self.position >= events.get_eventcount() {
            return;
        }
        let limit = events.get_eventcount();
        self.position += 1;
        let event = events.get_event(self.position);
        while framecount > 0 && self.position < limit {
            match event {
                // FIXME do this without exposing Events internal details
                UseAnimShader(idx) => events.animshader = Some(events.copyshaders.get_object(idx)),
                UseCopyShader(idx) => events.copyshader = Some(events.copyshaders.get_object(idx)),
                UsePointShader(idx) => events.pointshader = Some(events.pointshaders.get_object(idx)),
                UseBrush(idx) => events.brush = Some(events.textures.get_object(idx)),
                UseInterpolator(idx) => events.interpolator = Some(events.luascripts.get_object(idx)),
                Frame => {
                    framecount -= 1;
                    if playback {
                        ::glinit::render_frame(gl);
                    }
                },
                Point(p) => m.push(p),
            }
        }
    }
}
