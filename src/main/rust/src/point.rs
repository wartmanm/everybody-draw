// TODO: more meaningful names
use std::sync::mpsc;
use core::ops::{Add, Div, Sub, Mul};

#[derive(Clone, Debug, PartialEq, Copy, Default)]
#[repr(C)]
pub struct Coordinate {
    pub x: f32,
    pub y: f32,
}

pub trait AsSelf<T> {
    fn as_self(&self) -> &T;
}
impl AsSelf<Coordinate> for Coordinate {
    #[inline(always)]
    fn as_self(&self) -> &Coordinate { self }
}
impl AsSelf<f32> for f32 {
    #[inline(always)]
    fn as_self(&self) -> &f32 { self }
}

#[inline(always)]
pub fn as_self<T, U: AsSelf<T>>(u: &U) -> &T { u.as_self() }

/// Holds data from motionevent entries.
#[derive(Clone, Debug, PartialEq, Copy)]
#[repr(C)]
pub struct PaintPoint {
    pub pos: Coordinate,
    pub time: f32, // floating-point seconds
    pub size: f32,
}

/// Holds raw data used for pointshader attribs.
/// These fields overlap with PaintPoint somewhat but aren't necessarily directly sourced from one
/// so adding it as a child doesn't seem ideal
#[derive(Clone, Debug, Copy)]
#[repr(C)]
pub struct ShaderPaintPoint {
    pub pos: Coordinate,
    pub time: f32,
    pub size: f32,
    pub speed: Coordinate,
    pub distance: f32,
    pub counter: f32, // could become a uniform? only floating-point allowed for attribs
}

/// Pointer state, corresponding to a single motionevent historical entry
/// Stop, unsurprisingly, indicates a pointer has been lifted
/// this enables us to use raw motionevent pointer ids, which get recycled regularly
/// it's arguably simpler than ensuring each pointer gets a unique queue for its entire
/// lifetime and maintaining an up-to-date pointer id -> queue mapping
/// FrameStop indicates that we should stop reading 
#[derive(PartialEq, Copy, Clone)]
pub enum PointInfo {
    Stop,
    FrameStop,
    Point(PaintPoint),
}

#[derive(Copy)]
pub enum ShaderPointEvent {
    Move(ShaderPaintPoint, ShaderPaintPoint),
    Down(ShaderPaintPoint),
    Up(ShaderPaintPoint),
    NoEvent,
}

/// A single entry in the point queue.
#[derive(PartialEq, Copy, Clone)]
pub struct PointEntry {
    pub index: i32,
    pub entry: PointInfo,
}

pub type PointConsumer = mpsc::Receiver<PointEntry>;
pub type PointProducer = mpsc::Sender<PointEntry>;

impl Add<Coordinate> for Coordinate {
    type Output = Coordinate;
    #[inline(always)]
    fn add(self, rhs: Coordinate) -> Coordinate {
        Coordinate { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}
impl Sub<Coordinate> for Coordinate {
    type Output = Coordinate;
    #[inline(always)]
    fn sub(self, rhs: Coordinate) -> Coordinate {
        Coordinate { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}
impl Div<f32> for Coordinate {
    type Output = Coordinate;
    #[inline(always)]
    fn div(self, rhs: f32) -> Coordinate {
        Coordinate { x: self.x / rhs, y: self.y / rhs }
    }
}
impl Mul<f32> for Coordinate {
    type Output = Coordinate;
    #[inline(always)]
    fn mul(self, rhs: f32) -> Coordinate {
        Coordinate { x: self.x * rhs, y: self.y * rhs }
    }
}
