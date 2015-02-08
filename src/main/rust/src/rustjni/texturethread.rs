use core::prelude::*;
use core::mem;
use alloc::boxed::Box;
use jni::{jobject, jclass, jmethodID, JNIEnv, jint, jfloat, jboolean, jfloatArray, JNINativeMethod};

use glcommon::GLResult;
use glinit::GLInit;
use drawevent::event_stream::EventStream;
use rustjni::android_bitmap::AndroidBitmap;
use drawevent::Events;
use matrix::Matrix;

use rustjni::{register_classmethods, CaseClass, get_safe_data, str_to_jstring, GLInitEvents, JNIUndoCallback, JNICallbackClosure, jpointer, GL_EXCEPTION};
use jni_helpers::ToJValue;
use jni_constants::*;
use lua_geom;

static mut LUA_EXCEPTION: CaseClass = CaseClass { constructor: 0 as jmethodID, class: 0 as jclass };
static mut RUNTIME_EXCEPTION: CaseClass = CaseClass { constructor: 0 as jmethodID, class: 0 as jclass };

impl<'a> ::core::ops::Fn<(i32,)> for JNICallbackClosure<'a> {
    type Output = ();
    extern "rust-call" fn call(&self, args: (i32,)) -> () {
        let (arg,) = args;
        unsafe {
            self.undo_callback.call(self.env, arg);
        }
    }
}

unsafe fn rethrow_lua_result(env: *mut JNIEnv, result: GLResult<()>) {
    if let Err(msg) = result {
        let errmsg = str_to_jstring(env, msg.as_slice()).as_jvalue();
        let err = LUA_EXCEPTION.construct(env, [errmsg].as_mut_slice());
        ((**env).Throw)(env, err);
    }
}

unsafe extern "C" fn init_gl(env: *mut JNIEnv, _: jobject, w: jint, h: jint, callback: jobject) -> jpointer {
    let mut events = Events::new();
    let glinit = GLInit::setup_graphics(w, h, &mut events);
    let jni_undo_callback = JNIUndoCallback::new(env, callback);
    let _ = lua_geom::ensure_lua_exists(w, h);
    mem::transmute(Box::new(GLInitEvents {
        glinit: glinit,
        events: events,
        jni_undo_callback: jni_undo_callback,
        owning_thread: ::rustjni::gettid(),
        /* lua: lua */
    }))
}

unsafe extern "C" fn finish_gl(env: *mut JNIEnv, _: jobject, data: jpointer) {
    let mut data: Box<GLInitEvents> = mem::transmute(data);
    data.jni_undo_callback.destroy(env);
    data.glinit.destroy();
    debug_logi!("finished deinit");
}

unsafe extern "C" fn native_draw_queued_points(env: *mut JNIEnv, _: jobject, data: jpointer, handler: jpointer, java_matrix: jfloatArray) {
    let data = get_safe_data(data);
    let callback = data.jni_undo_callback.create_closure(env);
    let mut matrix: Matrix = mem::uninitialized();
    ((**env).GetFloatArrayRegion)(env, java_matrix, 0, 16, matrix.as_mut_ptr());
    let luaerr = data.glinit.draw_queued_points(mem::transmute(handler), &mut data.events, &matrix, &callback);
    rethrow_lua_result(env, luaerr);
}

unsafe extern "C" fn native_finish_lua_script(env: *mut JNIEnv, _: jobject, data: jpointer, handler: jpointer) {
    let data = get_safe_data(data);
    let callback = data.jni_undo_callback.create_closure(env);
    let luaerr = data.glinit.unload_interpolator(mem::transmute(handler), &mut data.events, &callback);
    rethrow_lua_result(env, luaerr);
}

unsafe extern "C" fn native_update_gl(_: *mut JNIEnv, _: jobject, data: jpointer) {
    let data = get_safe_data(data);
    data.glinit.render_frame();
    data.events.pushframe(); // FIXME make sure a frame was actually drawn! No java exceptions, missing copy shader, etc
}



unsafe extern "C" fn set_anim_shader(env: *mut JNIEnv, _: jobject, data: jpointer, shader: jint) {
    let data = get_safe_data(data);
    let shader = try_or_throw!(env, RUNTIME_EXCEPTION, data.events.use_animshader(mem::transmute(shader)));
    data.glinit.set_anim_shader(shader);
}

unsafe extern "C" fn set_copy_shader(env: *mut JNIEnv, _: jobject, data: jpointer, shader: jint) {
    let data = get_safe_data(data);
    let shader = try_or_throw!(env, RUNTIME_EXCEPTION, data.events.use_copyshader(mem::transmute(shader)));
    data.glinit.set_copy_shader(shader);
}

unsafe extern "C" fn set_point_shader(env: *mut JNIEnv, _: jobject, data: jpointer, shader: jint) {
    let data = get_safe_data(data);
    let shader = try_or_throw!(env, RUNTIME_EXCEPTION, data.events.use_pointshader(mem::transmute(shader)));
    data.glinit.set_point_shader(shader);
}

unsafe extern "C" fn set_brush_texture(env: *mut JNIEnv, _: jobject, data: jpointer, texture: jint) {
    let data = get_safe_data(data);
    let brush = try_or_throw!(env, RUNTIME_EXCEPTION, data.events.use_brush(mem::transmute(texture)));
    data.glinit.set_brush_texture(&brush.texture);
}

unsafe extern "C" fn clear_framebuffer(_: *mut JNIEnv, _: jobject, data: jpointer) {
    let data = get_safe_data(data);
    data.events.clear();
    data.glinit.clear_buffer();
}

pub unsafe extern "C" fn export_pixels(env: *mut JNIEnv, _: jobject, data: jpointer) -> jobject {
    let glinit = &mut get_safe_data(data).glinit;
    let (w, h) = glinit.get_buffer_dimensions();
    let mut bitmap = AndroidBitmap::new(env, w, h);
    glinit.get_pixels(bitmap.as_mut_slice().unwrap());
    bitmap.set_premultiplied(true);
    bitmap.obj
}

pub unsafe extern "C" fn draw_image(env: *mut JNIEnv, _: jobject, data: jpointer, bitmap: jobject, rotation: jint) {
    let bitmap = AndroidBitmap::from_jobject(env, bitmap);

    // This is really dumb.
    // AndroidBitmap_unlockPixels can't be called with a pending exception.
    // So, bitmap has to be dropped before we can throw.  But it can't be dropped in either arm
    // because bitmap.as_slice() is technically still borrowing it even though
    //  - nothing uses it past that point
    //  - the match arm consumes it
    //  - Err(_) doesn't even use the bitmap's lifetime anywhere
    //  In conclusion, blargh.
    let exception = match bitmap.as_slice() {
        Ok(pixels) => {
            let data = get_safe_data(data);
            data.glinit.draw_image(bitmap.info.width as i32, bitmap.info.height as i32, pixels, mem::transmute(rotation));
            return;
        },
        Err(err) => {
            // this must be done manually, as AndroidBitmap_unlockPixels cannot be called with a
            // pending exception
            let errmsg = str_to_jstring(env, format!("{}", err).as_slice()).as_jvalue();
            GL_EXCEPTION.construct(env, [errmsg].as_mut_slice())
        }
    };
    mem::drop(bitmap);
    ((**env).Throw)(env, exception);
}

unsafe extern "C" fn jni_lua_set_interpolator(env: *mut JNIEnv, _: jobject, data: jpointer, scriptid: jint) {
    let data = get_safe_data(data);
    let script = try_or_throw!(env, RUNTIME_EXCEPTION, data.events.use_interpolator(mem::transmute(scriptid)));
    data.glinit.set_interpolator(script);
}

unsafe extern "C" fn jni_add_layer(_: *mut JNIEnv, _: jobject, data: jpointer, copyshader: jint, pointshader: jint, pointidx: jint) {
    let data = get_safe_data(data);
    let copyshader = Some(mem::transmute(copyshader));
    let pointshader = Some(mem::transmute(pointshader));
    let layer = data.events.add_layer(data.glinit.dimensions, copyshader, pointshader, mem::transmute(pointidx));
    data.glinit.add_layer(layer);
}

unsafe extern "C" fn jni_clear_layers(_: *mut JNIEnv, _: jobject, data: jpointer) {
    let data = get_safe_data(data);
    data.events.clear_layers();
    data.glinit.clear_layers();
}

unsafe extern "C" fn jni_replay_begin(_: *mut JNIEnv, _: jobject, data: jpointer) -> jpointer {
    let data = get_safe_data(data);
    data.glinit.clear_layers();
    data.glinit.clear_buffer();
    mem::transmute(Box::new(EventStream::new()))
}

#[allow(unused)]
unsafe extern "C" fn jni_replay_advance_frame(env: *mut JNIEnv, _: jobject, data: jpointer, replay: jpointer, java_matrix: jfloatArray) -> jboolean {
    let data = get_safe_data(data);
    let replay: &mut EventStream = mem::transmute(replay);
    let mut matrix: Matrix = mem::uninitialized();
    ((**env).GetFloatArrayRegion)(env, java_matrix, 0, 16, matrix.as_mut_ptr());
    let done = replay.advance_frame(&mut data.glinit, &mut data.events);
    let callback = data.jni_undo_callback.create_closure(env);
    data.glinit.draw_queued_points(&mut replay.consumer, &mut data.events, &matrix, &callback);
    if done { JNI_TRUE as jboolean } else { JNI_FALSE as jboolean }
}

unsafe extern "C" fn jni_replay_destroy(_: *mut JNIEnv, _: jobject, replay: jpointer) {
    let replay: Box<EventStream> = mem::transmute(replay);
    mem::drop(replay);
}

unsafe extern "C" fn jni_load_undo(_: *mut JNIEnv, _: jobject, data: jpointer, idx: jint) {
    let data = get_safe_data(data);
    data.glinit.load_undo_frame(idx);
}

unsafe extern "C" fn jni_push_undo_frame(_: *mut JNIEnv, _: jobject, data: jpointer) -> jint {
    let data = get_safe_data(data);
    data.glinit.push_undo_frame()
}

unsafe extern "C" fn jni_clear_undo_frames(_: *mut JNIEnv, _: jobject, data: jpointer) {
    let data = get_safe_data(data);
    data.glinit.clear_undo_frames();
}

unsafe extern "C" fn jni_set_brush_color(_: *mut JNIEnv, _: jobject, data: jpointer, color: jint) {
    get_safe_data(data).glinit.set_brush_color(color);
}

unsafe extern "C" fn jni_set_brush_size(_: *mut JNIEnv, _: jobject, data: jpointer, size: jfloat) {
    get_safe_data(data).glinit.set_brush_size(size);
}

pub unsafe fn init(env: *mut JNIEnv) {
    LUA_EXCEPTION = CaseClass::new(env, cstr!("com/github/wartman4404/gldraw/LuaException"), cstr!("(Ljava/lang/String;)V")); 
    RUNTIME_EXCEPTION = CaseClass::new(env, cstr!("java/lang/IndexOutOfBoundsException"), cstr!("(Ljava/lang/String;)V")); 

    

    let glinitstaticmethods = [
        native_method!("initGL", "(IILcom/github/wartman4404/gldraw/UndoCallback;)I", init_gl),
        native_method!("destroy", "(I)V", finish_gl),
    ];
    register_classmethods(env, cstr!("com/github/wartman4404/gldraw/GLInit$"), &glinitstaticmethods);

    let texturemethods = [
        native_method!("nativeUpdateGL", "(I)V", native_update_gl),
        native_method!("nativeDrawQueuedPoints", "(II[F)V", native_draw_queued_points),
        native_method!("nativeFinishLuaScript", "(II)V", native_finish_lua_script),
        native_method!("nativeClearFramebuffer", "(I)V", clear_framebuffer),
        native_method!("nativeDrawImage", "(ILandroid/graphics/Bitmap;I)V", draw_image),
        native_method!("nativeSetAnimShader", "(II)Z", set_anim_shader),
        native_method!("nativeSetCopyShader", "(II)Z", set_copy_shader),
        native_method!("nativeSetPointShader", "(II)Z", set_point_shader),
        native_method!("nativeSetBrushTexture", "(II)V", set_brush_texture),
        native_method!("nativeExportPixels", "(I)Landroid/graphics/Bitmap;", export_pixels),
        native_method!("nativeSetInterpolator", "(II)V", jni_lua_set_interpolator),
        native_method!("nativeAddLayer", "(IIII)V", jni_add_layer),
        native_method!("nativeClearLayers", "(I)V", jni_clear_layers),
        native_method!("nativeLoadUndo", "(II)V", jni_load_undo),
        native_method!("nativePushUndoFrame", "(I)I", jni_push_undo_frame),
        native_method!("nativeClearUndoFrames", "(I)V", jni_clear_undo_frames),
        native_method!("nativeSetBrushColor", "(II)V", jni_set_brush_color),
        native_method!("nativeSetBrushSize", "(IF)V", jni_set_brush_size),
    ];
    register_classmethods(env, cstr!("com/github/wartman4404/gldraw/TextureSurfaceThread"), &texturemethods);
    debug_logi!("registered texture thread methods!");

    let replayhandlerstaticmethods = [
        native_method!("init", "(I)I", jni_replay_begin),
        native_method!("destroy", "(I)V", jni_replay_destroy),
        native_method!("advanceFrame", "(II[F)Z", jni_replay_advance_frame),
    ];
    register_classmethods(env, cstr!("com/github/wartman4404/gldraw/Replay$"), &replayhandlerstaticmethods);
    debug_logi!("registered replay methods!");
}

pub unsafe fn destroy(env: *mut JNIEnv) {
    LUA_EXCEPTION.destroy(env);
    RUNTIME_EXCEPTION.destroy(env);
}
