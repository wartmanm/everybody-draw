//extern crate core;
use core::prelude::*;
use core::mem;
//use core::clone::Clone;

use std::sync::spsc_queue;

use collections::vec::Vec;
use collections::SmallIntMap;
use collections::MutableMap;
use collections::MutableSeq;
use collections::Mutable;
use collections::Map;

use log::logi;
use motionevent;
use motionevent::append_motion_event;
use android::input::AInputEvent;

use point;
use point::{ShaderPaintPoint, Coordinate, PointEntry, PointConsumer, PointProducer, ShaderPointInfo};
use activestate;
use drawevent::Events;

use alloc::boxed::Box;

rolling_average_count!(RollingAverage16, 16)

//type LuaPointIter<'a> = ::core::slice::Windows<'a, ShaderPointInfo>;
//type LuaPointIter<'a> = Box<Iterator<::core::slice::Windows<'a, ShaderPointInfo>>+'static>;
//type LuaPointIter<'a> = Box<Iterator<&'a [ShaderPointInfo]>+'static>;
pub type LuaPointIter<'a> = ::core::iter::FlatMap<'a,&'a Vec<ShaderPointInfo>,::core::iter::Map<'a,&'a PointStorage,&'a Vec<ShaderPointInfo>,::core::iter::Map<'a,(uint,&'a PointStorage),&'a PointStorage,::collections::smallintmap::Entries<'a,PointStorage>>>,::core::slice::Windows<'a, ShaderPointInfo>>;

/// lifetime storage for a pointer's past state
struct PointStorage {
    info: Option<ShaderPaintPoint>, // FIXME can be replaced with queue[queue.len() - 1]
    sizeavg: RollingAverage16<f32>,
    speedavg: RollingAverage16<f32>,
    queue: Vec<ShaderPointInfo>,
}

#[allow(ctypes)]
pub struct MotionEventConsumer {
    consumer: PointConsumer,
    current_points: SmallIntMap<PointStorage>,
    //drawvec: Vec<ShaderPaintPoint>,
    point_counter: i32,
    point_count: i32,
    all_pointer_state: activestate::ActiveState,
}

pub struct MotionEventProducer {
    pointer_data: motionevent::Data,
    producer: PointProducer,
}

fn get_safe_data<'a, T>(data: *mut T) -> &'a mut T {
    unsafe { &mut *data }
}

//struct ChainAll<T> {
    //iters: &[T],
    //idx: uint,
//}

//impl <A, T: Iterator<A>> Iterator<A> for ChainAll<T> {
    //#[inline]
    //fn next(&mut self) -> Option<A> {

    

#[no_mangle]
pub extern fn create_motion_event_handler() -> (*mut MotionEventConsumer, *mut MotionEventProducer) {
    let (consumer, producer) = spsc_queue::queue::<PointEntry>(0);
    let handler = box MotionEventConsumer {
        consumer: consumer,
        current_points: SmallIntMap::new(),
        point_counter: 0, // unique value for each new pointer
        point_count: 0, // # of currently active pointers
        all_pointer_state: activestate::inactive,
    };
    let producer = box MotionEventProducer {
        producer: producer,
        pointer_data: motionevent::Data::new(),
    };
    logi("created statics");
    unsafe {
        let handlerptr: *mut MotionEventConsumer = mem::transmute(handler) ;
        let producerptr: *mut MotionEventProducer = mem::transmute(producer) ;
        (handlerptr, producerptr)
    }
}

#[no_mangle]
pub unsafe extern fn destroy_motion_event_handler(consumer: *mut MotionEventConsumer, producer: *mut MotionEventProducer) {
    let handler: Box<MotionEventConsumer> = mem::transmute(consumer);
    let producer: Box<MotionEventProducer> = mem::transmute(producer);
    mem::drop(handler);
    mem::drop(producer);
}

#[no_mangle]
//FIXME: needs meaningful name
pub extern fn jni_append_motion_event(s: &mut MotionEventProducer, evt: *const AInputEvent) {
    let s = get_safe_data(s);
    append_motion_event(&mut s.pointer_data, evt, &mut s.producer);
}

fn manhattan_distance(a: Coordinate, b: Coordinate) -> f32 {
    let x = if a.x > b.x { a.x - b.x } else { b.x - a.x };
    let y = if a.y > b.y { a.y - b.y } else { b.y - a.y };
    return if x > y { x } else { y };
}

pub fn run_interpolators(dimensions: (i32, i32), s: *mut MotionEventConsumer, events: &mut Events, drawvecs: &mut [Vec<ShaderPaintPoint>]) -> bool {
    let s = get_safe_data(s);
    gather_points(s, events);
    let interpolators = events.interpolators.as_slice();
    if s.current_points.len() > 0 {
        for interpolator in interpolators.iter() {
            let pointqueues = s.current_points.values().map(|v| &v.queue);
            let pointiter = pointqueues.flat_map(|q| q.as_slice().windows(2));
            //let windowed = pointqueues.map(|q| q.as_slice().windows(2));
            //let init = box windowed.next().unwrap() as LuaPointIter;
            //let pointiter = windowed.fold(init, |accum, elem| box accum.chain(elem) as LuaPointIter);
            interpolator.prep();
            run_lua_shader(dimensions, drawvecs, pointiter);
        }
        for (_, point) in s.current_points.iter_mut() {
            point.queue.clear();
        }
    }
    s.all_pointer_state = s.all_pointer_state.push(s.point_count > 0);
    s.all_pointer_state == activestate::stopping
}

#[no_mangle]
pub extern "C" fn next_point_from_lua(spi: &mut (&[Vec<ShaderPaintPoint>], LuaPointIter), points: &mut (ShaderPaintPoint, ShaderPaintPoint)) -> bool {
    let (ref mut _ignore, ref mut pi) = *spi;
    loop {
        match pi.next() {
            Some([point::Point(a), point::Point(b)]) => {
                *points = (a,b);
                return true;
            }
            None => {
                return false;
            }
            _ => {
                continue;
            }
        }
    }
}

fn gather_points(s: &mut MotionEventConsumer, e: &mut Events) {
    let ref mut queue = s.consumer;
    let ref mut current_points = s.current_points;
    loop {
        match queue.pop() {
            Some(point) => {
                e.pushpoint(point);
                let idx = point.index;
                let newpoint = point.entry;
                if !current_points.contains_key(&(idx as uint)) {
                    let mut newvec = Vec::new();
                    newvec.push(point::Stop);
                    current_points.insert(idx as uint, PointStorage {
                        info: None,
                        sizeavg: RollingAverage16::new(),
                        speedavg: RollingAverage16::new(),
                        queue: newvec,
                    });
                }
                let oldpoint = current_points.find_mut(&(idx as uint)).unwrap();
                match (oldpoint.info, newpoint) {
                    (Some(op), point::Point(np)) => {
                        let dist = manhattan_distance(op.pos, np.pos);
                        let avgsize = oldpoint.sizeavg.push(np.size);
                        let avgspeed = oldpoint.speedavg.push(dist);
                        let npdata = ShaderPaintPoint {
                            pos: np.pos,
                            time: np.time,
                            size: avgsize,
                            speed: avgspeed,
                            distance: op.distance + dist,
                            counter: op.counter,
                        };
                        oldpoint.info = Some(npdata);
                        oldpoint.queue.push(point::Point(npdata));
                    },
                    (_, point::Stop) => {
                        oldpoint.info = None;
                        oldpoint.sizeavg.clear();
                        oldpoint.speedavg.clear();
                        s.point_count -= 1;
                        oldpoint.queue.push(point::Stop);
                    },
                    (_, point::Point(p)) => {
                        let old_counter = s.point_counter;
                        s.point_counter += 1;
                        s.point_count += 1;
                        let npdata = ShaderPaintPoint {
                            pos: p.pos,
                            time: p.time,
                            size: p.size,
                            distance: 0f32,
                            speed: 0f32,
                            counter: old_counter as f32,
                        };
                        oldpoint.info = Some(npdata);
                        oldpoint.queue.push(point::Point(npdata));
                    },
                }
            },
            None => {
                return;
            }
        }
    }
}

fn run_lua_shader(dimensions: (i32, i32), drawvecs: &[Vec<ShaderPaintPoint>], iter: LuaPointIter) {
    let (x,y) = dimensions;
    unsafe {
        doInterpolateLua(x, y, &mut (drawvecs, iter));
    }
}


#[allow(non_snake_case)]
#[allow(ctypes)]
extern "C" {
    pub fn doInterpolateLua(x: i32, y: i32, statics: *mut (&[Vec<ShaderPaintPoint>], LuaPointIter));
}

#[no_mangle]
pub unsafe extern "C" fn pushrustvec(statics: &mut (&mut [Vec<ShaderPaintPoint>], &mut LuaPointIter), queue: i32, point: *const ShaderPaintPoint) {
    let (ref mut s, _) = *statics;
    s[queue as uint].push(*point);
}
