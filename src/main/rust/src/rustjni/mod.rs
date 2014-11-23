use core::prelude::*;
use core::{ptr, mem, raw};
use core::ptr::RawMutPtr;
use libc::{c_void, c_char};
use collections::str::IntoMaybeOwned;
use collections::string::String;
use collections::vec::Vec;

use jni::{jobject, jclass, jmethodID, JNIEnv, jint, jstring, jvalue, JNINativeMethod, JavaVM};
use jni_constants::*;

use log::logi;

use glinit::GLInit;
use drawevent::Events;
use glcommon::MString;


mod macros;
pub mod texturethread;
pub mod android_bitmap;
pub mod gldataclasses;
pub mod motionevent;

struct CaseClass {
    constructor: jmethodID,
    class: jclass,
}

struct GLInitEvents<'a> {
    glinit: GLInit<'a>,
    events: Events<'a>,
    jni_undo_callback: JNIUndoCallback,
}

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
    pub unsafe fn destroy(self, env: *mut JNIEnv) {
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
        Some(s) => Some(s.into_maybe_owned()),
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

fn get_safe_data<'a>(data: i32) -> &'a mut GLInitEvents<'a> {
    unsafe { mem::transmute(data) }
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

    texturethread::init(env);
    texturethread::init(env);
    android_bitmap::init(env);
    gldataclasses::init(env);
    motionevent::init(env);

    logi!("got environment!: {}", env);
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
}
