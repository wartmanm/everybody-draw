#![macro_escape]
macro_rules!  format (
    ($($arg:tt)*) => (
        format_args!(::std::fmt::format, $($arg)*)
    )
)
