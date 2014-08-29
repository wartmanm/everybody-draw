//extern crate core;
use core::prelude::*;
use core::mem;

use std::sync::{Once, ONCE_INIT, spsc_queue};

use collections::vec::Vec;
use collections::SmallIntMap;
use collections::MutableMap;
use collections::MutableSeq;
use collections::Mutable;
use collections::Map;

use log::logi;
use motionevent::append_motion_event;
use android::input::AInputEvent;

use opengles::gl2::*;

use pointshader::{PointShader};
use glcommon::check_gl_error;
use gltexture::Texture;
use point;
use point::{ShaderPaintPoint, Coordinate, PointEntry, PointConsumer, PointProducer};
use dropfree::DropFree;
use matrix::Matrix;
use luascript::LuaScript;
use activestate;

rolling_average_count!(RollingAverage16, 16)

/// lifetime storage for a pointer's past state
struct PointStorage {
    info: Option<ShaderPaintPoint>,
    sizeavg: RollingAverage16<f32>,
    speedavg: RollingAverage16<f32>,
}

pub struct RustStatics {
    consumer: PointConsumer,
    producer: PointProducer,
    currentPoints: SmallIntMap<PointStorage>,
    drawvec: Vec<ShaderPaintPoint>,
    pointCounter: i32,
    point_count: i32,
    pointer_state: activestate::ActiveState,
}

static mut dataRef: DropFree<RustStatics> = DropFree(0 as *mut RustStatics);
static mut pathinit: Once = ONCE_INIT;

fn do_path_init() -> () {
    unsafe {
        pathinit.doit(|| {
            let (consumer, producer) = spsc_queue::queue::<PointEntry>(0);
            dataRef = DropFree::new(RustStatics {
                consumer: consumer,
                producer: producer,
                currentPoints: SmallIntMap::new(),
                drawvec: Vec::new(),
                pointCounter: 0,
                point_count: 0,
                pointer_state: activestate::inactive,
            });
            logi("created statics");
        });
    }
}

fn get_statics<'a>() -> &'a mut RustStatics {
    do_path_init();
    unsafe { dataRef.get_mut() }
}

#[no_mangle]
//FIXME: needs meaningful name
pub extern fn jni_append_motion_event(evt: *const AInputEvent) {
    let s = get_statics();
    append_motion_event(evt, &mut s.producer);
}

fn manhattan_distance(a: Coordinate, b: Coordinate) -> f32 {
    let x = if a.x > b.x { a.x - b.x } else { b.x - a.x };
    let y = if a.y > b.y { a.y - b.y } else { b.y - a.y };
    return if x > y { x } else { y };
}

#[allow(dead_code)]
fn append_points(a: ShaderPaintPoint, b: ShaderPaintPoint, c: &mut Vec<ShaderPaintPoint>, count: uint) -> () {
    // transform seconds from [0..timescale] to [0..1]
    // this is done here to avoid rollover resulting in negative steptime
    // it might be better to leave it alone and do fract() in the vertex shader?
    let timescale = 10f32;
    let stepx = (b.pos.x - a.pos.x) / count as f32;
    let stepy = (b.pos.y - a.pos.y) / count as f32;
    let steptime = (b.time - a.time) / (count as f32 * timescale);
    let stepsize = (b.size - a.size) / count as f32;
    let stepspeed = (b.speed - a.speed) / count as f32;
    let stepdistance = (b.distance - a.distance) / count as f32;
    let mut addPoint = a;
    addPoint.time = (addPoint.time / timescale) % 1f32;
    for _ in range(0, count) {
        c.push(addPoint);
        addPoint.pos.x += stepx;
        addPoint.pos.y += stepy;
        addPoint.time += steptime;
        addPoint.time = if addPoint.time > 1f32 { addPoint.time - 1f32 } else { addPoint.time };
        addPoint.size += stepsize;
        addPoint.speed += stepspeed;
        addPoint.distance += stepdistance;
    }
}

pub fn draw_path(framebuffer: GLuint, shader: &PointShader, interpolator: &LuaScript, matrix: *mut f32, color: [f32, ..3], brush: &Texture, backBuffer: &Texture) -> bool {
    let s = get_statics();
    s.drawvec.clear();

    interpolator.prep();
    run_lua_shader(backBuffer.dimensions, s);

    let ref mut pointvec = s.drawvec;
    if pointvec.len() > 0 {
        bind_framebuffer(FRAMEBUFFER, framebuffer);
        let safe_matrix: &Matrix = unsafe { mem::transmute(matrix) };
        shader.prep(safe_matrix.as_slice(), pointvec.as_slice(), color, brush, backBuffer);
        draw_arrays(POINTS, 0, pointvec.len() as i32);
        check_gl_error("draw_arrays");
    }
    s.pointer_state = s.pointer_state.push(s.point_count > 0);
    s.pointer_state == activestate::stopping
}

#[no_mangle]
pub extern "C" fn next_point_from_lua(s: &mut RustStatics, points: &mut (ShaderPaintPoint, ShaderPaintPoint)) -> bool {
    let ref mut queue = s.consumer;
    let ref mut currentPoints = s.currentPoints;
    loop {
        match queue.pop() {
            Some(point) => {
                let idx = point.index;
                let newpoint = point.entry;
                if !currentPoints.contains_key(&(idx as uint)) {
                    currentPoints.insert(idx as uint, PointStorage {
                        info: None,
                        sizeavg: RollingAverage16::new(),
                        speedavg: RollingAverage16::new(),
                    });
                }
                let oldpoint = currentPoints.find_mut(&(idx as uint)).unwrap();
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
                        *points = (op, npdata);
                        return true;
                    },
                    (_, point::Stop) => {
                        oldpoint.info = None;
                        oldpoint.sizeavg.clear();
                        oldpoint.speedavg.clear();
                        s.point_count -= 1;
                    },
                    (_, point::Point(p)) => {
                        let oldCounter = s.pointCounter;
                        s.pointCounter += 1;
                        s.point_count += 1;
                        oldpoint.info = Some(ShaderPaintPoint {
                            pos: p.pos,
                            time: p.time,
                            size: p.size,
                            distance: 0f32,
                            speed: 0f32,
                            counter: oldCounter as f32,
                        });

                    },
                }
            },
            None => {
                return false;
            }
        }
    }
}

fn run_lua_shader(dimensions: (i32, i32), statics: &mut RustStatics) {
    let (x,y) = dimensions;
    unsafe {
        doInterpolateLua(x, y, statics);
    }
}


#[allow(non_snake_case_functions)]
extern "C" {
    pub fn doInterpolateLua(x: i32, y: i32, statics: *mut RustStatics);
}

#[no_mangle]
pub unsafe extern "C" fn pushrustvec(statics: &mut RustStatics, point: *const ShaderPaintPoint) {
    statics.drawvec.push(*point);
}

