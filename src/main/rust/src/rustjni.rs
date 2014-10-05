#![allow(unused_imports, unused_variable, dead_code)]
use core::prelude::*;
use core::{ptr, mem};
use core::ptr::RawMutPtr;
use core::raw;
use libc;
use libc::{c_void, c_char};
use collections::string::String;

use jni;
use jni::{jobject, jclass, jfieldID, jmethodID, JNIEnv, jint, jfloat, jstring, jfloatArray, JNINativeMethod, JavaVM, jboolean, jchar, jsize};
use android::input::AInputEvent;
use android::bitmap::{AndroidBitmap_getInfo, AndroidBitmap_lockPixels, AndroidBitmap_unlockPixels, AndroidBitmapInfo};
use android::bitmap::{ANDROID_BITMAP_FORMAT_RGBA_8888, ANDROID_BITMAP_FORMAT_A_8};
use android::native_window_jni::{ANativeWindow_fromSurface};//, ANativeWindow_release};
use android::native_window::ANativeWindow_release;

use log::logi;

use glstore::DrawObjectIndex;
use copyshader::CopyShader;
use pointshader::PointShader;
use glinit;
use glpoint;
use glpoint::{MotionEventConsumer, MotionEventProducer};
use motionevent;
use matrix::Matrix;
use eglinit;
use jni_constants::*;

type GLInit<'a> = *mut glinit::Data<'a>;

macro_rules! cstr(
    ($str:expr) => (
        concat!($str, "\0").as_ptr() as *const ::libc::c_char
    )
)
macro_rules! native_method(
    ($name:expr, $sig:expr, $fn_ptr:expr) => (
        JNINativeMethod {
            name: cstr!($name),
            signature: cstr!($sig),
            fnPtr: $fn_ptr as *mut libc::c_void,
        }
    )
)


static mut MOTION_CLASS: jclass = 0 as jclass;
static mut MOTIONEVENT_NATIVE_PTR_FIELD: jfieldID = 0 as jfieldID;

unsafe extern "C" fn init_gl(env: *mut JNIEnv, thiz: jobject, w: jint, h: jint) -> jint {
    glinit::setup_graphics(w, h) as i32
}

unsafe extern "C" fn finish_gl(env: *mut JNIEnv, thiz: jobject, data: jint) {
    glinit::deinit_gl(data as GLInit);
    logi!("finished deinit");
}

unsafe extern "C" fn native_draw_queued_points(env: *mut JNIEnv, thiz: jobject, data: i32, handler: i32, java_matrix: jfloatArray) {
    let mut matrix: Matrix = mem::uninitialized();
    ((**env).GetFloatArrayRegion)(env, java_matrix, 0, 16, matrix.as_mut_ptr());
    glinit::draw_queued_points(data as GLInit, handler as *mut MotionEventConsumer, matrix.as_mut_ptr());
}

unsafe extern "C" fn native_update_gl(env: *mut JNIEnv, thiz: jobject, data: i32) {
    glinit::render_frame(data as GLInit);
}

unsafe extern "C" fn init_motion_event_handler(env: *mut JNIEnv, thiz: jobject) -> jobject {
    let (consumer, producer) = glpoint::create_motion_event_handler();
    let pairclass = ((**env).FindClass)(env, cstr!("com/github/wartman4404/gldraw/MotionEventHandlerPair"));
    let constructor = ((**env).GetMethodID)(env, pairclass, cstr!("<init>"), cstr!("(II)V"));
    let newobj: extern "C" fn(*mut JNIEnv, jclass, jmethodID, ...) -> jobject = mem::transmute((**env).NewObject);
    newobj(env, pairclass, constructor, consumer as int, producer as int)
}

unsafe extern "C" fn destroy_motion_event_handler(env: *mut JNIEnv, thiz: jobject, pairobj: jobject) {
    let pairclass = ((**env).FindClass)(env, cstr!("com/github/wartman4404/gldraw/MotionEventHandlerPair"));
    let consumerfield = ((**env).GetFieldID)(env, pairclass, cstr!("consumer"), cstr!("I"));
    let producerfield = ((**env).GetFieldID)(env, pairclass, cstr!("producer"), cstr!("I"));
    let consumer = ((**env).GetIntField)(env, pairobj, consumerfield) as *mut MotionEventConsumer;
    let producer = ((**env).GetIntField)(env, pairobj, producerfield) as *mut MotionEventProducer;
    glpoint::destroy_motion_event_handler(consumer, producer);
}

unsafe extern "C" fn native_append_motion_event(env: *mut JNIEnv, thiz: jobject, handler: jint, evtobj: jobject) {
    let evtptr = ((**env).GetIntField)(env, evtobj, MOTIONEVENT_NATIVE_PTR_FIELD);
    glpoint::jni_append_motion_event(mem::transmute(handler), evtptr as *const AInputEvent);
}

unsafe extern "C" fn set_anim_shader(env: *mut JNIEnv, thiz: jobject, data: jint, shader: jint) {
    glinit::set_anim_shader(data as GLInit, mem::transmute(shader));
}

unsafe extern "C" fn set_copy_shader(env: *mut JNIEnv, thiz: jobject, data: jint, shader: jint) {
    glinit::set_copy_shader(data as GLInit, mem::transmute(shader));
}

unsafe extern "C" fn set_point_shader(env: *mut JNIEnv, thiz: jobject, data: jint, shader: jint) {
    glinit::set_point_shader(data as GLInit, mem::transmute(shader));
}

unsafe extern "C" fn set_brush_texture(env: *mut JNIEnv, thiz: jobject, data: jint, texture: jint) {
    glinit::set_brush_texture(data as GLInit, texture as i32);
}

unsafe extern "C" fn create_texture(env: *mut JNIEnv, thiz: jobject, data: jint, bitmap: jobject) -> jint {
    let bitmap = AndroidBitmap::from_jobject(env, bitmap);
    let texture = glinit::load_texture(data as GLInit, bitmap.info.width as i32, bitmap.info.height as i32, bitmap.pixels as *const u8, bitmap.info.format);
    texture
}

unsafe extern "C" fn clear_framebuffer(env: *mut JNIEnv, thiz: jobject, data: jint) {
    glinit::clear_buffer(data as GLInit);
}

unsafe fn shader_strs<T>(env: *mut JNIEnv, data: GLInit, vec: jstring, frag: jstring, callback: unsafe fn(GLInit, Option<String>, Option<String>) -> T) -> T {
    let vecstr = get_string(env, vec);
    let fragstr = get_string(env, frag);
    let ret = callback(data, vecstr, fragstr);
    ret
}

unsafe fn get_string(env: *mut JNIEnv, string: jstring) -> Option<String> {
    match string.as_mut() {
        Some(string) => {
            let getchars: extern "C" fn(*mut JNIEnv, jstring, *mut jboolean) -> *const jchar = mem::transmute((**env).GetStringChars);
            let chars = getchars(env, string, ptr::null_mut()).as_ref();
            match chars {
                Some(c) => {
                    let getlen: extern "C" fn(*mut JNIEnv, jstring) -> jsize = mem::transmute((**env).GetStringLength);
                    let strslice: &[u16] = mem::transmute(raw::Slice { data: c, len: getlen(env, string) as uint });
                    let ruststr = String::from_utf16(strslice);
                    let releasechars: extern "C" fn(*mut JNIEnv, jstring, *const jchar) = mem::transmute((**env).ReleaseStringChars);
                    releasechars(env, string as jstring, strslice.as_ptr());
                    ruststr
                },
                None => None,
            }
        },
        None => None,
    }
}

unsafe extern "C" fn compile_copyshader(env: *mut JNIEnv, thiz: jobject, data: i32, vec: jstring, frag: jstring) -> jint {
    mem::transmute(shader_strs(env, data as GLInit, vec, frag, glinit::compile_copy_shader))
}

unsafe extern "C" fn compile_pointshader(env: *mut JNIEnv, thiz: jobject, data: i32, vec: jstring, frag: jstring) -> jint {
    mem::transmute(shader_strs(env, data as GLInit, vec, frag, glinit::compile_point_shader))
}

unsafe extern "C" fn draw_image(env: *mut JNIEnv, thiz: jobject, data: i32, bitmap: jobject) {
    // TODO: ensure rgba_8888 format and throw error
    let bitmap = AndroidBitmap::from_jobject(env, bitmap);
    glinit::draw_image(data as GLInit, bitmap.info.width as i32, bitmap.info.height as i32, bitmap.pixels as *const u8);
}

unsafe extern "C" fn export_pixels(env: *mut JNIEnv, thiz: jobject, data: i32) -> jobject {
    glinit::with_pixels(data as GLInit, |w, h, pixels| {
        logi!("in callback!");
        let bitmapclass = ((**env).FindClass)(env, cstr!("android/graphics/Bitmap"));
        let bitmap = AndroidBitmap::new(env, w, h);
        let outpixels = bitmap.as_slice();
        ptr::copy_nonoverlapping_memory(outpixels.as_mut_ptr(), pixels as *const u8, outpixels.len());
        let bitmap = bitmap.obj;
        let premult = ((**env).GetMethodID)(env, bitmapclass, cstr!("setPremultiplied"), cstr!("(Z)V"));
        let voidmethod: extern "C" fn(*mut JNIEnv, jobject, jmethodID, ...) = mem::transmute((**env).CallVoidMethod);
        voidmethod(env, bitmap, premult, JNI_TRUE);
        logi!("done with callback");
        bitmap
    })
}

struct AndroidBitmap {
    env: *mut JNIEnv,
    obj: jobject,
    pixels: *mut u8,
    info: AndroidBitmapInfo,
}
impl AndroidBitmap {
    unsafe fn from_jobject(env: *mut JNIEnv, bitmap: jobject) -> AndroidBitmap {
        let mut pixels: *mut libc::c_void = ptr::null_mut();
        AndroidBitmap_lockPixels(env, bitmap, &mut pixels);
        logi!("locked pixels in {}", pixels);
        let mut result = AndroidBitmap { env: env, obj: bitmap, pixels: pixels as *mut u8, info: mem::zeroed() };
        AndroidBitmap_getInfo(env, bitmap, &mut result.info);
        result
    }

    unsafe fn new(env: *mut JNIEnv, w: i32, h: i32) -> AndroidBitmap {
        let bitmapclass = ((**env).FindClass)(env, cstr!("android/graphics/Bitmap"));
        let configclass = ((**env).FindClass)(env, cstr!("android/graphics/Bitmap$Config"));
        let argbfield = ((**env).GetStaticFieldID)(env, configclass, cstr!("ARGB_8888"), cstr!("Landroid/graphics/Bitmap$Config;"));
        let argb = ((**env).GetStaticObjectField)(env, configclass, argbfield);
        let createbitmap = ((**env).GetStaticMethodID)(env, bitmapclass, cstr!("createBitmap"), cstr!("(IILandroid/graphics/Bitmap$Config;)Landroid/graphics/Bitmap;"));
        let bitmap = ((**env).CallStaticObjectMethod)(env, bitmapclass, createbitmap, w, h, argb);
        logi!("created bitmap");
        AndroidBitmap::from_jobject(env, bitmap)
    }
    
    unsafe fn as_slice(&self) -> &mut [u8] {
        let pixelsize = match self.info.format as u32 {
            ANDROID_BITMAP_FORMAT_RGBA_8888 => 4,
            ANDROID_BITMAP_FORMAT_A_8 => 1,
            x => fail!("bitmap format {} not implemented!", x),
        };
        let pixelvec = raw::Slice { data: self.pixels as *const u8, len: (self.info.width * self.info.height * pixelsize) as uint };
        mem::transmute(pixelvec)
    }
}

impl Drop for AndroidBitmap {
    fn drop(&mut self) {
        unsafe {
            AndroidBitmap_unlockPixels(self.env, self.obj);
        }
        logi!("unlocked pixels");
    }
}

unsafe extern "C" fn jni_egl_finish(env: *mut JNIEnv, thiz: jobject) {
    eglinit::egl_finish();
}

unsafe extern "C" fn jni_egl_init(env: *mut JNIEnv, thiz: jobject, surface: jobject) {
    let window = ANativeWindow_fromSurface(env, surface);
    logi!("got ANAtiveWindow: 0x{:x}", window as u32);
    eglinit::egl_init(window as *mut libc::c_void);
    ANativeWindow_release(window);
}

unsafe extern "C" fn jni_lua_compile_script(env: *mut JNIEnv, thiz: jobject, data: i32, script: jstring) -> jint {
    let scriptstr = get_string(env, script);
    mem::transmute(glinit::compile_luascript(data as GLInit, scriptstr))
}

unsafe extern "C" fn jni_lua_set_interpolator(env: *mut JNIEnv, thiz: jobject, data: jint, scriptid: jint) {
    glinit::set_interpolator(data as GLInit, mem::transmute(scriptid));
}

unsafe extern "C" fn jni_add_layer(env: *mut JNIEnv, thiz: jobject, data: jint, copyshader: jint, pointshader: jint, pointidx: jint) {
    glinit::add_layer(data as GLInit, mem::transmute(copyshader), mem::transmute(pointshader), mem::transmute(pointidx));
}

unsafe extern "C" fn jni_clear_layers(env: *mut JNIEnv, thiz: jobject, data: jint) {
    glinit::clear_layers(data as GLInit);
}

unsafe fn register_classmethods(env: *mut JNIEnv, classname: *const i8, methods: &[JNINativeMethod]) {
    let class = ((**env).FindClass)(env, classname);
    ((**env).RegisterNatives)(env, class, methods.as_ptr(), methods.len() as i32);
}

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn JNI_OnLoad(vm: *mut JavaVM, reserved: *mut libc::c_void) -> jint {
    logi!("jni onload!!");
    let mut env: *mut libc::c_void = ptr::null_mut();
    if ((**vm).GetEnv)(vm, (&mut env as *mut *mut libc::c_void), JNI_VERSION_1_6) != JNI_OK {
        return -1;
    }
    let env = env as *mut JNIEnv;
    logi!("got environment!: {}", env);
    MOTION_CLASS = ((**env).FindClass)(env, cstr!("android/view/MotionEvent"));
    MOTIONEVENT_NATIVE_PTR_FIELD = ((**env).GetFieldID)(env, MOTION_CLASS, cstr!("mNativePtr"), cstr!("I"));
    logi!("got motion classes");

    let mainmethods = [
        native_method!("nativeAppendMotionEvent", "(ILandroid/view/MotionEvent;)V", native_append_motion_event),
    ];
    register_classmethods(env, cstr!("com/github/wartman4404/gldraw/MainActivity"), mainmethods);

    let texturemethods = [
        native_method!("nativeUpdateGL", "(I)V", native_update_gl),
        native_method!("nativeDrawQueuedPoints", "(II[F)V", native_draw_queued_points),
        native_method!("nativeClearFramebuffer", "(I)V", clear_framebuffer),
        native_method!("drawImage", "(ILandroid/graphics/Bitmap;)V", draw_image),
        native_method!("nativeSetAnimShader", "(II)Z", set_anim_shader),
        native_method!("nativeSetCopyShader", "(II)Z", set_copy_shader),
        native_method!("nativeSetPointShader", "(II)Z", set_point_shader),
        native_method!("nativeSetBrushTexture", "(II)V", set_brush_texture),
        native_method!("exportPixels", "(I)Landroid/graphics/Bitmap;", export_pixels),
        native_method!("nativeSetInterpolator", "(II)V", jni_lua_set_interpolator),
        native_method!("nativeAddLayer", "(IIII)V", jni_add_layer),
        native_method!("nativeClearLayers", "(I)V", jni_clear_layers),
    ];
    register_classmethods(env, cstr!("com/github/wartman4404/gldraw/TextureSurfaceThread"), texturemethods);
    logi!("registered texture thread methods!");

    let pointshaderstaticmethods = [
        native_method!("compile", "(ILjava/lang/String;Ljava/lang/String;)I", compile_pointshader),
    ];
    let copyshaderstaticmethods = [
        native_method!("compile", "(ILjava/lang/String;Ljava/lang/String;)I", compile_copyshader),
    ];
    let texturestaticmethods = [
        native_method!("init", "(ILandroid/graphics/Bitmap;)I", create_texture),
    ];
    register_classmethods(env, cstr!("com/github/wartman4404/gldraw/PointShader$"), pointshaderstaticmethods);
    register_classmethods(env, cstr!("com/github/wartman4404/gldraw/CopyShader$"), copyshaderstaticmethods);
    register_classmethods(env, cstr!("com/github/wartman4404/gldraw/Texture$"), texturestaticmethods);
    logi!("registered point|copy|texture static methods!");

    let eglhelpermethods = [
        native_method!("nativeFinish", "()V", jni_egl_finish),
        native_method!("nativeInit", "(Landroid/view/Surface;)V", jni_egl_init),
    ];
    register_classmethods(env, cstr!("com/github/wartman4404/gldraw/EGLHelper"), eglhelpermethods);
    logi!("registered egl methods!");

    let luastaticmethods = [
        native_method!("init", "(ILjava/lang/String;)I", jni_lua_compile_script),
    ];
    register_classmethods(env, cstr!("com/github/wartman4404/gldraw/LuaScript$"), luastaticmethods);
    logi!("registered lua methods!");

    let glinitstaticmethods = [
        native_method!("initGL", "(II)I", init_gl),
        native_method!("destroy", "(I)V", finish_gl),
    ];
    register_classmethods(env, cstr!("com/github/wartman4404/gldraw/GLInit$"), glinitstaticmethods);

    let motioneventhandlerstaticmethods = [
        native_method!("init", "()Lcom/github/wartman4404/gldraw/MotionEventHandlerPair;", init_motion_event_handler),
        native_method!("destroy", "(Lcom/github/wartman4404/gldraw/MotionEventHandlerPair;)V", destroy_motion_event_handler),
    ];
    register_classmethods(env, cstr!("com/github/wartman4404/gldraw/MotionEventHandlerPair$"), motioneventhandlerstaticmethods);
    logi!("registered motionevent methods!");
    logi!("finished jni_onload");
    JNI_VERSION_1_2
}
