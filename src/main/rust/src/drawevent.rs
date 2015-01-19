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
use core::borrow::ToOwned;
use collections::vec::Vec;
use point::PointEntry;
use glstore::{DrawObjectIndex, DrawObjectList};
use glstore::{ShaderInitValues, BrushInitValues, LuaInitValues};
use glstore::{BrushUnfilledValues, LuaUnfilledValues};
//use glstore::MaybeInitFromCache; // FIXME separate out get_source()
use gltexture::{BrushTexture, PixelFormat};
use pointshader::PointShader;
use copyshader::CopyShader;
use luascript::LuaScript;
use paintlayer::PaintLayer;
use glcommon::{GLResult, MString, UsingDefaults};
use drawevent::event_stream::EventState;
//use collections::slice::CloneSliceExt;

// can't use Copy, wtf
#[derive(Clone)]
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
    pub fn load_copyshader(&mut self, vert: Option<MString>, frag: Option<MString>) -> GLResult<DrawObjectIndex<CopyShader>> {
        let initargs = (vert, frag);
        self.copyshaders.push_object(initargs)
    }

    pub fn use_copyshader(&mut self, idx: DrawObjectIndex<CopyShader>) -> GLResult<&'a CopyShader> {
        self.eventlist.push(DrawEvent::UseCopyShader(idx.clone()));
        self.copyshaders.maybe_get_object(idx)
    }

    pub fn use_animshader(&mut self, idx: DrawObjectIndex<CopyShader>) -> GLResult<&'a CopyShader> {
        self.eventlist.push(DrawEvent::UseAnimShader(idx.clone()));
        self.copyshaders.maybe_get_object(idx)
    }

    pub fn load_pointshader(&mut self, vert: Option<MString>, frag: Option<MString>) -> GLResult<DrawObjectIndex<PointShader>> {
        let initargs = (vert, frag);
        self.pointshaders.push_object(initargs)
    }
    pub fn use_pointshader(&mut self, idx: DrawObjectIndex<PointShader>) -> GLResult<&'a PointShader> {
        self.eventlist.push(DrawEvent::UsePointShader(idx.clone()));
        self.pointshaders.maybe_get_object(idx)
    }
    pub fn load_brush(&mut self, w: i32, h: i32, pixels: &[u8], format: PixelFormat) -> DrawObjectIndex<BrushTexture> {
        let ownedpixels = pixels.to_owned();
        let init: BrushUnfilledValues = (format, (w, h), ownedpixels);
        self.textures.safe_push_object(init)
    }
    pub fn use_brush(&mut self, idx: DrawObjectIndex<BrushTexture>) -> GLResult<&'a BrushTexture> {
        self.eventlist.push(DrawEvent::UseBrush(idx.clone()));
        self.textures.maybe_get_object(idx)
    }
    pub fn load_interpolator(&mut self, script: Option<MString>) -> GLResult<DrawObjectIndex<LuaScript>> {
        let initopt: LuaUnfilledValues = script;
        self.luascripts.push_object(initopt)
    }

    pub fn use_interpolator(&mut self, idx: DrawObjectIndex<LuaScript>) -> GLResult<&'a LuaScript> {
        self.eventlist.push(DrawEvent::UseInterpolator(idx.clone()));
        self.luascripts.maybe_get_object(idx)
    }

    pub fn add_layer(&mut self, dimensions: (i32, i32)
                     , copyshader: Option<DrawObjectIndex<CopyShader>>, pointshader: Option<DrawObjectIndex<PointShader>>
                     , pointidx: i32) -> PaintLayer<'a> {
        self.eventlist.push(DrawEvent::AddLayer(copyshader.clone(), pointshader.clone(), pointidx));
        let copyshader = match copyshader { Some(x) => Some(self.copyshaders.get_object(x)), None => None };
        let pointshader = match pointshader { Some(x) => Some(self.pointshaders.get_object(x)), None => None };
        PaintLayer::new(dimensions, copyshader, pointshader, pointidx)
    }

    pub fn clear_layers(&mut self) {
        self.eventlist.push(DrawEvent::ClearLayers);
    }

    pub fn get_pointshader_source(&mut self, pointshader: DrawObjectIndex<PointShader>) -> &(MString, MString) {
        self.pointshaders.get_object(pointshader).get_source()
    }

    pub fn get_copyshader_source(&mut self, copyshader: DrawObjectIndex<CopyShader>) -> &(MString, MString) {
        self.copyshaders.get_object(copyshader).get_source()
    }

    pub fn get_luascript_source(&mut self, luascript: DrawObjectIndex<LuaScript>) -> &MString {
        self.luascripts.get_object(luascript).get_source()
    }

    pub fn pushpoint(&mut self, event: PointEntry) {
        self.eventlist.push(DrawEvent::Point(event));
    }
    pub fn pushframe(&mut self) {
        self.eventlist.push(DrawEvent::Frame);
    }
    pub fn clear(&mut self) {
        self.eventlist.clear();
    }
    //fn get_event(&self, idx: uint) -> Option<&DrawEvent> {
        //self.eventlist.as_slice().get(idx)
    //}
}

#[inline]
#[allow(unused)]
pub fn handle_event<'a>(gl: &mut ::glinit::GLInit<'a>, events: &mut Events<'a>, queue: &mut ::point::PointProducer, eventidx: i32) -> event_stream::EventState {

    // FIXME do this without exposing Events or GLInit internal details
    /*
    match events.get_event(eventidx as uint) {
        Some(event) => match event.clone() {
            DrawEvent::UseAnimShader(idx) => gl.set_anim_shader(events.copyshaders.get_object(idx)),
            DrawEvent::UseCopyShader(idx) => gl.set_copy_shader(events.copyshaders.get_object(idx)),
            DrawEvent::UsePointShader(idx) => gl.set_point_shader(events.pointshaders.get_object(idx)),
            DrawEvent::UseBrush(idx) => gl.set_brush_texture(&events.textures.get_object(idx).texture),
            DrawEvent::UseInterpolator(idx) => gl.set_interpolator(events.luascripts.get_object(idx)),
            DrawEvent::Point(p) => queue.send(p),
            DrawEvent::AddLayer(copyshader, pointshader, pointidx) => {
                let copyshader = match copyshader { Some(x) => Some(events.copyshaders.get_object(x)), None => None };
                let pointshader = match pointshader { Some(x) => Some(events.pointshaders.get_object(x)), None => None };
                let layer = PaintLayer::new(gl.dimensions, copyshader, pointshader, pointidx);
                gl.add_layer(layer);
            },
            DrawEvent::ClearLayers => gl.clear_layers(),
            DrawEvent::Frame => return EventState::Frame,
        },
        None => return EventState::Done,
    }
    */
    return EventState::NoFrame;
}

pub mod event_stream {
    use core::prelude::*;
    use drawevent::{Events, handle_event};

    #[derive(Copy)]
    pub enum EventState {
        Done,
        Frame,
        NoFrame,
    }


    pub struct EventStream {
        position: i32,
        pub consumer: ::glpoint::MotionEventConsumer,
        producer: ::glpoint::MotionEventProducer,
    }

    impl EventStream {

        pub fn new() -> EventStream {
            let (consumer, producer) = ::glpoint::create_motion_event_handler(0);
            EventStream {
                position: 0,
                producer: producer,
                consumer: consumer
            }
        }

        pub fn advance_frame<'a>(&mut self, init: &mut ::glinit::GLInit<'a>, events: &mut Events<'a>) -> bool {
            loop {
                match handle_event(init, events, &mut self.producer.producer, self.position) {
                    EventState::Done => return true,
                    EventState::Frame => return false,
                    EventState::NoFrame => { },
                }
            }
        }
    }
}
