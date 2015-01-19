#![macro_escape]
macro_rules! native_method(
    ($name:expr, $sig:expr, $fn_ptr:expr) => (
        JNINativeMethod {
            name: cstr!($name),
            signature: cstr!($sig),
            fnPtr: $fn_ptr as *mut ::libc::c_void,
        }
    )
)

