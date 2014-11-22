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
use collections::slice::CloneSliceAllocPrelude;
use collections::vec::Vec;
use collections::string::String;
use point::PointEntry;
use glstore::{DrawObjectIndex, DrawObjectList};
use glstore::{ShaderInitValues, BrushInitValues, LuaInitValues};
use glstore::{ShaderUnfilledValues, BrushUnfilledValues, LuaUnfilledValues};
use gltexture::{BrushTexture, PixelFormat};
use pointshader::PointShader;
use copyshader::CopyShader;
use luascript::LuaScript;
use paintlayer::PaintLayer;
use glcommon::GLResult;

enum DrawEvent {
    UseAnimShader(DrawObjectIndex<CopyShader>),
    UseCopyShader(DrawObjectIndex<CopyShader>),
    UsePointShader(DrawObjectIndex<PointShader>),
    UseBrush(DrawObjectIndex<BrushTexture>),
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
    textures: DrawObjectList<'a, BrushTexture, BrushInitValues>,
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
    pub fn load_copyshader(&mut self, vert: Option<String>, frag: Option<String>) -> GLResult<DrawObjectIndex<CopyShader>> {
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

    pub fn load_pointshader(&mut self, vert: Option<String>, frag: Option<String>) -> GLResult<DrawObjectIndex<PointShader>> {
        let initargs = (vert, frag);
        self.pointshaders.push_object(initargs)
    }
    pub fn use_pointshader(&mut self, idx: DrawObjectIndex<PointShader>) -> &'a PointShader {
        self.eventlist.push(UsePointShader(idx));
        self.pointshaders.get_object(idx)
    }
    pub fn load_brush(&mut self, w: i32, h: i32, pixels: &[u8], format: PixelFormat) -> DrawObjectIndex<BrushTexture> {
        let ownedpixels = pixels.to_vec();
        let init: BrushUnfilledValues = (format, (w, h), ownedpixels);
        self.textures.safe_push_object(init)
    }
    pub fn use_brush(&mut self, idx: DrawObjectIndex<BrushTexture>) -> &'a BrushTexture {
        self.eventlist.push(UseBrush(idx));
        self.textures.get_object(idx)
    }
    pub fn load_interpolator(&mut self, script: Option<String>) -> GLResult<DrawObjectIndex<LuaScript>> {
        let initopt: LuaUnfilledValues = script;
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
    fn get_event(&self, idx: uint) -> Option<&DrawEvent> {
        self.eventlist.as_slice().get(idx)
    }
}

#[inline]
pub fn handle_event<'a>(gl: &mut ::glinit::GLInit<'a>, events: &mut Events<'a>, queue: &mut ::point::PointProducer, eventidx: i32) -> event_stream::EventState {

    // FIXME do this without exposing Events or GLInit internal details
    match events.get_event(eventidx as uint) {
        Some(&event) => match event {
            UseAnimShader(idx) => gl.set_anim_shader(events.copyshaders.get_object(idx)),
            UseCopyShader(idx) => gl.set_copy_shader(events.copyshaders.get_object(idx)),
            UsePointShader(idx) => gl.set_point_shader(events.pointshaders.get_object(idx)),
            UseBrush(idx) => gl.set_brush_texture(&events.textures.get_object(idx).texture),
            UseInterpolator(idx) => gl.set_interpolator(events.luascripts.get_object(idx)),
            Point(p) => queue.push(p),
            AddLayer(copyshader, pointshader, pointidx) => {
                let copyshader = match copyshader { Some(x) => Some(events.copyshaders.get_object(x)), None => None };
                let pointshader = match pointshader { Some(x) => Some(events.pointshaders.get_object(x)), None => None };
                let layer = PaintLayer::new(gl.dimensions, copyshader, pointshader, pointidx);
                gl.add_layer(layer);
            },
            ClearLayers => gl.clear_layers(),
            Frame => return event_stream::Frame,
        },
        None => return event_stream::Done,
    }
    return event_stream::NoFrame;
}

pub mod event_stream {
    use core::prelude::*;
    use drawevent::{Events, handle_event};

    pub enum EventState {
        Done,
        Frame,
        NoFrame,
    }


    pub struct EventStream {
        position: i32,
        producer: ::glpoint::MotionEventProducer,
        pub consumer: ::glpoint::MotionEventConsumer,
    }

    impl EventStream {

        pub fn new() -> EventStream {
            let (consumer, producer) = ::glpoint::create_motion_event_handler();
            EventStream {
                position: 0,
                producer: producer,
                consumer: consumer
            }
        }

        pub fn advance_frame<'a>(&mut self, init: &mut ::glinit::GLInit<'a>, events: &mut Events<'a>) -> bool {
            loop {
                match handle_event(init, events, &mut self.producer.producer, self.position) {
                    Done => return true,
                    Frame => return false,
                    NoFrame => { },
                }
            }
        }
    }
}
