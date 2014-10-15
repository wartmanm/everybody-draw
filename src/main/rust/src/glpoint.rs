use core::prelude::*;
use core::mem;
use collections::vec::Vec;
use collections::{SmallIntMap, MutableMap, MutableSeq, Mutable, Map};
use alloc::boxed::Box;

use std::sync::spsc_queue;

use log::logi;
use motionevent;
use motionevent::append_motion_event;
use android::input::AInputEvent;

use point;
use point::{ShaderPaintPoint, Coordinate, PointEntry, PointConsumer, PointProducer};
use activestate;
use drawevent::Events;

rolling_average_count!(RollingAverage16, 16)

/// lifetime storage for a pointer's past state
struct PointStorage {
    info: Option<ShaderPaintPoint>,
    sizeavg: RollingAverage16<f32>,
    speedavg: RollingAverage16<Coordinate>,
}

#[allow(ctypes)]
pub struct MotionEventConsumer {
    consumer: PointConsumer,
    current_points: SmallIntMap<PointStorage>,
    point_counter: i32,
    point_count: i32,
    all_pointer_state: activestate::ActiveState,
}

pub struct MotionEventProducer {
    pointer_data: motionevent::Data,
    producer: PointProducer,
}

pub fn create_motion_event_handler() -> (Box<MotionEventConsumer>, Box<MotionEventProducer>) {
    let (consumer, producer) = spsc_queue::queue::<PointEntry>(0);
    let handler = box MotionEventConsumer {
        consumer: consumer,
        current_points: SmallIntMap::new(),
        point_counter: 0, // unique value for each new pointer
        point_count: 0, // # of currently active pointers
        all_pointer_state: activestate::INACTIVE,
    };
    let producer = box MotionEventProducer {
        producer: producer,
        pointer_data: motionevent::Data::new(),
    };
    logi("created statics");
    (handler, producer)
}

pub unsafe fn destroy_motion_event_handler(consumer: Box<MotionEventConsumer>, producer: Box<MotionEventProducer>) {
    mem::drop(consumer);
    mem::drop(producer);
}

//FIXME: needs meaningful name
pub fn jni_append_motion_event(s: &mut MotionEventProducer, evt: *const AInputEvent) {
    append_motion_event(&mut s.pointer_data, evt, &mut s.producer);
}

fn manhattan_distance(a: Coordinate, b: Coordinate) -> f32 {
    let x = if a.x > b.x { a.x - b.x } else { b.x - a.x };
    let y = if a.y > b.y { a.y - b.y } else { b.y - a.y };
    return if x > y { x } else { y };
}

impl MotionEventConsumer {
    pub fn frame_done(&mut self) -> bool {
        self.all_pointer_state = self.all_pointer_state.push(self.point_count > 0);
        self.all_pointer_state == activestate::STOPPING
    }
}
        
#[inline]
pub fn next_point(s: &mut MotionEventConsumer, e: &mut Events) -> (point::ShaderPointEvent, u8) {
    let ref mut queue = s.consumer;
    let ref mut current_points = s.current_points;
    match queue.pop() {
        Some(point) => {
            e.pushpoint(point);
            let idx = point.index;
            let newpoint = point.entry;
            if !current_points.contains_key(&(idx as uint)) {
                current_points.insert(idx as uint, PointStorage {
                    info: None,
                    sizeavg: RollingAverage16::new(),
                    speedavg: RollingAverage16::new(),
                });
            }
            let oldpoint = current_points.find_mut(&(idx as uint)).unwrap();
            let pointevent = match (oldpoint.info, newpoint) {
                (Some(op), point::Point(np)) => {
                    let dist = manhattan_distance(op.pos, np.pos);
                    let avgsize = oldpoint.sizeavg.push(np.size);
                    let avgspeed = oldpoint.speedavg.push(op.pos - np.pos);
                    let npdata = ShaderPaintPoint {
                        pos: np.pos,
                        time: np.time,
                        size: avgsize,
                        speed: avgspeed,
                        distance: op.distance + dist,
                        counter: op.counter,
                    };
                    oldpoint.info = Some(npdata);
                    point::Move(op, npdata)
                },
                (_, point::Stop) => {
                    oldpoint.info = None;
                    oldpoint.sizeavg.clear();
                    oldpoint.speedavg.clear();
                    s.point_count -= 1;
                    point::Up
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
                        speed: Coordinate { x: 0f32, y: 0f32 },
                        counter: old_counter as f32,
                    };
                    oldpoint.info = Some(npdata);
                    point::Down(npdata)
                },
            };
            (pointevent, idx as u8)
        },
        None => {
            (point::NoEvent, 0u8)
        }
    }
}

#[inline]
pub fn push_line(drawvec: &mut Vec<ShaderPaintPoint>, a: &ShaderPaintPoint, b: &ShaderPaintPoint) {
    let distx = if (*a).pos.x > (*b).pos.x { (*a).pos.x - (*b).pos.x } else { (*b).pos.x - (*a).pos.x };
    let disty = if (*a).pos.y > (*b).pos.y { (*a).pos.y - (*b).pos.y } else { (*b).pos.y - (*a).pos.y };
    let count = if distx > disty { distx } else { disty } as i32;
    let timescale = 10f32;
    let stepx = ((*b).pos.x - (*a).pos.x) / count as f32;
    let stepy = ((*b).pos.y - (*a).pos.y) / count as f32;
    let steptime = ((*b).time - (*a).time) / (count as f32 * timescale);
    let stepsize = ((*b).size - (*a).size) / count as f32;
    let stepspeedx = ((*b).speed.x - (*a).speed.x) / count as f32;
    let stepspeedy = ((*b).speed.y - (*a).speed.y) / count as f32;
    let stepdistance = ((*b).distance - (*a).distance) / count as f32;
    let mut addpoint = *a;
    addpoint.time = (addpoint.time / timescale) % 1f32;
    for _ in range(0, count) {
        drawvec.push(addpoint);
        addpoint.pos.x += stepx;
        addpoint.pos.y += stepy;
        addpoint.time += steptime;
        addpoint.time = if addpoint.time > 1f32 { addpoint.time - 1f32 } else { addpoint.time };
        addpoint.size += stepsize;
        addpoint.speed.x += stepspeedx;
        addpoint.speed.y += stepspeedy;
        addpoint.distance += stepdistance;
    }
}
