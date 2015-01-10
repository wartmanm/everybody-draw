#![macro_escape]

macro_rules! format {
    ($($arg:tt)*) => ($crate::fmt::format(format_args!($($arg)*)))
}

macro_rules! write {
    ($dst:expr, $($arg:tt)*) => ((&mut *$dst).write_fmt(format_args!($($arg)*)))
}

macro_rules! println {
    ($($arg:tt)*) => ($crate::io::stdio::println_args(format_args!($($arg)*)))
}

macro_rules! cstr {
    ($str:expr) => ({
        concat!($str, "\0").as_ptr() as *const ::libc::c_char
    })
}

macro_rules! panic {
    () => ({
        panic!("explicit panic")
    });
    ($msg:expr) => ({
        $crate::rt::begin_unwind($msg, {
            // static requires less code at runtime, more constant data
            static _FILE_LINE: (&'static str, usize) = (file!(), line!());
            &_FILE_LINE
        })
    });
    ($fmt:expr, $($arg:tt)+) => ({
        $crate::rt::begin_unwind_fmt(format_args!($fmt, $($arg)+), {
            // The leading _'s are to avoid dead code warnings if this is
            // used inside a dead function. Just `#[allow(dead_code)]` is
            // insufficient, since the user may have
            // `#[forbid(dead_code)]` and which cannot be overridden.
            ::log::log(
                format!(concat!(file!(), ":", line!(), ": ", $fmt), $($arg)*).as_slice(),
                ::android::log::ANDROID_LOG_FATAL);
            static _FILE_LINE: (&'static str, usize) = (file!(), line!());
            &_FILE_LINE
        })
    });
}

macro_rules! try {
    ($expr:expr) => (match $expr {
        $crate::result::Result::Ok(val) => val,
        $crate::result::Result::Err(err) => {
            return $crate::result::Result::Err($crate::error::FromError::from_error(err))
        }
    })
}

macro_rules! try_opt(
    ($e:expr) => (match $e { Some(e) => e, None => return None })
);

macro_rules! assert {
    ($cond:expr) => (
        if !$cond {
            panic!(concat!("assertion failed: ", stringify!($cond)))
        }
    );
    ($cond:expr, $($arg:tt)+) => (
        if !$cond {
            panic!($($arg)+)
        }
    );
}

macro_rules! debug_assert {
    ($($arg:tt)*) => (if cfg!(not(ndebug)) { assert!($($arg)*); })
}

macro_rules! assert_eq {
    ($left:expr , $right:expr) => ({
        match (&($left), &($right)) {
            (left_val, right_val) => {
                // check both directions of equality....
                if !((*left_val == *right_val) &&
                     (*right_val == *left_val)) {
                    panic!("assertion failed: `(left == right) && (right == left)` \
                           (left: `{:?}`, right: `{:?}`)", *left_val, *right_val)
                }
            }
        }
    })
}
