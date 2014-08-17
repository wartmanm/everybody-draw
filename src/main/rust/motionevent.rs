use core::prelude::*;

use log::logi;
use android::input::*;

use std::sync::{Once, ONCE_INIT};

use collections::{SmallIntMap, MutableSeq, MutableMap, Map};

use point;
use point::{PaintPoint, Coordinate, PointEntry, PointProducer};
use dropfree::DropFree;

static AMOTION_EVENT_ACTION_POINTER_INDEX_SHIFT: uint = 8;

// TODO: consider eliminating entirely and putting faith in ACTION_POINTER_UP/DOWN
mod activestate {
    #![allow(unused_variable, dead_code)]
    static newmask: u8 = 0x01;
    static oldmask: u8 = 0x02;

    pub static starting: ActiveState = ActiveState(newmask);
    pub static stopping: ActiveState  = ActiveState(oldmask);
    pub static continuing: ActiveState  = ActiveState(newmask | oldmask);
    pub static inactive: ActiveState = ActiveState(0);
    #[deriving(Eq, PartialEq)]
    pub struct ActiveState(u8);

    impl ActiveState {
        #[inline]
        pub fn push(self, newstate: bool) -> ActiveState {
            let ActiveState(state) = self;
            ActiveState(((state << 1) & oldmask) | newstate as u8)
        }
    }
}

type Data = SmallIntMap<activestate::ActiveState>;

static mut dataRef: DropFree<Data> = DropFree(0 as *mut Data) ;
fn get_safe_data<'a>() -> &'a mut Data {
    do_data_init();
    unsafe { dataRef.get_mut() }
}

static mut datainit: Once = ONCE_INIT;
fn do_data_init() {
    unsafe {
        datainit.doit(|| dataRef = DropFree::new(SmallIntMap::new()) );
    }
}

pub fn append_motion_event(evt: *const AInputEvent, queue: &mut PointProducer) -> () {
    let active = get_safe_data();
    for (_, state) in active.mut_iter() {
        *state = state.push(false);
    }

    match unsafe { AInputEvent_getType(evt) } as u32 {
        AINPUT_EVENT_TYPE_KEY => { logi("got key event??"); return; },
        _ => { }
    }
    let fullAction = unsafe { AMotionEvent_getAction(evt) } as u32;
    let (actionEvent, actionIndex): (u32, u32) = (fullAction & AMOTION_EVENT_ACTION_MASK, (fullAction & AMOTION_EVENT_ACTION_POINTER_INDEX_MASK) >> AMOTION_EVENT_ACTION_POINTER_INDEX_SHIFT);
    let actionId = unsafe { AMotionEvent_getPointerId(evt, actionIndex) };
    match actionEvent {
        AMOTION_EVENT_ACTION_DOWN => {
            logi!("ACTION_DOWN: {}", actionId);
            push_stops(queue, active); // in case it's not paired with an action_up
            push_moves(queue, active, evt);
        }
        AMOTION_EVENT_ACTION_UP => {
            logi!("ACTION_UP: {}", actionId);
            push_stops(queue, active);
        }
        AMOTION_EVENT_ACTION_CANCEL => {
            logi!("ACTION_CANCEL: {}", actionId);
            push_stops(queue, active);
        }
        AMOTION_EVENT_ACTION_POINTER_UP => {
            logi!("ACTION_POINTER_UP: {}", actionId);
            make_active(queue, active, actionId, false);
            push_moves(queue, active, evt);
        }
        AMOTION_EVENT_ACTION_POINTER_DOWN => {
            logi!("ACTION_POINTER_DOWN: {}", actionId);
            make_active(queue, active, actionId, false); // in case it's not paired with an action_pointer_up
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

fn push_moves(queue: &mut PointProducer, active: &mut Data, evt: *const AInputEvent) {
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

fn make_active(queue: &mut PointProducer, active: &mut Data, id: i32, newstate: bool) {
    let updated = active.find(&(id as uint)).unwrap_or(&activestate::inactive).push(newstate);
    active.insert(id as uint, updated);
    if updated == activestate::stopping {
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

fn push_stops(queue: &mut PointProducer, active: &mut Data) {
    for (idx, active) in active.mut_iter() {
        if *active == activestate::stopping {
            queue.push(PointEntry { index: idx as i32, entry: point::Stop });
        }
    }
}

