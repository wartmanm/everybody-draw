#![macro_escape]

macro_rules! rolling_average_count (
    ($name:ident, $count:expr) => (
        pub struct $name<T> {
            pub entries: [T, ..$count],
            sum: T,
            count: uint,
            pos: uint,
        }

        #[allow(dead_code)]
        impl<T: ::core::ops::Sub<T, T> + ::core::num::Zero + ::core::ops::Div<f32, T> + ::core::kinds::Copy> $name<T> {
            pub fn new() -> $name<T> {
                $name {
                    entries: [::std::num::zero::<T>(), ..$count],
                    sum: ::std::num::zero::<T>(),
                    count: 0,
                    pos: 0,
                }
            }
            pub fn push(&mut self, value: T) -> T {
                if self.count < $count {
                    self.count += 1;
                } else {
                    self.sum = self.sum - self.entries[self.pos];
                }
                self.entries[self.pos] = value;
                self.sum = self.sum + value;
                self.pos = (self.pos + 1) % $count;
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
    )
);
