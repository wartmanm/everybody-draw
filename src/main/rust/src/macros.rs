#![macro_escape]

macro_rules!  format (
    ($($arg:tt)*) => (
        format_args!(::std::fmt::format, $($arg)*)
    )
)

macro_rules! write(
    ($dst:expr, $($arg:tt)*) => ({
        let dst = &mut *$dst;
        format_args!(|args| { dst.write_fmt(args) }, $($arg)*)
    })
)

macro_rules! println(
    ($($arg:tt)*) => (format_args!(::std::io::stdio::println_args, $($arg)*))
)

macro_rules! cstr(
    ($str:expr) => (
        concat!($str, "\0").as_ptr() as *const ::libc::c_char
    )
)

macro_rules! panic(
    () => ({
        panic!("explicit panic")
    });
    ($msg:expr) => ({
        // static requires less code at runtime, more constant data
        static _FILE_LINE: (&'static str, uint) = (file!(), line!());
        ::log::log(
            concat!(file!(), ":", line!(), ": ", $msg),
            ::android::log::ANDROID_LOG_FATAL);
        ::std::rt::begin_unwind($msg, &_FILE_LINE)
    });
    ($fmt:expr, $($arg:tt)*) => ({
        // a closure can't have return type !, so we need a full
        // function to pass to format_args!, *and* we need the
        // file and line numbers right here; so an inner bare fn
        // is our only choice.
        //
        // LLVM doesn't tend to inline this, presumably because begin_unwind_fmt
        // is #[cold] and #[inline(never)] and because this is flagged as cold
        // as returning !. We really do want this to be inlined, however,
        // because it's just a tiny wrapper. Small wins (156K to 149K in size)
        // were seen when forcing this to be inlined, and that number just goes
        // up with the number of calls to panic!()
        //
        // The leading _'s are to avoid dead code warnings if this is
        // used inside a dead function. Just `#[allow(dead_code)]` is
        // insufficient, since the user may have
        // `#[forbid(dead_code)]` and which cannot be overridden.
        #[inline(always)]
        fn _run_fmt(fmt: &::std::fmt::Arguments) -> ! {
            static _FILE_LINE: (&'static str, uint) = (file!(), line!());
            ::std::rt::begin_unwind_fmt(fmt, &_FILE_LINE)
        }
        ::log::log(
            format!(concat!(file!(), ":", line!(), ": ", $fmt), $($arg)*).as_slice(),
            ::android::log::ANDROID_LOG_FATAL);
        format_args!(_run_fmt, $fmt, $($arg)*)
    });
)

macro_rules! try(
    ($e:expr) => (match $e { Ok(e) => e, Err(e) => return Err(e) })
)

macro_rules! try_opt(
    ($e:expr) => (match $e { Some(e) => e, None => return None })
)

macro_rules! assert(
    ($cond:expr) => (
        if !$cond {
            panic!(concat!(": assertion failed: ", stringify!($cond)))
        }
    );
    ($cond:expr, $($arg:expr),+) => (
        if !$cond {
            panic!($($arg),+)
        }
    );
)

macro_rules! debug_assert(
    ($($arg:tt)*) => (if cfg!(not(ndebug)) { assert!($($arg)*); })
)

macro_rules! assert_eq(
    ($given:expr , $expected:expr) => ({
        match (&($given), &($expected)) {
            (given_val, expected_val) => {
                // check both directions of equality....
                if !((*given_val == *expected_val) &&
                     (*expected_val == *given_val)) {
                    panic!("assertion failed: `(left == right) && (right == left)` \
                           (left: `{}`, right: `{}`)", *given_val, *expected_val)
                }
            }
        }
    })
)
