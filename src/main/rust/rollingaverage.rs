use core::prelude::*;

use collections::vec::Vec;
use collections::Mutable;

pub struct RollingAverage<T> {
    pub entries: Vec<T>,
    sum: T,
    count: uint,
    pos: uint,
}

#[allow(dead_code)]
impl<T: Add<T,T> + Sub<T, T> + ::std::num::Zero + Div<f32, T> + Clone> RollingAverage<T> {
    pub fn new(count: uint) -> RollingAverage<T> {
        RollingAverage {
            entries: Vec::from_elem(count, ::std::num::zero::<T>()),
            sum: ::std::num::zero::<T>(),
            count: 0,
            pos: 0,
        }
    }
    pub fn push(&mut self, value: T) -> T {
        let len = self.entries.len();
        if self.count < len {
            self.count += 1;
        } else {
            self.sum = self.sum - self.entries[self.pos];
        }
        *self.entries.get_mut(self.pos) = value.clone();
        self.sum = self.sum + value;
        self.pos = (self.pos + 1) % self.entries.len();
        self.get_average()
    }
    pub fn get_average(&self) -> T {
        self.sum / self.count as f32
    }
    pub fn clear(&mut self) {
        self.sum = ::std::num::zero::<T>();
        self.count = 0;
        self.pos = 0;
    }
}
