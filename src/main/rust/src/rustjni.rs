#![allow(unused_variable)]
use core::prelude::*;
use core::{ptr, mem};
use core::ptr::RawMutPtr;
use core::raw;
use alloc::boxed::Box;
use libc::{c_void, c_char};
use collections::string::String;
use collections::vec::Vec;

use jni::{jobject, jclass, jfieldID, jmethodID, JNIEnv, jint, jstring, jboolean, jfloatArray, JNINativeMethod, JavaVM};
use android::input::AInputEvent;
use android::bitmap::{AndroidBitmap_getInfo, AndroidBitmap_lockPixels, AndroidBitmap_unlockPixels, AndroidBitmapInfo};
use android::bitmap::{ANDROID_BITMAP_FORMAT_RGBA_8888, ANDROID_BITMAP_FORMAT_A_8};
use android::native_window_jni::{ANativeWindow_fromSurface};//, ANativeWindow_release};
use android::native_window::ANativeWindow_release;
use glcommon::GLResult;

use log::logi;

use glstore::DrawObjectIndex;
use glinit::{GLInit, AndroidBitmapFormat};
use glpoint;
use matrix::Matrix;
use eglinit;
use jni_constants::*;
use drawevent::Events;
use drawevent::event_stream::EventStream;
use gltexture::ToPixelFormat;
use gltexture::Texture;

macro_rules! native_method(
    ($name:expr, $sig:expr, $fn_ptr:expr) => (
        JNINativeMethod {
            name: cstr!($name),
            signature: cstr!($sig),
            fnPtr: $fn_ptr as *mut c_void,
        }
    )
)

static mut MOTION_CLASS: jclass = 0 as jclass;
static mut MOTIONEVENT_NATIVE_PTR_FIELD: jfieldID = 0 as jfieldID;

struct CaseClass {
    constructor: jmethodID,
    class: jclass,
}

struct GLInitEvents<'a> {
    glinit: GLInit<'a>,
    events: Events<'a>,
}

impl CaseClass {
    pub unsafe fn new(env: *mut JNIEnv, name: *const c_char, sig: *const c_char) -> CaseClass {
        let class = ((**env).FindClass)(env, name);
        let constructor = ((**env).GetMethodID)(env, class, cstr!("<init>"), sig);
        let globalclass = ((**env).NewGlobalRef)(env, class);

        CaseClass { constructor: constructor, class: globalclass }
    }
    pub unsafe fn construct<T>(&self, env: *mut JNIEnv, arg: T) -> jobject {
        ((**env).NewObject)(env, self.class, self.constructor, arg)
    }
    pub unsafe fn destroy(self, env: *mut JNIEnv) {
        ((**env).DeleteGlobalRef)(env, self.class);
    }
}

static mut SCALA_LEFT: CaseClass = CaseClass { constructor: 0 as jmethodID, class: 0 as jclass };
static mut SCALA_RIGHT: CaseClass = CaseClass { constructor: 0 as jmethodID, class: 0 as jclass };
static mut BOXED_JINT: CaseClass = CaseClass { constructor: 0 as jmethodID, class: 0 as jclass };

unsafe fn glresult_to_either<T>(env: *mut JNIEnv, result: GLResult<DrawObjectIndex<T>>) -> jobject {
    logi!("in glresult_to_either");
    match result {
        Err(msg) => {
            logi!("creating scala.util.Left for message: \"{}\"", msg);
            let jmsg = str_to_jstring(env, msg.as_slice());
            SCALA_LEFT.construct(env, jmsg)
        },
        Ok(idx) => {
            let idx: jint = mem::transmute(idx);
            logi!("creating scala.util.Right for drawobjectindex {}", idx);
            let boxedidx = BOXED_JINT.construct(env, idx);
            let result = SCALA_RIGHT.construct(env, boxedidx);
            result
        }
    }
}

unsafe fn str_to_jstring(env: *mut JNIEnv, s: &str) -> jstring {
    let u16msg: Vec<u16> = s.utf16_units().collect();
    ((**env).NewString)(env, u16msg.as_ptr(), u16msg.len() as i32)
}

fn get_safe_data<'a>(data: i32) -> &'a mut GLInitEvents<'a> {
    unsafe { mem::transmute(data) }
}

unsafe extern "C" fn init_gl(env: *mut JNIEnv, thiz: jobject, w: jint, h: jint) -> jint {
    mem::transmute(box GLInitEvents {
        glinit: GLInit::setup_graphics(w, h),
        events: Events::new(),
    })
}

unsafe extern "C" fn finish_gl(env: *mut JNIEnv, thiz: jobject, data: jint) {
    let data: Box<GLInitEvents> = mem::transmute(data);
    data.glinit.destroy();
    logi!("finished deinit");
}

unsafe extern "C" fn native_draw_queued_points(env: *mut JNIEnv, thiz: jobject, data: i32, handler: i32, java_matrix: jfloatArray) {
    let data = get_safe_data(data);
    let mut matrix: Matrix = mem::uninitialized();
    ((**env).GetFloatArrayRegion)(env, java_matrix, 0, 16, matrix.as_mut_ptr());
    if let Err(msg) = data.glinit.draw_queued_points(mem::transmute(handler), &mut data.events, &matrix) {
        let luaerr_class = ((**env).FindClass)(env, cstr!("com/github/wartman4404/gldraw/LuaException"));
        let luaerr_init = ((**env).GetMethodID)(env, luaerr_class, cstr!("<init>"), cstr!("(Ljava/lang/String;)V"));
        let err = ((**env).NewObject)(env, luaerr_class, luaerr_init, str_to_jstring(env, msg.as_slice()));
        ((**env).Throw)(env, err);
    }
}

unsafe extern "C" fn native_update_gl(env: *mut JNIEnv, thiz: jobject, data: i32) {
    let data = get_safe_data(data);
    data.glinit.render_frame();
    data.events.pushframe(); // FIXME make sure a frame was actually drawn! No java exceptions, missing copy shader, etc
}

unsafe extern "C" fn init_motion_event_handler(env: *mut JNIEnv, thiz: jobject) -> jobject {
    let (consumer, producer) = glpoint::create_motion_event_handler();
    let (consumer, producer) = (box consumer, box producer);
    let pairclass = ((**env).FindClass)(env, cstr!("com/github/wartman4404/gldraw/MotionEventHandlerPair"));
    let constructor = ((**env).GetMethodID)(env, pairclass, cstr!("<init>"), cstr!("(II)V"));
    let (consumer, producer): (i32, i32) = (mem::transmute(consumer), mem::transmute(producer));
    ((**env).NewObject)(env, pairclass, constructor, consumer, producer)
}

unsafe extern "C" fn destroy_motion_event_handler(env: *mut JNIEnv, thiz: jobject, pairobj: jobject) {
    let pairclass = ((**env).FindClass)(env, cstr!("com/github/wartman4404/gldraw/MotionEventHandlerPair"));
    let consumerfield = ((**env).GetFieldID)(env, pairclass, cstr!("consumer"), cstr!("I"));
    let producerfield = ((**env).GetFieldID)(env, pairclass, cstr!("producer"), cstr!("I"));
    let consumer = ((**env).GetIntField)(env, pairobj, consumerfield);
    let producer = ((**env).GetIntField)(env, pairobj, producerfield);
    glpoint::destroy_motion_event_handler(mem::transmute(consumer), mem::transmute(producer));
}

unsafe extern "C" fn native_append_motion_event(env: *mut JNIEnv, thiz: jobject, handler: jint, evtobj: jobject) {
    let evtptr = ((**env).GetIntField)(env, evtobj, MOTIONEVENT_NATIVE_PTR_FIELD);
    glpoint::jni_append_motion_event(mem::transmute(handler), evtptr as *const AInputEvent);
}

unsafe extern "C" fn set_anim_shader(env: *mut JNIEnv, thiz: jobject, data: jint, shader: jint) {
    let data = get_safe_data(data);
    let shader = data.events.use_animshader(mem::transmute(shader));
    data.glinit.set_anim_shader(shader);
}

unsafe extern "C" fn set_copy_shader(env: *mut JNIEnv, thiz: jobject, data: jint, shader: jint) {
    let data = get_safe_data(data);
    let shader = data.events.use_copyshader(mem::transmute(shader));
    data.glinit.set_copy_shader(shader);
}

unsafe extern "C" fn set_point_shader(env: *mut JNIEnv, thiz: jobject, data: jint, shader: jint) {
    let data = get_safe_data(data);
    let shader = data.events.use_pointshader(mem::transmute(shader));
    data.glinit.set_point_shader(shader);
}

unsafe extern "C" fn set_brush_texture(env: *mut JNIEnv, thiz: jobject, data: jint, texture: jint) {
    let data = get_safe_data(data);
    let brush = data.events.use_brush(mem::transmute(texture));
    data.glinit.set_brush_texture(brush);
}

unsafe fn safe_create_texture(env: *mut JNIEnv, data: jint, bitmap: jobject) -> GLResult<DrawObjectIndex<Texture>> {
    let bitmap = AndroidBitmap::from_jobject(env, bitmap);
    let (w, h) = (bitmap.info.width, bitmap.info.height);
    let format: AndroidBitmapFormat = mem::transmute(bitmap.info.format);
    let texformat = try!(format.to_pixelformat());
    Ok(get_safe_data(data).events.load_brush(w as i32, h as i32, bitmap.as_slice(), texformat))
}

unsafe extern "C" fn create_texture(env: *mut JNIEnv, thiz: jobject, data: jint, bitmap: jobject) -> jobject {
    glresult_to_either(env, safe_create_texture(env, data, bitmap))
}

unsafe extern "C" fn clear_framebuffer(env: *mut JNIEnv, thiz: jobject, data: jint) {
    let data = get_safe_data(data);
    data.events.clear();
    data.glinit.clear_buffer();
}

unsafe fn get_string(env: *mut JNIEnv, string: jstring) -> Option<String> {
    match string.as_mut() {
        Some(string) => {
            let chars = ((**env).GetStringChars)(env, string, ptr::null_mut()).as_ref();
            match chars {
                Some(c) => {
                    let len = ((**env).GetStringLength)(env, string);
                    let strslice: &[u16] = mem::transmute(raw::Slice { data: c, len: len as uint });
                    let ruststr = String::from_utf16(strslice);
                    ((**env).ReleaseStringChars)(env, string as jstring, strslice.as_ptr());
                    ruststr
                },
                None => None,
            }
        },
        None => None,
    }
}

unsafe extern "C" fn compile_copyshader(env: *mut JNIEnv, thiz: jobject, data: i32, vec: jstring, frag: jstring) -> jobject {
    glresult_to_either(env, get_safe_data(data).events.load_copyshader(get_string(env, vec), get_string(env, frag)))
}

unsafe extern "C" fn compile_pointshader(env: *mut JNIEnv, thiz: jobject, data: i32, vec: jstring, frag: jstring) -> jobject {
    glresult_to_either(env, get_safe_data(data).events.load_pointshader(get_string(env, vec), get_string(env, frag)))
}

unsafe extern "C" fn draw_image(env: *mut JNIEnv, thiz: jobject, data: i32, bitmap: jobject) {
    // TODO: ensure rgba_8888 format and throw error
    let bitmap = AndroidBitmap::from_jobject(env, bitmap);
    let pixels = bitmap.as_slice();
    get_safe_data(data).glinit.draw_image(bitmap.info.width as i32, bitmap.info.height as i32, pixels);
}

unsafe extern "C" fn export_pixels(env: *mut JNIEnv, thiz: jobject, data: i32) -> jobject {
    get_safe_data(data).glinit.with_pixels(|w, h, pixels| {
        logi!("in callback!");
        let bitmapclass = ((**env).FindClass)(env, cstr!("android/graphics/Bitmap"));
        let bitmap = AndroidBitmap::new(env, w, h);
        let outpixels = bitmap.as_slice();
        ptr::copy_nonoverlapping_memory(outpixels.as_mut_ptr(), pixels.as_ptr(), outpixels.len());
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
        let mut pixels: *mut c_void = ptr::null_mut();
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
    eglinit::egl_init(window as *mut c_void);
    ANativeWindow_release(window);
}

unsafe extern "C" fn jni_lua_compile_script(env: *mut JNIEnv, thiz: jobject, data: i32, script: jstring) -> jobject {
    let scriptstr = get_string(env, script);
    glresult_to_either(env, get_safe_data(data).events.load_interpolator(scriptstr))
}

unsafe extern "C" fn jni_lua_set_interpolator(env: *mut JNIEnv, thiz: jobject, data: jint, scriptid: jint) {
    let data = get_safe_data(data);
    let script = data.events.use_interpolator(mem::transmute(scriptid));
    data.glinit.set_interpolator(script);
}

unsafe extern "C" fn jni_add_layer(env: *mut JNIEnv, thiz: jobject, data: jint, copyshader: jint, pointshader: jint, pointidx: jint) {
    let data = get_safe_data(data);
    let copyshader = Some(mem::transmute(copyshader));
    let pointshader = Some(mem::transmute(pointshader));
    let layer = data.events.add_layer(data.glinit.dimensions, copyshader, pointshader, mem::transmute(pointidx));
    data.glinit.add_layer(layer);
}

unsafe extern "C" fn jni_clear_layers(env: *mut JNIEnv, thiz: jobject, data: jint) {
    let data = get_safe_data(data);
    data.events.clear_layers();
    data.glinit.clear_layers();
}

unsafe extern "C" fn jni_replay_begin(env: *mut JNIEnv, thiz: jobject, data: jint) -> jint {
    let data = get_safe_data(data);
    data.glinit.clear_layers();
    data.glinit.clear_buffer();
    mem::transmute(box EventStream::new())
}

#[allow(unused)]
unsafe extern "C" fn jni_replay_advance_frame(env: *mut JNIEnv, thiz: jobject, data: jint, replay: jint, java_matrix: jfloatArray) -> jboolean {
    let data = get_safe_data(data);
    let replay: &mut EventStream = mem::transmute(replay);
    let mut matrix: Matrix = mem::uninitialized();
    ((**env).GetFloatArrayRegion)(env, java_matrix, 0, 16, matrix.as_mut_ptr());
    let done = replay.advance_frame(&mut data.glinit, &mut data.events);
    data.glinit.draw_queued_points(&mut replay.consumer, &mut data.events, &matrix);
    if done { JNI_TRUE as jboolean } else { JNI_FALSE as jboolean }
}

unsafe extern "C" fn jni_replay_destroy(env: *mut JNIEnv, thiz: jobject, replay: jint) {
    let replay: Box<EventStream> = mem::transmute(replay);
    mem::drop(replay);
}

unsafe fn register_classmethods(env: *mut JNIEnv, classname: *const i8, methods: &[JNINativeMethod]) {
    let class = ((**env).FindClass)(env, classname);
    ((**env).RegisterNatives)(env, class, methods.as_ptr(), methods.len() as i32);
}

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn JNI_OnLoad(vm: *mut JavaVM, reserved: *mut c_void) -> jint {
    logi!("jni onload!!");
    let mut env: *mut c_void = ptr::null_mut();
    if ((**vm).GetEnv)(vm, (&mut env as *mut *mut c_void), JNI_VERSION_1_6) != JNI_OK {
        return -1;
    }
    let env = env as *mut JNIEnv;
    logi!("got environment!: {}", env);
    MOTION_CLASS = ((**env).FindClass)(env, cstr!("android/view/MotionEvent"));
    MOTIONEVENT_NATIVE_PTR_FIELD = ((**env).GetFieldID)(env, MOTION_CLASS, cstr!("mNativePtr"), cstr!("I"));
    logi!("got motion classes");
    SCALA_LEFT = CaseClass::new(env, cstr!("scala/util/Left"), cstr!("(Ljava/lang/Object;)V"));
    SCALA_RIGHT = CaseClass::new(env, cstr!("scala/util/Right"), cstr!("(Ljava/lang/Object;)V"));
    BOXED_JINT = CaseClass::new(env, cstr!("java/lang/Integer"), cstr!("(I)V"));

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
        native_method!("compile", "(ILjava/lang/String;Ljava/lang/String;)Lscala/util/Either;", compile_pointshader),
    ];
    let copyshaderstaticmethods = [
        native_method!("compile", "(ILjava/lang/String;Ljava/lang/String;)Lscala/util/Either;", compile_copyshader),
    ];
    let texturestaticmethods = [
        native_method!("init", "(ILandroid/graphics/Bitmap;)Lscala/util/Either;", create_texture),
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
        native_method!("init", "(ILjava/lang/String;)Lscala/util/Either;", jni_lua_compile_script),
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

    let replayhandlerstaticmethods = [
        native_method!("init", "(I)I", jni_replay_begin),
        native_method!("destroy", "(I)V", jni_replay_destroy),
        native_method!("advanceFrame", "(II[F)Z", jni_replay_advance_frame),
    ];
    register_classmethods(env, cstr!("com/github/wartman4404/gldraw/Replay$"), replayhandlerstaticmethods);
    logi!("registered replay methods!");
    logi!("finished jni_onload");
    JNI_VERSION_1_2
}

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn JNI_OnUnload(vm: *mut JavaVM, reserved: *mut c_void) {
    logi!("jni onload!!");
    let mut env: *mut c_void = ptr::null_mut();
    if ((**vm).GetEnv)(vm, (&mut env as *mut *mut c_void), JNI_VERSION_1_6) != JNI_OK {
        return;
    }
    let env = env as *mut JNIEnv;
    SCALA_LEFT.destroy(env);
    SCALA_RIGHT.destroy(env);
    BOXED_JINT.destroy(env);
}
