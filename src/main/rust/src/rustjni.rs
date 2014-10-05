#![allow(unused_imports, unused_variable, dead_code)]
use core::prelude::*;
use core::{ptr, mem};
use core::ptr::RawMutPtr;
use libc;

use jni;
use jni::{jobject, jclass, jfieldID, jmethodID, JNIEnv, jint, jstring, jfloatArray, JNINativeMethod, JavaVM};
use android::input::AInputEvent;
use android::bitmap::{AndroidBitmap_getInfo, AndroidBitmap_lockPixels, AndroidBitmap_unlockPixels, AndroidBitmapInfo};
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
    ((**env).GetFloatArrayRegion).unwrap()(env, java_matrix, 0, 16, matrix.as_mut_ptr());
    glinit::draw_queued_points(data as GLInit, handler as *mut MotionEventConsumer, matrix.as_mut_ptr());
}

unsafe extern "C" fn native_update_gl(env: *mut JNIEnv, thiz: jobject, data: i32) {
    glinit::render_frame(data as GLInit);
}

unsafe extern "C" fn init_motion_event_handler(env: *mut JNIEnv, thiz: jobject) -> jobject {
    let (consumer, producer) = glpoint::create_motion_event_handler();
    let pairclass = ((**env).FindClass).unwrap()(env, cstr!("com/github/wartman4404/gldraw/MotionEventHandlerPair"));
    let constructor = ((**env).GetMethodID).unwrap()(env, pairclass, cstr!("<init>"), cstr!("(II)V"));
    ((**env).NewObject).unwrap()(env, pairclass, constructor, consumer as int, producer as int)
}

unsafe extern "C" fn destroy_motion_event_handler(env: *mut JNIEnv, thiz: jobject, pairobj: jobject) {
    let pairclass = ((**env).FindClass).unwrap()(env, cstr!("com/github/wartman4404/gldraw/MotionEventHandlerPair"));
    let consumerfield = ((**env).GetFieldID).unwrap()(env, pairclass, cstr!("consumer"), cstr!("I"));
    let producerfield = ((**env).GetFieldID).unwrap()(env, pairclass, cstr!("producer"), cstr!("I"));
    let consumer = ((**env).GetIntField).unwrap()(env, pairobj, consumerfield) as *mut MotionEventConsumer;
    let producer = ((**env).GetIntField).unwrap()(env, pairobj, producerfield) as *mut MotionEventProducer;
    glpoint::destroy_motion_event_handler(consumer, producer);
}

unsafe extern "C" fn native_append_motion_event(env: *mut JNIEnv, thiz: jobject, handler: jint, evtobj: jobject) {
    let evtptr = ((**env).GetIntField).unwrap()(env, evtobj, MOTIONEVENT_NATIVE_PTR_FIELD);
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
    let mut info: AndroidBitmapInfo = mem::uninitialized();
    AndroidBitmap_getInfo(env, bitmap, &mut info);
    let mut pixels: *mut libc::c_void = ptr::null_mut();
    AndroidBitmap_lockPixels(env, bitmap, &mut pixels);
    let texture = glinit::load_texture(data as GLInit, info.width as i32, info.height as i32, pixels as *const u8, info.format);
    AndroidBitmap_unlockPixels(env, bitmap);
    texture
}

unsafe extern "C" fn clear_framebuffer(env: *mut JNIEnv, thiz: jobject, data: jint) {
    glinit::clear_buffer(data as GLInit);
}

unsafe fn shader_strs<T>(env: *mut JNIEnv, data: GLInit, vec: jstring, frag: jstring, callback: unsafe fn(GLInit, *const i8, *const i8) -> T) -> T {
    let vecstr  = vec .as_mut().and_then(|v| ((**env).GetStringUTFChars).unwrap()(env, v, ptr::null_mut()).as_ref().map(|x| x as *const i8));
    let fragstr = frag.as_mut().and_then(|f| ((**env).GetStringUTFChars).unwrap()(env, f, ptr::null_mut()).as_ref().map(|x| x as *const i8));
    let ret = callback(data, vecstr.unwrap_or(ptr::null()), fragstr.unwrap_or(ptr::null()));
    for v in vecstr.iter()  { ((**env).ReleaseStringUTFChars).unwrap()(env, vec, *v); }
    for f in fragstr.iter() { ((**env).ReleaseStringUTFChars).unwrap()(env, frag, *f); }
    ret
}

unsafe extern "C" fn compile_copyshader(env: *mut JNIEnv, thiz: jobject, data: i32, vec: jstring, frag: jstring) -> jint {
    mem::transmute(shader_strs(env, data as GLInit, vec, frag, glinit::compile_copy_shader))
}

unsafe extern "C" fn compile_pointshader(env: *mut JNIEnv, thiz: jobject, data: i32, vec: jstring, frag: jstring) -> jint {
    mem::transmute(shader_strs(env, data as GLInit, vec, frag, glinit::compile_point_shader))
}

unsafe extern "C" fn draw_image(env: *mut JNIEnv, thiz: jobject, data: i32, bitmap: jobject) {
    // TODO: ensure rgba_8888 format and throw error
    let mut info: AndroidBitmapInfo = mem::uninitialized();
    AndroidBitmap_getInfo(env, bitmap, &mut info);
    let mut pixels: *mut libc::c_void = ptr::null_mut();
    AndroidBitmap_lockPixels(env, bitmap, &mut pixels);
    glinit::draw_image(data as GLInit, info.width as i32, info.height as i32, pixels as *const u8);
    AndroidBitmap_unlockPixels(env, bitmap);
}

unsafe extern "C" fn export_pixels(env: *mut JNIEnv, thiz: jobject, data: i32) -> jobject {
    glinit::with_pixels(data as GLInit, |w, h, pixels| {
        logi!("in callback!");
        let bitmapclass = ((**env).FindClass).unwrap()(env, cstr!("android/graphics/Bitmap"));
        let configclass = ((**env).FindClass).unwrap()(env, cstr!("android/graphics/Bitmap$Config"));
        let argbfield = ((**env).GetStaticFieldID).unwrap()(env, configclass, cstr!("ARGB_8888"), cstr!("Landroid/graphics/Bitmap$Config;"));
        let argb = ((**env).GetStaticObjectField).unwrap()(env, configclass, argbfield);
        let createbitmap = ((**env).GetStaticMethodID).unwrap()(env, bitmapclass, cstr!("createBitmap"), cstr!("(IILandroid/graphics/Bitmap$Config;)Landroid/graphics/Bitmap;"));
        let bitmap = ((**env).CallStaticObjectMethod).unwrap()(env, bitmapclass, createbitmap, w, h, argb);
        logi!("created bitmap");
        let mut outpixels: *mut libc::c_void = ptr::null_mut();
        AndroidBitmap_lockPixels(env, bitmap, &mut outpixels);
        logi!("locked pixels");
        ptr::copy_nonoverlapping_memory(outpixels, pixels as *const libc::c_void, (w*h*4) as uint);
        logi!("copied pixels");
        AndroidBitmap_unlockPixels(env, bitmap);
        logi!("unlocked pixels");
        let premult = ((**env).GetMethodID).unwrap()(env, bitmapclass, cstr!("setPremultiplied"), cstr!("(Z)V"));
        ((**env).CallVoidMethod).unwrap()(env, bitmap, premult, ::jni_constants::JNI_TRUE);
        logi!("done with callback");
        bitmap
    })
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
    let scriptchars = script.as_mut().and_then(|s| ((**env).GetStringUTFChars).unwrap()(env, s, ptr::null_mut()).as_ref().map(|x| x as *const i8));
    let scriptid = glinit::compile_luascript(data as GLInit, scriptchars.unwrap_or(ptr::null()));
    for s in scriptchars.iter()  { ((**env).ReleaseStringUTFChars).unwrap()(env, script, *s); }
    mem::transmute(scriptid)
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

/// because this gets compiled into a static library first, we can't directly implement JNI_OnLoad
#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn JNI_OnLoad(vm: *mut JavaVM, reserved: *mut libc::c_void) -> jint {
    logi!("jni onload!!");
    let mut env: *mut libc::c_void = ptr::null_mut();
    if ((**vm).GetEnv).unwrap()(vm, (&mut env as *mut *mut libc::c_void), JNI_VERSION_1_6) != JNI_OK {
        return -1;
    }
    let env = env as *mut JNIEnv;
    logi!("got environment!: {}", env);
    MOTION_CLASS = ((**env).FindClass).unwrap()(env, cstr!("android/view/MotionEvent"));
    MOTIONEVENT_NATIVE_PTR_FIELD = ((**env).GetFieldID).unwrap()(env, MOTION_CLASS, cstr!("mNativePtr"), cstr!("I"));
    logi!("got motion classes");

    let mainmethods = [
        JNINativeMethod { 
            name: cstr!("nativeAppendMotionEvent"),
            signature: cstr!("(ILandroid/view/MotionEvent;)V"),
            fnPtr: native_append_motion_event as *mut libc::c_void,
        }
    ];

    let mainactivityclass = ((**env).FindClass).unwrap()(env, cstr!("com/github/wartman4404/gldraw/MainActivity"));
    ((**env).RegisterNatives).unwrap()(env, mainactivityclass, mainmethods.as_ptr(), mainmethods.len() as i32);

    let texturemethods = [
        JNINativeMethod {
            name: cstr!("nativeUpdateGL"),
            signature: cstr!("(I)V"),
            fnPtr: native_update_gl as *mut libc::c_void as *mut libc::c_void,
        }, JNINativeMethod {
            name: cstr!("nativeDrawQueuedPoints"),
            signature: cstr!("(II[F)V"),
            fnPtr: native_draw_queued_points as *mut libc::c_void as *mut libc::c_void,
        }, JNINativeMethod {
            name: cstr!("nativeClearFramebuffer"),
            signature: cstr!("(I)V"),
            fnPtr: clear_framebuffer as *mut libc::c_void,
        }, JNINativeMethod {
            name: cstr!("drawImage"),
            signature: cstr!("(ILandroid/graphics/Bitmap;)V"),
            fnPtr: draw_image as *mut libc::c_void,
        }, JNINativeMethod {
            name: cstr!("nativeSetAnimShader"),
            signature: cstr!("(II)Z"),
            fnPtr: set_anim_shader as *mut libc::c_void,
        }, JNINativeMethod {
            name: cstr!("nativeSetCopyShader"),
            signature: cstr!("(II)Z"),
            fnPtr: set_copy_shader as *mut libc::c_void,
        }, JNINativeMethod {
            name: cstr!("nativeSetPointShader"),
            signature: cstr!("(II)Z"),
            fnPtr: set_point_shader as *mut libc::c_void,
        }, JNINativeMethod {
            name: cstr!("nativeSetBrushTexture"),
            signature: cstr!("(II)V"),
            fnPtr: set_brush_texture as *mut libc::c_void,
        }, JNINativeMethod {
            name: cstr!("exportPixels"),
            signature: cstr!("(I)Landroid/graphics/Bitmap;"),
            fnPtr: export_pixels as *mut libc::c_void,
        }, JNINativeMethod {
            name: cstr!("nativeSetInterpolator"),
            signature: cstr!("(II)V"),
            fnPtr: jni_lua_set_interpolator as *mut libc::c_void,
        }, JNINativeMethod {
            name: cstr!("nativeAddLayer"),
            signature: cstr!("(IIII)V"),
            fnPtr: jni_add_layer as *mut libc::c_void,
        }, JNINativeMethod {
            name: cstr!("nativeClearLayers"),
            signature: cstr!("(I)V"),
            fnPtr: jni_clear_layers as *mut libc::c_void,
        },
    ];
    let textureclass = ((**env).FindClass).unwrap()(env, cstr!("com/github/wartman4404/gldraw/TextureSurfaceThread"));
    ((**env).RegisterNatives).unwrap()(env, textureclass, texturemethods.as_ptr(), texturemethods.len() as i32);
    logi!("registered texture methods!");

    let pointshaderstaticmethods = [
        JNINativeMethod {
            name: cstr!("compile"),
            signature: cstr!("(ILjava/lang/String;Ljava/lang/String;)I"),
            fnPtr: compile_pointshader as *mut libc::c_void,
        },
    ];
    let copyshaderstaticmethods = [
        JNINativeMethod {
            name: cstr!("compile"),
            signature: cstr!("(ILjava/lang/String;Ljava/lang/String;)I"),
            fnPtr: compile_copyshader as *mut libc::c_void,
        },
    ];
    let texturestaticmethods = [
        JNINativeMethod {
            name: cstr!("init"),
            signature: cstr!("(ILandroid/graphics/Bitmap;)I"),
            fnPtr: create_texture as *mut libc::c_void,
        },
    ];

    let copyshaderstatic = ((**env).FindClass).unwrap()(env, cstr!("com/github/wartman4404/gldraw/CopyShader$"));
    let pointshaderstatic = ((**env).FindClass).unwrap()(env, cstr!("com/github/wartman4404/gldraw/PointShader$"));
    let texturestatic = ((**env).FindClass).unwrap()(env, cstr!("com/github/wartman4404/gldraw/Texture$"));
    ((**env).RegisterNatives).unwrap()(env, copyshaderstatic, copyshaderstaticmethods.as_ptr(), copyshaderstaticmethods.len() as i32);
    ((**env).RegisterNatives).unwrap()(env, pointshaderstatic, pointshaderstaticmethods.as_ptr(), pointshaderstaticmethods.len() as i32);
    ((**env).RegisterNatives).unwrap()(env, texturestatic, texturestaticmethods.as_ptr(), texturestaticmethods.len() as i32);
    logi!("registered point|copy|texture static methods!");

    let eglhelpermethods = [
        JNINativeMethod {
            name: cstr!("nativeFinish"),
            signature: cstr!("()V"),
            fnPtr: jni_egl_finish as *mut libc::c_void,
        }, JNINativeMethod {
            name: cstr!("nativeInit"),
            signature: cstr!("(Landroid/view/Surface;)V"),
            fnPtr: jni_egl_init as *mut libc::c_void,
        }
    ];
    let eglhelper = ((**env).FindClass).unwrap()(env, cstr!("com/github/wartman4404/gldraw/EGLHelper"));
    ((**env).RegisterNatives).unwrap()(env, eglhelper, eglhelpermethods.as_ptr(), eglhelpermethods.len() as i32);
    logi!("registered egl methods!");

    let luastaticmethods = [
        JNINativeMethod {
            name: cstr!("init"),
            signature: cstr!("(ILjava/lang/String;)I"),
            fnPtr: jni_lua_compile_script as *mut libc::c_void,
        }
    ];
    let luastatic = ((**env).FindClass).unwrap()(env, cstr!("com/github/wartman4404/gldraw/LuaScript$"));
    ((**env).RegisterNatives).unwrap()(env, luastatic, luastaticmethods.as_ptr(), luastaticmethods.len() as i32);
    logi!("registered lua methods!");

    let glinitstaticmethods = [
        JNINativeMethod {
            name: cstr!("initGL"),
            signature: cstr!("(II)I"),
            fnPtr: init_gl as *mut libc::c_void,
        }, JNINativeMethod {
            name: cstr!("destroy"),
            signature: cstr!("(I)V"),
            fnPtr: finish_gl as *mut libc::c_void,
        }
    ];
    let glinitstatic = ((**env).FindClass).unwrap()(env, cstr!("com/github/wartman4404/gldraw/GLInit$"));
    ((**env).RegisterNatives).unwrap()(env, glinitstatic, glinitstaticmethods.as_ptr(), glinitstaticmethods.len() as i32);

    let motioneventhandlerstaticmethods = [
        JNINativeMethod {
            name: cstr!("init"),
            signature: cstr!("()Lcom/github/wartman4404/gldraw/MotionEventHandlerPair;"),
            fnPtr: init_motion_event_handler as *mut libc::c_void,
        }, JNINativeMethod {
            name: cstr!("destroy"),
            signature: cstr!("(Lcom/github/wartman4404/gldraw/MotionEventHandlerPair;)V"),
            fnPtr: destroy_motion_event_handler as *mut libc::c_void,
        }
    ];
    let motioneventhandlerpairstatic = ((**env).FindClass).unwrap()(env, cstr!("com/github/wartman4404/gldraw/MotionEventHandlerPair$"));
    ((**env).RegisterNatives).unwrap()(env, motioneventhandlerpairstatic, motioneventhandlerstaticmethods.as_ptr(), motioneventhandlerstaticmethods.len() as i32);
    logi!("registered motionevent methods!");
    logi!("finished jni_onload");
    JNI_VERSION_1_2
}








