macro_rules! rolling_average_count (
    ($name:ident, $count:expr) => (
        pub struct $name<T> {
            pub entries: [T; $count],
            sum: T,
            count: usize,
            pos: usize,
        }

        #[allow(dead_code)]
        impl<T> $name<T>
        where T: ::core::ops::Add<T> + ::core::ops::Sub<T> + ::core::default::Default + ::core::ops::Div<f32> + ::core::marker::Copy
        + ::point::AsSelf<T>,
        <T as ::core::ops::Sub>::Output: ::point::AsSelf<T>,
        <T as ::core::ops::Add>::Output: ::point::AsSelf<T>,
        <T as ::core::ops::Div<f32>>::Output: ::point::AsSelf<T>, {
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
                    self.sum = *::point::as_self(&(self.sum - self.entries[self.pos]));
                }
                self.entries[self.pos] = value;
                self.sum = *::point::as_self(&(self.sum + value));
                self.pos = (self.pos + 1) % $count;
                self.get_average()
            }
            pub fn get_average(&self) -> T {
                *::point::as_self(&(self.sum / self.count as f32))
            }
            pub fn clear(&mut self) {
                self.sum = ::core::default::Default::default();
                self.count = 0;
                self.pos = 0;
            }
        }
    )
);
