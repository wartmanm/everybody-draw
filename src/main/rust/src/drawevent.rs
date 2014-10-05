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
use collections::string::String;
use point::PointEntry;
use glstore::{DrawObjectIndex, DrawObjectList};
use glstore::{ShaderInitValues, BrushInitValues, LuaInitValues};
use gltexture::{Texture, PixelFormat};
use pointshader::PointShader;
use copyshader::CopyShader;
use luascript::LuaScript;
use paintlayer::PaintLayer;

enum DrawEvent {
    UseAnimShader(DrawObjectIndex<CopyShader>),
    UseCopyShader(DrawObjectIndex<CopyShader>),
    UsePointShader(DrawObjectIndex<PointShader>),
    UseBrush(DrawObjectIndex<Texture>),
    UseInterpolator(DrawObjectIndex<LuaScript>),
    Point(PointEntry),
    AddLayer(Option<DrawObjectIndex<CopyShader>>, Option<DrawObjectIndex<PointShader>>, i32),
    ClearLayers,
    Frame,
}

pub struct Events<'a> {
    eventlist: Vec<DrawEvent>,
    pointshaders: DrawObjectList<'a, PointShader, ShaderInitValues>,
    copyshaders: DrawObjectList<'a, CopyShader, ShaderInitValues>,
    textures: DrawObjectList<'a, Texture, BrushInitValues>,
    luascripts: DrawObjectList<'a, LuaScript, LuaInitValues>,
}

impl<'a> Events<'a> {
    pub fn new() -> Events<'a> {
        Events {
            eventlist: Vec::new(),
            pointshaders: DrawObjectList::new(),
            copyshaders: DrawObjectList::new(),
            textures: DrawObjectList::new(),
            luascripts: DrawObjectList::new(),
        }
    }

    // FIXME: let glstore deal with optionalness
    pub fn load_copyshader(&mut self, vert: Option<String>, frag: Option<String>) -> Option<DrawObjectIndex<CopyShader>> {
        let initargs = (vert, frag);
        self.copyshaders.push_object(initargs)
    }

    pub fn use_copyshader(&mut self, idx: DrawObjectIndex<CopyShader>) -> &'a CopyShader {
        self.eventlist.push(UseCopyShader(idx));
        self.copyshaders.get_object(idx)
    }

    pub fn use_animshader(&mut self, idx: DrawObjectIndex<CopyShader>) -> &'a CopyShader {
        self.eventlist.push(UseAnimShader(idx));
        self.copyshaders.get_object(idx)
    }

    pub fn load_pointshader(&mut self, vert: Option<String>, frag: Option<String>) -> Option<DrawObjectIndex<PointShader>> {
        let initargs = (vert, frag);
        self.pointshaders.push_object(initargs)
    }
    pub fn use_pointshader(&mut self, idx: DrawObjectIndex<PointShader>) -> &'a PointShader {
        self.eventlist.push(UsePointShader(idx));
        self.pointshaders.get_object(idx)
    }
    pub fn load_brush(&mut self, w: i32, h: i32, pixels: &[u8], format: PixelFormat) -> DrawObjectIndex<Texture> {
        let ownedpixels = pixels.to_vec();
        let init: BrushInitValues = (format, (w, h), ownedpixels);
        self.textures.safe_push_object(init)
    }
    pub fn use_brush(&mut self, idx: DrawObjectIndex<Texture>) -> &'a Texture {
        self.eventlist.push(UseBrush(idx));
        self.textures.get_object(idx)
    }
    pub fn load_interpolator(&mut self, script: Option<String>) -> Option<DrawObjectIndex<LuaScript>> {
        let initopt: LuaInitValues = script;
        self.luascripts.push_object(initopt)
    }

    pub fn use_interpolator(&mut self, idx: DrawObjectIndex<LuaScript>) -> &'a LuaScript {
        self.eventlist.push(UseInterpolator(idx));
        self.luascripts.get_object(idx)
    }

    pub fn add_layer(&mut self, dimensions: (i32, i32)
                     , copyshader: Option<DrawObjectIndex<CopyShader>>, pointshader: Option<DrawObjectIndex<PointShader>>
                     , pointidx: i32) -> PaintLayer<'a> {
        self.eventlist.push(AddLayer(copyshader, pointshader, pointidx));
        let copyshader = match copyshader { Some(x) => Some(self.copyshaders.get_object(x)), None => None };
        let pointshader = match pointshader { Some(x) => Some(self.pointshaders.get_object(x)), None => None };
        PaintLayer::new(dimensions, copyshader, pointshader, pointidx)
    }

    pub fn clear_layers(&mut self) {
        self.eventlist.push(ClearLayers);
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
    pub fn advance<'a>(&mut self, events: &'a mut Events<'a>, mut framecount: u32, playback: bool, m: &mut ::point::PointProducer, gl: &'a mut ::glinit::GLInit<'a>) {
        if framecount == 0 || self.position >= events.get_eventcount() {
            return;
        }
        let limit = events.get_eventcount();
        self.position += 1;
        let event = events.get_event(self.position);
        while framecount > 0 && self.position < limit {
            match event {
                // FIXME do this without exposing Events or GLInit internal details
                UseAnimShader(idx) => gl.paintstate.animshader = Some(events.copyshaders.get_object(idx)),
                UseCopyShader(idx) => gl.paintstate.copyshader = Some(events.copyshaders.get_object(idx)),
                UsePointShader(idx) => gl.paintstate.pointshader = Some(events.pointshaders.get_object(idx)),
                UseBrush(idx) => gl.paintstate.brush = Some(events.textures.get_object(idx)),
                UseInterpolator(idx) => gl.paintstate.interpolator = Some(events.luascripts.get_object(idx)),
                Frame => {
                    framecount -= 1;
                    if playback {
                        gl.render_frame();
                    }
                },
                Point(p) => m.push(p),
                AddLayer(_, _, _) => { },
                ClearLayers => { },
            }
        }
    }
}
