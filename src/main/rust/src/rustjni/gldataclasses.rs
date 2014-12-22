use core::prelude::*;
use core::mem;
use libc::c_void;

use jni::{jobject, jclass, jmethodID, JNIEnv, jint, jstring, JNINativeMethod};
use android::native_window_jni::{ANativeWindow_fromSurface};//, ANativeWindow_release};
use android::native_window::ANativeWindow_release;

use glstore::DrawObjectIndex;
use glinit::AndroidBitmapFormat;
use eglinit;
use jni_helpers::ToJValue;
use gltexture::{ToPixelFormat, BrushTexture};
use glcommon::{GLResult, MString};
use rustjni::{register_classmethods, CaseClass, get_safe_data, str_to_jstring, get_mstring, jpointer};
use rustjni::android_bitmap::AndroidBitmap;

static mut SCALA_TUPLE2: CaseClass = CaseClass { constructor: 0 as jmethodID, class: 0 as jclass };
static mut GL_EXCEPTION: CaseClass = CaseClass { constructor: 0 as jmethodID, class: 0 as jclass };

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

unsafe fn safe_create_texture(env: *mut JNIEnv, data: jpointer, bitmap: jobject) -> GLResult<DrawObjectIndex<BrushTexture>> {
    let bitmap = AndroidBitmap::from_jobject(env, bitmap);
    let (w, h) = (bitmap.info.width, bitmap.info.height);
    let format: AndroidBitmapFormat = mem::transmute(bitmap.info.format);
    let texformat = try!(format.to_pixelformat());
    Ok(get_safe_data(data).events.load_brush(w as i32, h as i32, bitmap.as_slice(), texformat))
}

unsafe extern "C" fn compile_copyshader(env: *mut JNIEnv, _: jobject, data: jpointer, vec: jstring, frag: jstring) -> jint {
    glresult_or_exception(env, get_safe_data(data).events.load_copyshader(get_mstring(env, vec), get_mstring(env, frag)))
}

unsafe extern "C" fn compile_pointshader(env: *mut JNIEnv, _: jobject, data: jpointer, vec: jstring, frag: jstring) -> jint {
    glresult_or_exception(env, get_safe_data(data).events.load_pointshader(get_mstring(env, vec), get_mstring(env, frag)))
}

unsafe extern "C" fn jni_lua_compile_script(env: *mut JNIEnv, _: jobject, data: jpointer, script: jstring) -> jint {
    let scriptstr = get_mstring(env, script);
    let data = get_safe_data(data);
    // FIXME: this will retrieve a script from cache if there is one already
    // this may not be whatwe want, since the old one may have state already
    glresult_or_exception(env, data.events.load_interpolator(scriptstr))
}

unsafe extern "C" fn create_texture(env: *mut JNIEnv, _: jobject, data: jpointer, bitmap: jobject) -> jint {
    glresult_or_exception(env, safe_create_texture(env, data, bitmap))
}

unsafe fn get_shader_source_tuple(env: *mut JNIEnv, source: &(MString, MString)) -> jobject {
    let &(ref vert, ref frag) = source;
    let mut jvert = str_to_jstring(env, vert.as_slice());
    let mut jfrag = str_to_jstring(env, frag.as_slice());
    SCALA_TUPLE2.construct(env, [jvert.as_jvalue(), jfrag.as_jvalue()].as_mut_slice())
}

pub unsafe extern "C" fn jni_get_copyshader_source(env: *mut JNIEnv, _: jobject, data: jpointer, copyshader: jint) -> jobject {
    logi!("getting copyshader source");
    let source = get_safe_data(data).events.get_copyshader_source(mem::transmute(copyshader));
    logi!("got copyshader source");
    let tuple = get_shader_source_tuple(env, source);
    logi!("created tuple");
    tuple
}

unsafe extern "C" fn jni_get_pointshader_source(env: *mut JNIEnv, _: jobject, data: jpointer, pointshader: jint) -> jobject {
    logi!("getting pointshader source");
    let source = get_safe_data(data).events.get_pointshader_source(mem::transmute(pointshader));
    get_shader_source_tuple(env, source)
}

unsafe extern "C" fn jni_get_luascript_source(env: *mut JNIEnv, _: jobject, data: jpointer, luascript: jint) -> jstring {
    logi!("getting luascript source");
    let source = get_safe_data(data).events.get_luascript_source(mem::transmute(luascript));
    str_to_jstring(env, source.as_slice())
}

unsafe extern "C" fn jni_egl_init(env: *mut JNIEnv, _: jobject, surface: jobject) {
    let window = ANativeWindow_fromSurface(env, surface);
    logi!("got ANAtiveWindow: 0x{:x}", window as u32);
    eglinit::egl_init(window as *mut c_void);
    ANativeWindow_release(window);
}

unsafe extern "C" fn jni_egl_finish(_: *mut JNIEnv, _: jobject) {
    eglinit::egl_finish();
}

pub unsafe fn init(env: *mut JNIEnv) {
    SCALA_TUPLE2 = CaseClass::new(env, cstr!("scala/Tuple2"), cstr!("(Ljava/lang/Object;Ljava/lang/Object;)V"));
    GL_EXCEPTION = CaseClass::new(env, cstr!("com/github/wartman4404/gldraw/GLException"), cstr!("(Ljava/lang/String;)V"));

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
    let luastaticmethods = [
        native_method!("init", "(ILjava/lang/String;)I", jni_lua_compile_script),
        native_method!("getSource", "(II)Ljava/lang/String;", jni_get_luascript_source),
    ];
    let eglhelpermethods = [
        native_method!("nativeFinish", "()V", jni_egl_finish),
        native_method!("nativeInit", "(Landroid/view/Surface;)V", jni_egl_init),
    ];
    register_classmethods(env, cstr!("com/github/wartman4404/gldraw/PointShader$"), &pointshaderstaticmethods);
    register_classmethods(env, cstr!("com/github/wartman4404/gldraw/CopyShader$"), &copyshaderstaticmethods);
    register_classmethods(env, cstr!("com/github/wartman4404/gldraw/TexturePtr$"), &texturestaticmethods);
    register_classmethods(env, cstr!("com/github/wartman4404/gldraw/EGLHelper"), &eglhelpermethods);
    register_classmethods(env, cstr!("com/github/wartman4404/gldraw/LuaScript$"), &luastaticmethods);
    logi!("registered point|copy|texture|lua|egl static methods!");
}

pub unsafe fn destroy(env: *mut JNIEnv) {
    SCALA_TUPLE2.destroy(env);
    GL_EXCEPTION.destroy(env);
}
