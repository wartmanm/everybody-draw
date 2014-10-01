// HEEERE'S JNI!
// I'm so sorry.  I couldn't help myself.
//
// TODO: fix signatures on e.g. nativeSetAnimShader

#include <stdio.h>
#include <string.h>
#include <jni.h>
#include "glinit.h"

#include <android/log.h>

#include <stdlib.h>

#define  LOG_TAG    "mygl"
#define  LOGI(...)  __android_log_print(ANDROID_LOG_INFO,LOG_TAG,__VA_ARGS__)
#define  LOGE(...)  __android_log_print(ANDROID_LOG_ERROR,LOG_TAG,__VA_ARGS__)

// Fields for java classes to pass around pointers typesafely
struct JavaPointerClass {
  jclass class;
  jfieldID nativePtrField;
  jmethodID constructor;
};

static jclass motionclass;
static jfieldID motionEvent_nativePtrField;

static int initGL( JNIEnv* env, jobject thiz, jint w, jint h )
{
  LOGI("trying init! got args %d, %d\n", w, h);
  GLInit graphics = setup_graphics(w, h);
  LOGI("all good!\n");
  return (int) graphics;
}

static void nativeDrawQueuedPoints( JNIEnv* env, jobject thiz, int data, int handler, jfloatArray javaMatrix)
{
  float matrix[16];
  (*env)->GetFloatArrayRegion(env, javaMatrix, 0, 16, matrix);
  draw_queued_points((GLInit)data, (MotionEventConsumer) handler, matrix);
}

static void finishGL( JNIEnv* env, jobject thiz, int data)
{
  deinit_gl((GLInit) data);
  LOGI("finish good!\n");
}

static void nativeUpdateGL( JNIEnv* env, jobject thiz, int data)
{
  render_frame((GLInit) data);
}

static jobject initMotionEventHandler(JNIEnv *env, jobject thiz) {
  struct MotionEventHandlerPair pairdata = create_motion_event_handler();
  jclass pairclass = (*env)->FindClass(env, "com/github/wartman4404/gldraw/MotionEventHandlerPair");
  jmethodID constructor = (*env)->GetMethodID(env, pairclass, "<init>", "(II)V");
  jobject pairobj = (*env)->NewObject(env, pairclass, constructor, (int)pairdata.consumer, (int)pairdata.producer);
  return pairobj;
}

static void destroyMotionEventHandler(JNIEnv *env, jobject thiz, jobject pairobj) {
  jclass pairclass = (*env)->FindClass(env, "com/github/wartman4404/gldraw/MotionEventHandlerPair");
  jfieldID consumerid = (*env)->GetFieldID(env, pairclass, "consumer", "I");
  jfieldID producerid = (*env)->GetFieldID(env, pairclass, "producer", "I");
  MotionEventConsumer consumer = (MotionEventConsumer) (*env)->GetIntField(env, pairobj, consumerid);
  MotionEventProducer producer = (MotionEventProducer) (*env)->GetIntField(env, pairobj, producerid);
  destroy_motion_event_handler(consumer, producer);
}

static void nativeAppendMotionEvent(JNIEnv* env, jobject thiz, int handler, jobject evtobj) {
  AInputEvent *evtptr = (AInputEvent*) (*env)->GetIntField(env, evtobj, motionEvent_nativePtrField);
  jni_append_motion_event((MotionEventConsumer)handler, evtptr);
}

static int shaderStrObjects(JNIEnv* env, GLInit data, jstring vec, jstring frag, int (*callback)(GLInit data, const char* vec, const char* frag)) {
  const char* vecstr = vec == NULL ? NULL : (*env)->GetStringUTFChars(env, vec, NULL);
  const char* fragstr = frag == NULL ? NULL : (*env)->GetStringUTFChars(env, frag, NULL);
  int ret = callback(data, vecstr, fragstr);
  if (vecstr != NULL) (*env)->ReleaseStringUTFChars(env, vec, vecstr);
  if (fragstr != NULL) (*env)->ReleaseStringUTFChars(env, frag, fragstr);
  return ret;
}

static void setAnimShader(JNIEnv* env, jobject thiz, GLInit data, jint shader) {
  set_anim_shader(data, shader);
}
static void setCopyShader(JNIEnv* env, jobject thiz, GLInit data, jint shader) {
  set_copy_shader(data, shader);
}
static void setPointShader(JNIEnv* env, jobject thiz, GLInit data, jint shader) {
  set_point_shader(data, shader);
}

static void setBrushTexture(JNIEnv* env, jobject thiz, GLInit data, jint texture) {
  set_brush_texture(data, texture);
}

static jint createTexture(JNIEnv* env, jobject thiz, int data, jobject bitmap) {
  // TODO: ensure rgba_8888 format and throw error
  // TODO: or alpha8?
  AndroidBitmapInfo info;
  AndroidBitmap_getInfo(env, bitmap, &info);
  void *pixels;
  AndroidBitmap_lockPixels(env, bitmap, &pixels);
  int texture = load_texture((GLInit) data, info.width, info.height, pixels, info.format);
  AndroidBitmap_unlockPixels(env, bitmap);
  return texture;
}

static void clearFramebuffer(JNIEnv* env, jobject thiz, int data) {
  clear_buffer((GLInit) data);
}

static jint compileCopyShader(JNIEnv* env, jobject thiz, int data, jstring vec, jstring frag) {
  int shader = shaderStrObjects(env, (GLInit)data, vec, frag, compile_copy_shader);
  return shader;
}

static jint compilePointShader(JNIEnv* env, jobject thiz, int data, jstring vec, jstring frag) {
  int shader = shaderStrObjects(env, (GLInit)data, vec, frag, compile_point_shader);
  return shader;
}

static void drawImage(JNIEnv* env, jobject thiz, int data, jobject bitmap) {
  // TODO: ensure rgba_8888 format and throw error
  AndroidBitmapInfo info;
  AndroidBitmap_getInfo(env, bitmap, &info);
  void *pixels;
  AndroidBitmap_lockPixels(env, bitmap, &pixels);
  draw_image((GLInit) data, info.width, info.height, pixels);
  AndroidBitmap_unlockPixels(env, bitmap);
}

void* mycallback(int x, int y, const char *pixels, void *env_void) {
  LOGI("in callback!");
  JNIEnv *env = (JNIEnv*) env_void;
  jclass bitmapclass = (*env)->FindClass(env, "android/graphics/Bitmap");
  jclass configclass = (*env)->FindClass(env, "android/graphics/Bitmap$Config");
  jfieldID argbfield = (*env)->GetStaticFieldID(env, configclass, "ARGB_8888", "Landroid/graphics/Bitmap$Config;");
  jobject argb = (*env)->GetStaticObjectField(env, configclass, argbfield);
  jmethodID createbitmap = (*env)->GetStaticMethodID(env, bitmapclass, "createBitmap", "(IILandroid/graphics/Bitmap$Config;)Landroid/graphics/Bitmap;");
  jobject bitmap = (*env)->CallStaticObjectMethod(env, bitmapclass, createbitmap, x, y, argb);
  LOGI("created bitmap");
  void *outpixels;
  AndroidBitmap_lockPixels(env, bitmap, &outpixels);
  LOGI("locked pixels");
  memcpy(outpixels, pixels, x*y*4);
  LOGI("copied pixels");
  AndroidBitmap_unlockPixels(env, bitmap);
  LOGI("unlocked pixels");
  jmethodID premult = (*env)->GetMethodID(env, bitmapclass, "setPremultiplied", "(Z)V");
  (*env)->CallVoidMethod(env, bitmap, premult, JNI_TRUE);
  LOGI("done with callback");
  return bitmap;
}

//may return null!
//TODO: store bitmap class data
static jobject exportPixels(JNIEnv* env, jobject thiz, int data) {
  return (jobject) with_pixels((GLInit)data, mycallback, env);
}

/*static void jniSetSeparateBrushlayer(JNIEnv* env, jobject thiz, int data, jboolean separatelayer) {*/
  /*set_separate_brushlayer((GLInit)data, separatelayer);*/
/*}*/

static void jniEglFinish(JNIEnv* env, jobject thiz) {
  egl_finish();
}

static void jniEglInit(JNIEnv* env, jobject thiz, jobject surface) {
  ANativeWindow* window = ANativeWindow_fromSurface(env, surface);
  LOGI("got ANativeWindow: 0x%x", (int) window);
  egl_init(window);
  ANativeWindow_release(window);
}

static void jniLuaInit(JNIEnv* env, jobject thiz) {
}
static void jniLuaFinish(JNIEnv* env, jobject thiz) {
}
static int jniLuaCompileScript(JNIEnv* env, jobject thiz, int data, jstring script) {
  const char* scriptchars = script == NULL ? NULL : (*env)->GetStringUTFChars(env, script, NULL);
  int scriptid = compile_luascript((GLInit) data, scriptchars);
  if (scriptchars != NULL) (*env)->ReleaseStringUTFChars(env, script, scriptchars);
  return scriptid;
}

static void jniLuaSetInterpolator(JNIEnv* env, jobject thiz, int data, jint scriptid) {
  set_interpolator((GLInit) data, scriptid);
}

static void jniAddLayer(JNIEnv* env, jobject thiz, int data, int copyshader, int pointshader, int pointidx) {
  add_layer((GLInit) data, copyshader, pointshader, pointidx);
}

static void jniClearLayers(JNIEnv* env, jobject thiz, int data) {
  clear_layers((GLInit) data);
}

jint JNI_OnLoad(JavaVM* vm, void* reserved) {
  JNIEnv *env;
  if ((*vm)->GetEnv(vm, (void*)&env, JNI_VERSION_1_6) != JNI_OK) {
    return -1;
  }
  motionclass = (*env)->FindClass(env, "android/view/MotionEvent");
  motionEvent_nativePtrField = (*env)->GetFieldID(env, motionclass, "mNativePtr", "I");

  JNINativeMethod mainmethods[] = {
    { 
      .name = "nativeAppendMotionEvent",
      .signature = "(ILandroid/view/MotionEvent;)V",
      .fnPtr = nativeAppendMotionEvent,
    },
  };
  jclass mainactivityclass = (*env)->FindClass(env, "com/github/wartman4404/gldraw/MainActivity");
  (*env)->RegisterNatives(env, mainactivityclass, mainmethods, sizeof(mainmethods)/sizeof(JNINativeMethod));
  JNINativeMethod texturemethods[] = {
    {
      .name = "nativeUpdateGL",
      .signature = "(I)V",
      .fnPtr = nativeUpdateGL,
    }, {
      .name = "nativeDrawQueuedPoints",
      .signature = "(II[F)V",
      .fnPtr = nativeDrawQueuedPoints,
    }, {
      .name = "nativeClearFramebuffer",
      .signature = "(I)V",
      .fnPtr = clearFramebuffer,
    }, {
      .name = "drawImage",
      .signature = "(ILandroid/graphics/Bitmap;)V",
      .fnPtr = drawImage,
    }, {
      .name = "nativeSetAnimShader",
      .signature = "(II)Z",
      .fnPtr = setAnimShader,
    }, {
      .name = "nativeSetCopyShader",
      .signature = "(II)Z",
      .fnPtr = setCopyShader,
    }, {
      .name = "nativeSetPointShader",
      .signature = "(II)Z",
      .fnPtr = setPointShader,
    }, {
      .name = "nativeSetBrushTexture",
      .signature = "(II)V",
      .fnPtr = setBrushTexture,
    }, {
      .name = "exportPixels",
      .signature = "(I)Landroid/graphics/Bitmap;",
      .fnPtr = exportPixels,
    }, {
      .name = "nativeSetInterpolator",
      .signature = "(II)V",
      .fnPtr = jniLuaSetInterpolator,
    }, {
      .name = "nativeAddLayer",
      .signature = "(IIII)V",
      .fnPtr = jniAddLayer,
    }, {
      .name = "nativeClearLayers",
      .signature = "(II)V",
      .fnPtr = jniClearLayers,
    }
  };
  jclass textureclass = (*env)->FindClass(env, "com/github/wartman4404/gldraw/TextureSurfaceThread");
  (*env)->RegisterNatives(env, textureclass, texturemethods, sizeof(texturemethods)/sizeof(JNINativeMethod));
  JNINativeMethod pointshaderstaticmethods[] = {
    {
      .name = "compile",
      .signature = "(ILjava/lang/String;Ljava/lang/String;)I",
      .fnPtr = compilePointShader,
    },
  };
  JNINativeMethod copyshaderstaticmethods[] = {
    {
      .name = "compile",
      .signature = "(ILjava/lang/String;Ljava/lang/String;)I",
      .fnPtr = compileCopyShader,
    },
  };
  JNINativeMethod texturestaticmethods[] = {
    {
      .name = "init",
      .signature = "(ILandroid/graphics/Bitmap;)I",
      .fnPtr = createTexture,
    },
  };

  jclass copyshaderstatic = (*env)->FindClass(env, "com/github/wartman4404/gldraw/CopyShader$");
  jclass pointshaderstatic = (*env)->FindClass(env, "com/github/wartman4404/gldraw/PointShader$");
  jclass texturestatic = (*env)->FindClass(env, "com/github/wartman4404/gldraw/Texture$");
  (*env)->RegisterNatives(env, copyshaderstatic, copyshaderstaticmethods, sizeof(copyshaderstaticmethods)/sizeof(JNINativeMethod));
  (*env)->RegisterNatives(env, pointshaderstatic, pointshaderstaticmethods, sizeof(pointshaderstaticmethods)/sizeof(JNINativeMethod));
  (*env)->RegisterNatives(env, texturestatic, texturestaticmethods, sizeof(texturestaticmethods)/sizeof(JNINativeMethod));

  JNINativeMethod eglhelpermethods[] = {
    {
      .name = "nativeFinish",
      .signature = "()V",
      .fnPtr = jniEglFinish,
    }, {
      .name = "nativeInit",
      .signature = "(Landroid/view/Surface;)V",
      .fnPtr = jniEglInit,
    }
  };
  jclass eglhelper = (*env)->FindClass(env, "com/github/wartman4404/gldraw/EGLHelper");
  (*env)->RegisterNatives(env, eglhelper, eglhelpermethods, sizeof(eglhelpermethods)/sizeof(JNINativeMethod));

  JNINativeMethod luastaticmethods[] = {
    {
      .name = "init",
      .signature = "(ILjava/lang/String;)I",
      .fnPtr = jniLuaCompileScript,
    }
  };
  jclass luastatic = (*env)->FindClass(env, "com/github/wartman4404/gldraw/LuaScript$");
  (*env)->RegisterNatives(env, luastatic, luastaticmethods, sizeof(luastaticmethods)/sizeof(JNINativeMethod));

  JNINativeMethod glinitstaticmethods[] = {
    {
      .name = "initGL",
      .signature = "(II)I",
      .fnPtr = initGL,
    }, {
      .name = "destroy",
      .signature = "(I)V",
      .fnPtr = finishGL,
    }
  };
  jclass glinitstatic = (*env)->FindClass(env, "com/github/wartman4404/gldraw/GLInit$");
  (*env)->RegisterNatives(env, glinitstatic, glinitstaticmethods, sizeof(glinitstaticmethods)/sizeof(JNINativeMethod));

  JNINativeMethod motioneventhandlerstaticmethods[] = {
    {
      .name = "init",
      .signature = "()Lcom/github/wartman4404/gldraw/MotionEventHandlerPair;",
      .fnPtr = initMotionEventHandler,
    }, {
      .name = "destroy",
      .signature = "(Lcom/github/wartman4404/gldraw/MotionEventHandlerPair;)V",
      .fnPtr = destroyMotionEventHandler,
    }
  };
  jclass motioneventhandlerpairstatic = (*env)->FindClass(env, "com/github/wartman4404/gldraw/MotionEventHandlerPair$");
  (*env)->RegisterNatives(env, motioneventhandlerpairstatic, motioneventhandlerstaticmethods, sizeof(motioneventhandlerstaticmethods)/sizeof(JNINativeMethod));

  return JNI_VERSION_1_2;
}
