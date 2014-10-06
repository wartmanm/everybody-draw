use core::prelude::*;

use log::logi;
use android::input::*;

use collections::{SmallIntMap, MutableSeq, MutableMap, Map};

use point;
use point::{PaintPoint, Coordinate, PointEntry, PointProducer};
use activestate;

static AMOTION_EVENT_ACTION_POINTER_INDEX_SHIFT: uint = 8;

// TODO: consider eliminating entirely and putting faith in ACTION_POINTER_UP/DOWN
type PointerState = SmallIntMap<activestate::ActiveState>;
pub struct Data {
    pointer_states: PointerState,
}

impl Data {
    pub fn new() -> Data {
        Data { pointer_states: SmallIntMap::new() }
    }
}

pub fn append_motion_event(data: &mut Data, evt: *const AInputEvent, queue: &mut PointProducer) -> () {
    let active = &mut data.pointer_states;
    for (_, state) in active.iter_mut() {
        *state = state.push(false);
    }

    match unsafe { AInputEvent_getType(evt) } as u32 {
        AINPUT_EVENT_TYPE_KEY => { logi("got key event??"); return; },
        _ => { }
    }
    let full_action = unsafe { AMotionEvent_getAction(evt) } as u32;
    let (action_event, action_index): (u32, u32) = (full_action & AMOTION_EVENT_ACTION_MASK, (full_action & AMOTION_EVENT_ACTION_POINTER_INDEX_MASK) >> AMOTION_EVENT_ACTION_POINTER_INDEX_SHIFT);
    let action_id = unsafe { AMotionEvent_getPointerId(evt, action_index) };
    match action_event {
        AMOTION_EVENT_ACTION_DOWN => {
            logi!("ACTION_DOWN: {}", action_id);
            push_stops(queue, active); // in case it's not paired with an action_up
            push_moves(queue, active, evt);
        }
        AMOTION_EVENT_ACTION_UP => {
            logi!("ACTION_UP: {}", action_id);
            push_stops(queue, active);
        }
        AMOTION_EVENT_ACTION_CANCEL => {
            logi!("ACTION_CANCEL: {}", action_id);
            push_stops(queue, active);
        }
        AMOTION_EVENT_ACTION_POINTER_UP => {
            logi!("ACTION_POINTER_UP: {}", action_id);
            make_active(queue, active, action_id, false);
            push_moves(queue, active, evt);
        }
        AMOTION_EVENT_ACTION_POINTER_DOWN => {
            logi!("ACTION_POINTER_DOWN: {}", action_id);
            make_active(queue, active, action_id, false); // in case it's not paired with an action_pointer_up
            push_moves(queue, active, evt);
        }
        AMOTION_EVENT_ACTION_MOVE => {
            push_moves(queue, active, evt);
        },
        unknown => {
            logi!("unknown action event: {}", unknown);
        }
    }
}

fn push_moves(queue: &mut PointProducer, active: &mut PointerState, evt: *const AInputEvent) {
    let ptrcount = unsafe { AMotionEvent_getPointerCount(evt) };
    let historycount = unsafe { AMotionEvent_getHistorySize(evt) };
    for ptr in range(0, ptrcount) {
        let id = unsafe { AMotionEvent_getPointerId(evt, ptr) };
        for hist in range(0, historycount) {
            push_historical_point(queue, evt, id, ptr, hist);
        }
        push_current_point(queue, evt, id, ptr);
        make_active(queue, active, id, true);
    }
    push_stops(queue, active);
}

fn make_active(queue: &mut PointProducer, active: &mut PointerState, id: i32, newstate: bool) {
    let updated = active.find(&(id as uint)).unwrap_or(&activestate::INACTIVE).push(newstate);
    active.insert(id as uint, updated);
    if updated == activestate::STOPPING {
        queue.push(PointEntry { index: id, entry: point::Stop });
    }
}

fn push_historical_point(queue: &mut PointProducer, evt: *const AInputEvent, id: i32, ptr: u32, hist: u32) {
    queue.push(PointEntry { index: id, entry: point::Point(PaintPoint {
        pos: Coordinate {
             x: unsafe { AMotionEvent_getHistoricalX(evt, ptr, hist) },
             y: unsafe { AMotionEvent_getHistoricalY(evt, ptr, hist) },
        },
        time: (unsafe { AMotionEvent_getHistoricalEventTime(evt, hist) } / 1000) as f32 / 1000000f32,
        size: unsafe { AMotionEvent_getHistoricalSize(evt, ptr, hist) },
    })});
}

fn push_current_point(queue: &mut PointProducer, evt: *const AInputEvent, id: i32, ptr: u32) {
    queue.push(PointEntry { index: id, entry: point::Point(PaintPoint {
        pos: Coordinate {
            x: unsafe { AMotionEvent_getX(evt, ptr) },
            y: unsafe { AMotionEvent_getY(evt, ptr) },
        },
        time: (unsafe { AMotionEvent_getEventTime(evt) } / 1000) as f32 / 1000000f32,
        size: unsafe { AMotionEvent_getSize(evt, ptr) },
    })});
}

fn push_stops(queue: &mut PointProducer, active: &mut PointerState) {
    for (idx, active) in active.iter_mut() {
        if *active == activestate::STOPPING {
            queue.push(PointEntry { index: idx as i32, entry: point::Stop });
        }
    }
}
