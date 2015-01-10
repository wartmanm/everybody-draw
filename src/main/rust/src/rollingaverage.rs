macro_rules! rolling_average_count (
    ($name:ident, $count:expr) => (
        pub struct $name<T> {
            pub entries: [T; $count],
            sum: T,
            count: uint,
            pos: uint,
        }

        #[allow(dead_code)]
        impl<T> $name<T>
        where T: ::core::ops::Add<T> + ::core::ops::Sub<T> + ::core::default::Default + ::core::ops::Div<f32> + ::core::marker::Copy
        + ::core::num::NumCast,
        <T as ::core::ops::Sub>::Output: ::core::num::NumCast,
        <T as ::core::ops::Add>::Output: ::core::num::NumCast,
        <T as ::core::ops::Div<f32>>::Output: ::core::num::NumCast {
            pub fn new() -> $name<T> {
                $name {
                    entries: [::core::default::Default::default(); $count],
                    sum: ::core::default::Default::default(),
                    count: 0,
                    pos: 0,
                }
            }
            pub fn push(&mut self, value: T) -> T {
                if self.count < $count {
                    self.count += 1;
                } else {
                    self.sum = ::core::num::cast(self.sum - self.entries[self.pos]).unwrap();
                }
                self.entries[self.pos] = value;
                self.sum = ::core::num::cast(self.sum + value).unwrap();
                self.pos = (self.pos + 1) % $count;
                self.get_average()
            }
            pub fn get_average(&self) -> T {
                ::core::num::cast(self.sum / self.count as f32).unwrap()
            }
            pub fn clear(&mut self) {
                self.sum = ::core::default::Default::default();
                self.count = 0;
                self.pos = 0;
            }
        }
    )
);
