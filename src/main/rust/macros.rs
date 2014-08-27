#![macro_escape]

macro_rules!  format (
    ($($arg:tt)*) => (
        format_args!(::std::fmt::format, $($arg)*)
    )
)

macro_rules! write(
    ($dst:expr, $($arg:tt)*) => ({
        format_args_method!($dst, write_fmt, $($arg)*)
    })
)
