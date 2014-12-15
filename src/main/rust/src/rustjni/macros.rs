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

macro_rules! try_or_throw (
    ($env:expr, $errclass:expr, $e:expr, $ret:expr) => ({
        match $e {
            Ok(e) => e,
            Err(e) => {
                let errmsg = str_to_jstring(env, format!("{}", e).as_slice()).as_jvalue();
                let err = $errclass.construct(env, [errmsg].as_mut_slice());
                ((**$env).Throw)(env, err);
                return $ret;
            },
        }
    });
    ($env:expr, $errclass:expr, $e:expr) => {
        match $e {
            Ok(e) => e,
            Err(e) => {
                let errmsg = str_to_jstring($env, format!("{}", e).as_slice()).as_jvalue();
                let err = $errclass.construct($env, [errmsg].as_mut_slice());
                ((**$env).Throw)($env, err);
                return;
            },
        }
    };
)

