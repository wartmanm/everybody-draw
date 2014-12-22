use core::prelude::*;
use alloc::boxed::Box;
use collections::string::String;
use core::{ptr, mem, raw, fmt};
use core::ptr::RawMutPtr;
use core::any::{Any, AnyRefExt};
use core::fmt::Show;
use core::iter;
use libc::{c_void, c_char};
use core::borrow::IntoCow;
use collections::vec::Vec;
use collections::str::MaybeOwned;

use jni::{jobject, jclass, jmethodID, jfieldID, JNIEnv, jint, jstring, jvalue, JNINativeMethod, JavaVM};
#[cfg(target_word_size = "64")] use jni::jlong;
use jni_constants::*;

use glinit::GLInit;
use drawevent::Events;
use glcommon::MString;
use libc::types::os::arch::posix88::pid_t;

use lua_callbacks::LuaCallbackType;
use lua_geom::LuaInterpolatorState;

use std;

extern "C" {
    pub fn gettid() -> pid_t;
}

mod macros;
pub mod texturethread;
pub mod android_bitmap;
pub mod gldataclasses;
pub mod motionevent;


#[cfg(target_word_size = "32")]
#[allow(non_camel_case_types)]
pub type jpointer = jint;
#[cfg(target_word_size = "64")]
#[allow(non_camel_case_types)]
pub type jpointer = jlong;

struct CaseClass {
    constructor: jmethodID,
    class: jclass,
}

static mut GL_EXCEPTION: CaseClass = CaseClass { constructor: 0 as jmethodID, class: 0 as jclass };


struct GLInitEvents<'a> {
    glinit: GLInit<'a>,
    events: Events<'a>,
    jni_undo_callback: JNIUndoCallback,
    owning_thread: pid_t,
    //lua: LuaInterpolatorState<LuaCallbackType>,
}

#[deriving(Copy)]
pub struct JNIUndoCallback {
    callback_obj: jobject,
    callback_method: jmethodID,
}

pub struct JNICallbackClosure<'a> {
    undo_callback: &'a JNIUndoCallback,
    env: *mut JNIEnv,
}

impl JNIUndoCallback {
    pub unsafe fn new(env: *mut JNIEnv, obj: jobject) -> JNIUndoCallback {
        let obj = ((**env).NewGlobalRef)(env, obj);
        let objclass = ((**env).GetObjectClass)(env, obj);
        let method = ((**env).GetMethodID)(env, objclass, cstr!("undoBufferChanged"), cstr!("(I)V"));
        JNIUndoCallback { callback_obj: obj, callback_method: method }
    }

    pub unsafe fn call(&self, env: *mut JNIEnv, new_undo_size: i32) {
        ((**env).CallVoidMethod)(env, self.callback_obj, self.callback_method, new_undo_size as jint);
    }

    pub unsafe fn destroy(self, env: *mut JNIEnv) {
        ((**env).DeleteGlobalRef)(env, self.callback_obj);
    }
    pub fn create_closure(&self, env: *mut JNIEnv) -> JNICallbackClosure {
        JNICallbackClosure { undo_callback: self, env: env }
    }
}

impl CaseClass {
    pub unsafe fn new(env: *mut JNIEnv, name: *const c_char, sig: *const c_char) -> CaseClass {
        let class = ((**env).FindClass)(env, name);
        let constructor = ((**env).GetMethodID)(env, class, cstr!("<init>"), sig);
        let globalclass = ((**env).NewGlobalRef)(env, class);

        CaseClass { constructor: constructor, class: globalclass }
    }
    pub unsafe fn construct(&self, env: *mut JNIEnv, arg: &mut [jvalue]) -> jobject {
        ((**env).NewObjectA)(env, self.class, self.constructor, arg.as_mut_ptr())
    }
    pub unsafe fn destroy(&mut self, env: *mut JNIEnv) {
        ((**env).DeleteGlobalRef)(env, self.class);
    }
}

unsafe fn get_string(env: *mut JNIEnv, string: jstring) -> Option<String> {
    let string = try_opt!(string.as_mut());
    let c = try_opt!(((**env).GetStringChars)(env, string, ptr::null_mut()).as_ref());
    let len = ((**env).GetStringLength)(env, string);
    let strslice: &[u16] = mem::transmute(raw::Slice { data: c, len: len as uint });
    let ruststr = String::from_utf16(strslice);
    ((**env).ReleaseStringChars)(env, string as jstring, strslice.as_ptr());
    Some(try_opt!(ruststr))
}

unsafe fn get_mstring(env: *mut JNIEnv, string: jstring) -> Option<MString> {
    match get_string(env, string) {
        Some(s) => Some(s.into_cow()),
        None => None,
    }
}

unsafe fn str_to_jstring(env: *mut JNIEnv, s: &str) -> jstring {
    let u16msg: Vec<u16> = s.utf16_units().collect();
    ((**env).NewString)(env, u16msg.as_ptr(), u16msg.len() as i32)
}

pub unsafe fn register_classmethods(env: *mut JNIEnv, classname: *const i8, methods: &[JNINativeMethod]) {
    let class = ((**env).FindClass)(env, classname);
    ((**env).RegisterNatives)(env, class, methods.as_ptr(), methods.len() as i32);
}

fn get_safe_data<'a>(data: jpointer) -> &'a mut GLInitEvents<'a> {
    unsafe {
        let data: &'a mut GLInitEvents<'a> = mem::transmute(data);
        assert_eq!(gettid(), data.owning_thread);
        data
    }
}


#[cfg(target_word_size = "32")]
#[inline(always)]
unsafe fn get_jpointer(env: *mut JNIEnv, obj: jobject, field: jfieldID) -> jpointer {
    ((**env).GetIntField)(env, obj, field)
}
#[cfg(target_word_size = "64")]
#[inline(always)]
unsafe fn get_jpointer(env: *mut JNIEnv, obj: jobject, field: jfieldID) -> jpointer {
    ((**env).GetLongField)(env, obj, field)
}

//#[cfg(target_word_size = "32")]
//#[inline(always)]
//fn set_jpointer(env: *mut JNIEnv, obj: jobject, field: jfieldID) -> jpointer {
    //((**env).SetIntField)(env, obj, field)
//}
//#[cfg(target_word_size = "64")]
//#[inline(always)]
//fn set_jpointer(env: *mut JNIEnv, obj: jobject, field: jfieldID) -> jpointer {
    //((**env).SetLongField)(env, obj, field)
//}

fn on_unwind(msg: &(Any + Send), file: &'static str, line: uint) {
    use core::fmt::FormatWriter;
    // as far as I know there's no way to identify traits that can be cast to Show at runtime
    if let Some(s) = msg.downcast_ref::<&Show>() {
        loge!("fatal error in {}:{} as &Show: {}", file, line, s);
    } else if let Some(s) = msg.downcast_ref::<Box<Show>>() {
        loge!("fatal error in {}:{} as Box<Show>: {}", file, line, &**s);
    } else if let Some(s) = msg.downcast_ref::<&str>() {
        loge!("fatal error in {}:{} as &str: {}", file, line, s);
    } else if let Some(s) = msg.downcast_ref::<String>() {
        loge!("fatal error in {}:{} as String: {}", file, line, s);
    } else if let Some(s) = msg.downcast_ref::<MaybeOwned<'static>>() {
        loge!("fatal error in {}:{} as MaybeOwned: {}", file, line, s);
    } else {
        loge!("fatal error in {}:{}: unknown error message type {}!", file, line, msg.get_type_id());
        loge!("Printing start:");
        unsafe {
            let mut line = Vec::new();
            // stolen from unwind.rs
            struct VecWriter<'a> { v: &'a mut Vec<u8> }
            impl<'a> ::core::fmt::FormatWriter for VecWriter<'a> {
                fn write(&mut self, buf: &[u8]) -> fmt::Result {
                    self.v.push_all(buf);
                    Ok(())
                }
            }

            let width = 32;
            let raw::TraitObject { data: msgptr, vtable: _ } = mem::transmute(msg);
            let msgslice: &[u8] = mem::transmute(raw::Slice { data: msgptr as *const (), len: 1000 });
            let poscounter = iter::count(msgptr as u32, width as u32);
            for (chunk, pos) in msgslice.chunks(width).zip(poscounter) {
                {
                    let mut writer = VecWriter { v: &mut line };
                    let _ = write!(&mut writer, "{:08x}: ", pos);
                    for byte in chunk.iter() {
                        let _ = write!(&mut writer, "{} ", byte);
                    }
                }
                loge!("{}", ::core::str::from_utf8_unchecked(line.as_slice().init()));
                line.clear();
            }
        }
    }
    // Unwinding always fails, but not before messing up android's crash report backtrace.
    // So, commit suicide before that can happen.
    unsafe {
        let null: *mut u16 = ptr::null_mut();
        *null = 0xdead;
    }
}

#[allow(non_snake_case, unused_variables)]
#[no_mangle]
pub unsafe extern "C" fn JNI_OnLoad(vm: *mut JavaVM, reserved: *mut c_void) -> jint {
    logi!("jni onload!!");
    let mut env: *mut c_void = ptr::null_mut();
    if ((**vm).GetEnv)(vm, (&mut env as *mut *mut c_void), JNI_VERSION_1_6) != JNI_OK {
        return -1;
    }
    let env = env as *mut JNIEnv;
    logi!("got environment!: {}", env);

    texturethread::init(env);
    texturethread::init(env);
    android_bitmap::init(env);
    gldataclasses::init(env);
    motionevent::init(env);

    GL_EXCEPTION = CaseClass::new(env, cstr!("com/github/wartman4404/gldraw/GLException"), cstr!("(Ljava/lang/String;)V"));

    //rustrt::init(1, ["rustjni".as_ptr()].as_ptr());
    std::rt::unwind::register(on_unwind);

    logi!("finished jni_onload");
    JNI_VERSION_1_2
}

#[allow(non_snake_case, unused_variables)]
#[no_mangle]
pub unsafe extern "C" fn JNI_OnUnload(vm: *mut JavaVM, reserved: *mut c_void) {
    logi!("jni onload!!");
    let mut env: *mut c_void = ptr::null_mut();
    if ((**vm).GetEnv)(vm, (&mut env as *mut *mut c_void), JNI_VERSION_1_6) != JNI_OK {
        return;
    }
    let env = env as *mut JNIEnv;
    texturethread::destroy(env);
    texturethread::destroy(env);
    android_bitmap::destroy(env);
    gldataclasses::destroy(env);
    motionevent::destroy(env);
    GL_EXCEPTION.destroy(env);
}
