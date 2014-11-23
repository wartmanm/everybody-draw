use core::prelude::*;
use core::{ptr, mem};
use core::ptr::RawMutPtr;
use core::raw;
use alloc::boxed::Box;
use libc::{c_void, c_char};
use collections::string::String;
use collections::str::{MaybeOwned, IntoMaybeOwned};
use collections::vec::Vec;

use jni::{jobject, jclass, jfieldID, jmethodID, JNIEnv, jint, jfloat, jstring, jboolean, jvalue, jfloatArray, JNINativeMethod, JavaVM};
use android::input::AInputEvent;
use android::bitmap::{AndroidBitmap_getInfo, AndroidBitmap_lockPixels, AndroidBitmap_unlockPixels, AndroidBitmapInfo};
use android::bitmap::{ANDROID_BITMAP_FORMAT_RGBA_8888, ANDROID_BITMAP_FORMAT_A_8};
use android::native_window_jni::{ANativeWindow_fromSurface};//, ANativeWindow_release};
use android::native_window::ANativeWindow_release;
use glcommon::{GLResult, MString};

use log::{logi, loge};

use glstore::DrawObjectIndex;
use glinit::{GLInit, AndroidBitmapFormat};
use glpoint;
use matrix::Matrix;
use eglinit;
use jni_constants::*;
use jni_helpers::ToJValue;
use drawevent::Events;
use drawevent::event_stream::EventStream;
use gltexture::ToPixelFormat;
use gltexture::{Texture, BrushTexture};
use redirect_stderr;
use redirect_stderr::Struct_stdout_forwarder;
use rustjni::android_bitmap::{AndroidBitmap};

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

static mut REDIRECT_PTR: *mut () = 0 as *mut ();

struct CaseClass {
    constructor: jmethodID,
    class: jclass,
}

struct GLInitEvents<'a> {
    glinit: GLInit<'a>,
    events: Events<'a>,
    jni_undo_callback: JNIUndoCallback,
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

static mut LUA_EXCEPTION: CaseClass = CaseClass { constructor: 0 as jmethodID, class: 0 as jclass };
static mut GL_EXCEPTION: CaseClass = CaseClass { constructor: 0 as jmethodID, class: 0 as jclass };
static mut SCALA_TUPLE2: CaseClass = CaseClass { constructor: 0 as jmethodID, class: 0 as jclass };

struct JNIUndoCallback {
    callback_obj: jobject,
    callback_method: jmethodID,
}

pub struct JNICallbackClosure<'a> {
    undo_callback: &'a JNIUndoCallback,
    env: *mut JNIEnv,
}

impl<'a> ::core::ops::Fn<(i32,), ()> for JNICallbackClosure<'a> {
    extern "rust-call" fn call(&self, args: (i32,)) -> () {
        let (arg,) = args;
        unsafe {
            self.undo_callback.call(self.env, arg);
        }
    }
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

unsafe fn glresult_or_exception<T>(env: *mut JNIEnv, result: GLResult<DrawObjectIndex<T>>) -> jint {
    logi!("in glresult_or_exception");
    match result {
        Err(msg) => {
            let errmsg = str_to_jstring(env, msg.as_slice()).as_jvalue();
            let err = GL_EXCEPTION.construct(env, [errmsg].as_mut_slice());
            ((**env).Throw)(env, err);
            -1
        },
        Ok(idx) => mem::transmute(idx),
    }
}

unsafe fn str_to_jstring(env: *mut JNIEnv, s: &str) -> jstring {
    let u16msg: Vec<u16> = s.utf16_units().collect();
    ((**env).NewString)(env, u16msg.as_ptr(), u16msg.len() as i32)
}

fn get_safe_data<'a>(data: i32) -> &'a mut GLInitEvents<'a> {
    unsafe { mem::transmute(data) }
}

unsafe extern "C" fn init_gl(env: *mut JNIEnv, _: jobject, w: jint, h: jint, callback: jobject) -> jint {
    mem::transmute(box GLInitEvents {
        glinit: GLInit::setup_graphics(w, h),
        events: Events::new(),
        jni_undo_callback: JNIUndoCallback::new(env, callback),
    })
}

unsafe extern "C" fn finish_gl(env: *mut JNIEnv, _: jobject, data: jint) {
    let mut data: Box<GLInitEvents> = mem::transmute(data);
    data.jni_undo_callback.destroy(env);
    data.glinit.destroy();
    logi!("finished deinit");
}

unsafe fn rethrow_lua_result(env: *mut JNIEnv, result: GLResult<()>) {
    if let Err(msg) = result {
        let errmsg = str_to_jstring(env, msg.as_slice()).as_jvalue();
        let err = LUA_EXCEPTION.construct(env, [errmsg].as_mut_slice());
        ((**env).Throw)(env, err);
    }
}

unsafe extern "C" fn native_draw_queued_points(env: *mut JNIEnv, _: jobject, data: i32, handler: i32, java_matrix: jfloatArray) {
    let data = get_safe_data(data);
    let callback = data.jni_undo_callback.create_closure(env);
    //let callback = |&: x: i32| {
        //data.jni_undo_callback.call(env, x);
    //};
    let mut matrix: Matrix = mem::uninitialized();
    ((**env).GetFloatArrayRegion)(env, java_matrix, 0, 16, matrix.as_mut_ptr());
    let luaerr = data.glinit.draw_queued_points(mem::transmute(handler), &mut data.events, &matrix, &callback);
    rethrow_lua_result(env, luaerr);
}

unsafe extern "C" fn native_finish_lua_script(env: *mut JNIEnv, _: jobject, data: i32, handler: i32) {
    let data = get_safe_data(data);
    let callback = data.jni_undo_callback.create_closure(env);
    let luaerr = data.glinit.unload_interpolator(mem::transmute(handler), &mut data.events, &callback);
    rethrow_lua_result(env, luaerr);
}

unsafe extern "C" fn native_update_gl(_: *mut JNIEnv, _: jobject, data: i32) {
    let data = get_safe_data(data);
    data.glinit.render_frame();
    data.events.pushframe(); // FIXME make sure a frame was actually drawn! No java exceptions, missing copy shader, etc
}

unsafe extern "C" fn init_motion_event_handler(env: *mut JNIEnv, _: jobject) -> jobject {
    let (consumer, producer) = glpoint::create_motion_event_handler();
    let (consumer, producer) = (box consumer, box producer);
    let pairclass = ((**env).FindClass)(env, cstr!("com/github/wartman4404/gldraw/MotionEventHandlerPair"));
    let constructor = ((**env).GetMethodID)(env, pairclass, cstr!("<init>"), cstr!("(II)V"));
    let (consumer, producer): (i32, i32) = (mem::transmute(consumer), mem::transmute(producer));
    ((**env).NewObject)(env, pairclass, constructor, consumer, producer)
}

unsafe extern "C" fn destroy_motion_event_handler(env: *mut JNIEnv, _: jobject, pairobj: jobject) {
    let pairclass = ((**env).FindClass)(env, cstr!("com/github/wartman4404/gldraw/MotionEventHandlerPair"));
    let consumerfield = ((**env).GetFieldID)(env, pairclass, cstr!("consumer"), cstr!("I"));
    let producerfield = ((**env).GetFieldID)(env, pairclass, cstr!("producer"), cstr!("I"));
    let consumer = ((**env).GetIntField)(env, pairobj, consumerfield);
    let producer = ((**env).GetIntField)(env, pairobj, producerfield);
    glpoint::destroy_motion_event_handler(mem::transmute(consumer), mem::transmute(producer));
}

unsafe extern "C" fn native_append_motion_event(env: *mut JNIEnv, _: jobject, handler: jint, evtobj: jobject) {
    let evtptr = ((**env).GetIntField)(env, evtobj, MOTIONEVENT_NATIVE_PTR_FIELD);
    glpoint::jni_append_motion_event(mem::transmute(handler), evtptr as *const AInputEvent);
}

unsafe extern "C" fn native_pause_motion_event(_: *mut JNIEnv, _: jobject, handler: jint) {
    glpoint::jni_pause_motion_event(mem::transmute(handler));
}

unsafe extern "C" fn set_anim_shader(_: *mut JNIEnv, _: jobject, data: jint, shader: jint) {
    let data = get_safe_data(data);
    let shader = data.events.use_animshader(mem::transmute(shader));
    data.glinit.set_anim_shader(shader);
}

unsafe extern "C" fn set_copy_shader(_: *mut JNIEnv, _: jobject, data: jint, shader: jint) {
    let data = get_safe_data(data);
    let shader = data.events.use_copyshader(mem::transmute(shader));
    data.glinit.set_copy_shader(shader);
}

unsafe extern "C" fn set_point_shader(_: *mut JNIEnv, _: jobject, data: jint, shader: jint) {
    let data = get_safe_data(data);
    let shader = data.events.use_pointshader(mem::transmute(shader));
    data.glinit.set_point_shader(shader);
}

unsafe extern "C" fn set_brush_texture(_: *mut JNIEnv, _: jobject, data: jint, texture: jint) {
    let data = get_safe_data(data);
    let brush = data.events.use_brush(mem::transmute(texture));
    data.glinit.set_brush_texture(&brush.texture);
}

unsafe fn safe_create_texture(env: *mut JNIEnv, data: jint, bitmap: jobject) -> GLResult<DrawObjectIndex<BrushTexture>> {
    let mut bitmap = AndroidBitmap::from_jobject(env, bitmap);
    let (w, h) = (bitmap.info.width, bitmap.info.height);
    let format: AndroidBitmapFormat = mem::transmute(bitmap.info.format);
    let texformat = try!(format.to_pixelformat());
    Ok(get_safe_data(data).events.load_brush(w as i32, h as i32, bitmap.as_slice(), texformat))
}

unsafe extern "C" fn create_texture(env: *mut JNIEnv, _: jobject, data: jint, bitmap: jobject) -> jint {
    glresult_or_exception(env, safe_create_texture(env, data, bitmap))
}

unsafe extern "C" fn clear_framebuffer(_: *mut JNIEnv, _: jobject, data: jint) {
    let data = get_safe_data(data);
    data.events.clear();
    data.glinit.clear_buffer();
}

unsafe fn get_string(env: *mut JNIEnv, string: jstring) -> Option<MString> {
    let string = try_opt!(string.as_mut());
    let c = try_opt!(((**env).GetStringChars)(env, string, ptr::null_mut()).as_ref());
    let len = ((**env).GetStringLength)(env, string);
    let strslice: &[u16] = mem::transmute(raw::Slice { data: c, len: len as uint });
    let ruststr = String::from_utf16(strslice);
    ((**env).ReleaseStringChars)(env, string as jstring, strslice.as_ptr());
    Some(try_opt!(ruststr).into_maybe_owned())
}

unsafe extern "C" fn compile_copyshader(env: *mut JNIEnv, _: jobject, data: i32, vec: jstring, frag: jstring) -> jint {
    glresult_or_exception(env, get_safe_data(data).events.load_copyshader(get_string(env, vec), get_string(env, frag)))
}

unsafe extern "C" fn compile_pointshader(env: *mut JNIEnv, _: jobject, data: i32, vec: jstring, frag: jstring) -> jint {
    glresult_or_exception(env, get_safe_data(data).events.load_pointshader(get_string(env, vec), get_string(env, frag)))
}

pub unsafe extern "C" fn draw_image(env: *mut JNIEnv, _: jobject, data: i32, bitmap: jobject) {
    // TODO: ensure rgba_8888 format and throw error
    let bitmap = AndroidBitmap::from_jobject(env, bitmap);
    let pixels = bitmap.as_slice();
    get_safe_data(data).glinit.draw_image(bitmap.info.width as i32, bitmap.info.height as i32, pixels);
}

pub unsafe extern "C" fn export_pixels(env: *mut JNIEnv, _: jobject, data: i32) -> jobject {
    let glinit = &mut get_safe_data(data).glinit;
    let (w, h) = glinit.get_buffer_dimensions();
    let mut bitmap = AndroidBitmap::new(env, w, h);
    glinit.get_pixels(bitmap.as_mut_slice());
    bitmap.set_premultiplied(true);
    bitmap.obj
}

mod android_bitmap {
    use core::prelude::*;
    use core::raw;
    use core::{ptr, mem};
    use libc::{c_void, c_char};
    use jni::{jobject, jclass, jfieldID, jmethodID, JNIEnv, jint, jfloat, jstring, jboolean, jvalue, jfloatArray, JNINativeMethod, JavaVM};
    use jni_constants::{JNI_TRUE, JNI_FALSE};
    use log::{logi, loge};
    use android::bitmap::{AndroidBitmap_getInfo, AndroidBitmap_lockPixels, AndroidBitmap_unlockPixels, AndroidBitmapInfo};
    use android::bitmap::{ANDROID_BITMAP_FORMAT_RGBA_8888, ANDROID_BITMAP_FORMAT_A_8};
    static mut BITMAP_CLASS: jclass = 0 as jclass;
    static mut CONFIG_ARGB_8888: jobject = 0 as jobject;
    static mut CREATE_BITMAP: jmethodID = 0 as jmethodID;
    static mut SET_PREMULTIPLIED: jmethodID = 0 as jmethodID;

    pub struct AndroidBitmap {
        env: *mut JNIEnv,
        pub obj: jobject,
        pixels: *mut u8,
        pub info: AndroidBitmapInfo,
    }
    
    pub unsafe fn init(env: *mut JNIEnv) {
        let bitmapclass = ((**env).FindClass)(env, cstr!("android/graphics/Bitmap"));
        let configclass = ((**env).FindClass)(env, cstr!("android/graphics/Bitmap$Config"));
        let argbfield = ((**env).GetStaticFieldID)(env, configclass, cstr!("ARGB_8888"), cstr!("Landroid/graphics/Bitmap$Config;"));
        let argb = ((**env).GetStaticObjectField)(env, configclass, argbfield);
        let createbitmap = ((**env).GetStaticMethodID)(env, bitmapclass, cstr!("createBitmap"), cstr!("(IILandroid/graphics/Bitmap$Config;)Landroid/graphics/Bitmap;"));
        let premult = ((**env).GetMethodID)(env, bitmapclass, cstr!("setPremultiplied"), cstr!("(Z)V"));
        BITMAP_CLASS = ((**env).NewGlobalRef)(env, bitmapclass);
        CONFIG_ARGB_8888 = ((**env).NewGlobalRef)(env, argb);
        CREATE_BITMAP = createbitmap;
        SET_PREMULTIPLIED = premult;
    }

    impl AndroidBitmap {
        pub unsafe fn from_jobject(env: *mut JNIEnv, bitmap: jobject) -> AndroidBitmap {
            let mut pixels: *mut c_void = ptr::null_mut();
            AndroidBitmap_lockPixels(env, bitmap, &mut pixels);
            logi!("locked pixels in {}", pixels);
            let mut result = AndroidBitmap { env: env, obj: bitmap, pixels: pixels as *mut u8, info: mem::zeroed() };
            AndroidBitmap_getInfo(env, bitmap, &mut result.info);
            result
        }

        pub unsafe fn new(env: *mut JNIEnv, w: i32, h: i32) -> AndroidBitmap {
            let bitmap = ((**env).CallStaticObjectMethod)(env, BITMAP_CLASS, CREATE_BITMAP, w, h, CONFIG_ARGB_8888);
            logi!("created bitmap");
            AndroidBitmap::from_jobject(env, bitmap)
        }

        unsafe fn as_slice_unsafe(&self) -> &mut [u8] {
            let pixelsize = match self.info.format as u32 {
                ANDROID_BITMAP_FORMAT_RGBA_8888 => 4,
                ANDROID_BITMAP_FORMAT_A_8 => 1,
                x => panic!("bitmap format {} not implemented!", x),
            };
            let pixelvec = raw::Slice { data: self.pixels as *const u8, len: (self.info.width * self.info.height * pixelsize) as uint };
            mem::transmute(pixelvec)
        }

        pub unsafe fn as_mut_slice(&mut self) -> &mut [u8] {
            self.as_slice_unsafe()
        }

        pub unsafe fn as_slice(&self) -> &[u8] {
            self.as_slice_unsafe()
        }

        pub unsafe fn set_premultiplied(&mut self, premultiplied: bool) {
            let pm = if premultiplied { JNI_TRUE } else { JNI_FALSE };
            ((**self.env).CallVoidMethod)(self.env, BITMAP_CLASS, SET_PREMULTIPLIED, self.obj, pm);
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

    pub unsafe fn destroy(env: *mut JNIEnv) {
        ((**env).DeleteGlobalRef)(env, BITMAP_CLASS);
        ((**env).DeleteGlobalRef)(env, CONFIG_ARGB_8888);
    }
}

unsafe extern "C" fn jni_egl_finish(_: *mut JNIEnv, _: jobject) {
    eglinit::egl_finish();
}

unsafe extern "C" fn jni_egl_init(env: *mut JNIEnv, _: jobject, surface: jobject) {
    let window = ANativeWindow_fromSurface(env, surface);
    logi!("got ANAtiveWindow: 0x{:x}", window as u32);
    eglinit::egl_init(window as *mut c_void);
    ANativeWindow_release(window);
}

unsafe extern "C" fn jni_lua_compile_script(env: *mut JNIEnv, _: jobject, data: i32, script: jstring) -> jint {
    let scriptstr = get_string(env, script);
    glresult_or_exception(env, get_safe_data(data).events.load_interpolator(scriptstr))
}

unsafe extern "C" fn jni_lua_set_interpolator(_: *mut JNIEnv, _: jobject, data: jint, scriptid: jint) {
    let data = get_safe_data(data);
    let script = data.events.use_interpolator(mem::transmute(scriptid));
    data.glinit.set_interpolator(script);
}

unsafe extern "C" fn jni_add_layer(_: *mut JNIEnv, _: jobject, data: jint, copyshader: jint, pointshader: jint, pointidx: jint) {
    let data = get_safe_data(data);
    let copyshader = Some(mem::transmute(copyshader));
    let pointshader = Some(mem::transmute(pointshader));
    let layer = data.events.add_layer(data.glinit.dimensions, copyshader, pointshader, mem::transmute(pointidx));
    data.glinit.add_layer(layer);
}

unsafe extern "C" fn jni_clear_layers(_: *mut JNIEnv, _: jobject, data: jint) {
    let data = get_safe_data(data);
    data.events.clear_layers();
    data.glinit.clear_layers();
}

unsafe extern "C" fn jni_replay_begin(_: *mut JNIEnv, _: jobject, data: jint) -> jint {
    let data = get_safe_data(data);
    data.glinit.clear_layers();
    data.glinit.clear_buffer();
    mem::transmute(box EventStream::new())
}

#[allow(unused)]
unsafe extern "C" fn jni_replay_advance_frame(env: *mut JNIEnv, _: jobject, data: jint, replay: jint, java_matrix: jfloatArray) -> jboolean {
    let data = get_safe_data(data);
    let replay: &mut EventStream = mem::transmute(replay);
    let mut matrix: Matrix = mem::uninitialized();
    ((**env).GetFloatArrayRegion)(env, java_matrix, 0, 16, matrix.as_mut_ptr());
    let done = replay.advance_frame(&mut data.glinit, &mut data.events);
    let callback = data.jni_undo_callback.create_closure(env);
    data.glinit.draw_queued_points(&mut replay.consumer, &mut data.events, &matrix, &callback);
    if done { JNI_TRUE as jboolean } else { JNI_FALSE as jboolean }
}

unsafe extern "C" fn jni_replay_destroy(_: *mut JNIEnv, _: jobject, replay: jint) {
    let replay: Box<EventStream> = mem::transmute(replay);
    mem::drop(replay);
}

unsafe extern "C" fn jni_load_undo(_: *mut JNIEnv, _: jobject, data: jint, idx: jint) {
    let data = get_safe_data(data);
    data.glinit.load_undo_frame(idx);
}

unsafe extern "C" fn jni_set_brush_color(_: *mut JNIEnv, _: jobject, data: jint, color: jint) {
    get_safe_data(data).glinit.set_brush_color(color);
}

unsafe extern "C" fn jni_set_brush_size(_: *mut JNIEnv, _: jobject, data: jint, size: jfloat) {
    get_safe_data(data).glinit.set_brush_size(size);
}

unsafe fn get_shader_source_tuple(env: *mut JNIEnv, source: &(MString, MString)) -> jobject {
    let &(ref vert, ref frag) = source;
    let mut jvert = str_to_jstring(env, vert.as_slice());
    let mut jfrag = str_to_jstring(env, frag.as_slice());
    SCALA_TUPLE2.construct(env, [jvert.as_jvalue(), jfrag.as_jvalue()].as_mut_slice())
}

#[no_mangle]
pub unsafe extern "C" fn jni_get_copyshader_source(env: *mut JNIEnv, _: jobject, data: jint, copyshader: jint) -> jobject {
    logi!("getting copyshader source");
    let source = get_safe_data(data).events.get_copyshader_source(mem::transmute(copyshader));
    logi!("got copyshader source");
    let tuple = get_shader_source_tuple(env, source);
    logi!("created tuple");
    tuple
}

unsafe extern "C" fn jni_get_pointshader_source(env: *mut JNIEnv, _: jobject, data: jint, pointshader: jint) -> jobject {
    logi!("getting pointshader source");
    let source = get_safe_data(data).events.get_pointshader_source(mem::transmute(pointshader));
    get_shader_source_tuple(env, source)
}

unsafe extern "C" fn jni_get_luascript_source(env: *mut JNIEnv, _: jobject, data: jint, luascript: jint) -> jstring {
    logi!("getting luascript source");
    let source = get_safe_data(data).events.get_luascript_source(mem::transmute(luascript));
    str_to_jstring(env, source.as_slice())
}

unsafe fn register_classmethods(env: *mut JNIEnv, classname: *const i8, methods: &[JNINativeMethod]) {
    let class = ((**env).FindClass)(env, classname);
    ((**env).RegisterNatives)(env, class, methods.as_ptr(), methods.len() as i32);
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
    MOTION_CLASS = ((**env).FindClass)(env, cstr!("android/view/MotionEvent"));
    MOTIONEVENT_NATIVE_PTR_FIELD = ((**env).GetFieldID)(env, MOTION_CLASS, cstr!("mNativePtr"), cstr!("I"));
    logi!("got motion classes");
    GL_EXCEPTION = CaseClass::new(env, cstr!("com/github/wartman4404/gldraw/GLException"), cstr!("(Ljava/lang/String;)V"));
    LUA_EXCEPTION = CaseClass::new(env, cstr!("com/github/wartman4404/gldraw/LuaException"), cstr!("(Ljava/lang/String;)V")); 
    SCALA_TUPLE2 = CaseClass::new(env, cstr!("scala/Tuple2"), cstr!("(Ljava/lang/Object;Ljava/lang/Object;)V"));
    android_bitmap::init(env);

    let mainmethods = [
        native_method!("nativeAppendMotionEvent", "(ILandroid/view/MotionEvent;)V", native_append_motion_event),
        native_method!("nativePauseMotionEvent", "(I)V", native_pause_motion_event),
    ];
    register_classmethods(env, cstr!("com/github/wartman4404/gldraw/MainActivity"), mainmethods);

    let texturemethods = [
        native_method!("nativeUpdateGL", "(I)V", native_update_gl),
        native_method!("nativeDrawQueuedPoints", "(II[F)V", native_draw_queued_points),
        native_method!("nativeFinishLuaScript", "(II)V", native_finish_lua_script),
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
        native_method!("nativeLoadUndo", "(II)V", jni_load_undo),
        native_method!("nativeSetBrushColor", "(II)V", jni_set_brush_color),
        native_method!("nativeSetBrushSize", "(IF)V", jni_set_brush_size),
    ];
    register_classmethods(env, cstr!("com/github/wartman4404/gldraw/TextureSurfaceThread"), texturemethods);
    logi!("registered texture thread methods!");

    let pointshaderstaticmethods = [
        native_method!("compile", "(ILjava/lang/String;Ljava/lang/String;)I", compile_pointshader),
        native_method!("getSource", "(II)Lscala/Tuple2;", jni_get_pointshader_source),
    ];
    let copyshaderstaticmethods = [
        native_method!("compile", "(ILjava/lang/String;Ljava/lang/String;)I", compile_copyshader),
        native_method!("getSource", "(II)Lscala/Tuple2;", jni_get_copyshader_source),
    ];
    let texturestaticmethods = [
        native_method!("init", "(ILandroid/graphics/Bitmap;)I", create_texture),
    ];
    register_classmethods(env, cstr!("com/github/wartman4404/gldraw/PointShader$"), pointshaderstaticmethods);
    register_classmethods(env, cstr!("com/github/wartman4404/gldraw/CopyShader$"), copyshaderstaticmethods);
    register_classmethods(env, cstr!("com/github/wartman4404/gldraw/TexturePtr$"), texturestaticmethods);
    logi!("registered point|copy|texture static methods!");

    let eglhelpermethods = [
        native_method!("nativeFinish", "()V", jni_egl_finish),
        native_method!("nativeInit", "(Landroid/view/Surface;)V", jni_egl_init),
    ];
    register_classmethods(env, cstr!("com/github/wartman4404/gldraw/EGLHelper"), eglhelpermethods);
    logi!("registered egl methods!");

    let luastaticmethods = [
        native_method!("init", "(ILjava/lang/String;)I", jni_lua_compile_script),
        native_method!("getSource", "(II)Ljava/lang/String;", jni_get_luascript_source),
    ];
    register_classmethods(env, cstr!("com/github/wartman4404/gldraw/LuaScript$"), luastaticmethods);
    logi!("registered lua methods!");

    let glinitstaticmethods = [
        native_method!("initGL", "(IILcom/github/wartman4404/gldraw/UndoCallback;)I", init_gl),
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

#[allow(non_snake_case, unused_variables)]
#[no_mangle]
pub unsafe extern "C" fn JNI_OnUnload(vm: *mut JavaVM, reserved: *mut c_void) {
    logi!("jni onload!!");
    let mut env: *mut c_void = ptr::null_mut();
    if ((**vm).GetEnv)(vm, (&mut env as *mut *mut c_void), JNI_VERSION_1_6) != JNI_OK {
        return;
    }
    let env = env as *mut JNIEnv;
    GL_EXCEPTION.destroy(env);
    LUA_EXCEPTION.destroy(env);
    SCALA_TUPLE2.destroy(env);
    android_bitmap::destroy(env);
}
