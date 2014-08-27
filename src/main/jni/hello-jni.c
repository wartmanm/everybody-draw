// HEEERE'S JNI!
// I'm so sorry.  I couldn't help myself.
//
// TODO: fix signatures on e.g. nativeSetAnimShader

/*
 * Copyright (C) 2009 The Android Open Source Project
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */
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

/* This is a trivial JNI example where we use a native method
 * to return a new VM String. See the corresponding Java source
 * file located at:
 *
 *   apps/samples/hello-jni/project/src/com/example/hellojni/HelloJni.java
 */;
static void initGL( JNIEnv* env, jobject thiz, jint w, jint h )
{
  LOGI("trying init! got args %d, %d\n", w, h);
  setup_graphics(w, h);
  LOGI("all good!\n");
}

static void nativeDrawQueuedPoints( JNIEnv* env, jobject thiz, jfloatArray javaMatrix)
{
  float matrix[16];
  (*env)->GetFloatArrayRegion(env, javaMatrix, 0, 16, matrix);
  draw_queued_points(matrix);
}

static void finishGL( JNIEnv* env, jobject thiz )
{
  deinit_gl();
  LOGI("finish good!\n");
}

static void nativeUpdateGL( JNIEnv* env, jobject thiz)
{
  render_frame();
}

static void nativeAppendMotionEvent(JNIEnv* env, jobject thiz, jobject evtobj) {
  AInputEvent *evtptr = (AInputEvent*) (*env)->GetIntField(env, evtobj, motionEvent_nativePtrField);
  jni_append_motion_event(evtptr);
}

static int shaderStrObjects(JNIEnv* env, jstring vec, jstring frag, int callback(const char* vec, const char* frag)) {
  const char* vecstr = vec == NULL ? NULL : (*env)->GetStringUTFChars(env, vec, NULL);
  const char* fragstr = frag == NULL ? NULL : (*env)->GetStringUTFChars(env, frag, NULL);
  int ret = callback(vecstr, fragstr);
  if (vecstr != NULL) (*env)->ReleaseStringUTFChars(env, vec, vecstr);
  if (fragstr != NULL) (*env)->ReleaseStringUTFChars(env, frag, fragstr);
  return ret;
}

static void setAnimShader(JNIEnv* env, jobject thiz, jint shader) {
  set_anim_shader(shader);
}
static void setCopyShader(JNIEnv* env, jobject thiz, jint shader) {
  set_copy_shader(shader);
}
static void setPointShader(JNIEnv* env, jobject thiz, jint shader) {
  set_point_shader(shader);
}

static void setBrushTexture(JNIEnv* env, jobject thiz, jint texture) {
  set_brush_texture(texture);
}

static jint createTexture(JNIEnv* env, jobject thiz, jobject bitmap) {
  // TODO: ensure rgba_8888 format and throw error
  // TODO: or alpha8?
  AndroidBitmapInfo info;
  AndroidBitmap_getInfo(env, bitmap, &info);
  void *pixels;
  AndroidBitmap_lockPixels(env, bitmap, &pixels);
  int texture = load_texture(info.width, info.height, pixels, info.format);
  AndroidBitmap_unlockPixels(env, bitmap);
  return texture;
}

static void clearFramebuffer(JNIEnv* env, jobject thiz) {
  clear_buffer();
}

static jint compileCopyShader(JNIEnv* env, jobject thiz, jstring vec, jstring frag) {
  int shader = shaderStrObjects(env, vec, frag, compile_copy_shader);
  return shader;
}

static jint compilePointShader(JNIEnv* env, jobject thiz, jstring vec, jstring frag) {
  int shader = shaderStrObjects(env, vec, frag, compile_point_shader);
  return shader;
}

static void drawImage(JNIEnv* env, jobject thiz, jobject bitmap) {
  // TODO: ensure rgba_8888 format and throw error
  AndroidBitmapInfo info;
  AndroidBitmap_getInfo(env, bitmap, &info);
  void *pixels;
  AndroidBitmap_lockPixels(env, bitmap, &pixels);
  draw_image(info.width, info.height, pixels);
  AndroidBitmap_unlockPixels(env, bitmap);
}

//TODO: store bitmap class data
static jobject exportPixels(JNIEnv* env, jobject thiz) {
    LOGI("in callback");
    /*jbyteArray array = (*env)->NewByteArray(env, len);*/
    /*(*env)->SetByteArrayRegion(env, array, 0, len, (jbyte const*)pixels);*/
    /*return (void*)array;*/
    struct withpixels_tuple pxls = with_pixels();
    const char *pixels = pxls.pixels;
    int x = pxls.x, y = pxls.y;
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
    release_pixels(pxls.pixels);
    LOGI("released pixels");
    jmethodID premult = (*env)->GetMethodID(env, bitmapclass, "setPremultiplied", "(Z)V");
    (*env)->CallVoidMethod(env, bitmap, premult, JNI_TRUE);
    return bitmap;
}

static void jniEglSwap(JNIEnv* env, jobject thiz) {
  egl_swap();
}

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
static int jniLuaCompileScript(JNIEnv* env, jobject thiz, jstring script) {
  const char* scriptchars = script == NULL ? NULL : (*env)->GetStringUTFChars(env, script, NULL);
  int scriptid = compile_luascript(scriptchars);
  if (scriptchars != NULL) (*env)->ReleaseStringUTFChars(env, script, scriptchars);
  return scriptid;
}

static void jniLuaSetInterpolator(JNIEnv* env, jobject thiz, jint scriptid) {
  set_interpolator(scriptid);
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
      .signature = "(Landroid/view/MotionEvent;)V",
      .fnPtr = nativeAppendMotionEvent,
    },
  };
  jclass mainactivityclass = (*env)->FindClass(env, "com/github/wartman4404/gldraw/MainActivity");
  (*env)->RegisterNatives(env, mainactivityclass, mainmethods, sizeof(mainmethods)/sizeof(JNINativeMethod));
  JNINativeMethod texturemethods[] = {
    {
      .name = "initGL",
      .signature = "(II)V",
      .fnPtr = initGL,
    }, {
      .name = "finishGL",
      .signature = "()V",
      .fnPtr = finishGL,
    }, {
      .name = "nativeUpdateGL",
      .signature = "()V",
      .fnPtr = nativeUpdateGL,
    }, {
      .name = "nativeDrawQueuedPoints",
      .signature = "([F)V",
      .fnPtr = nativeDrawQueuedPoints,
    }, {
      .name = "nativeClearFramebuffer",
      .signature = "()V",
      .fnPtr = clearFramebuffer,
    }, {
      .name = "drawImage",
      .signature = "(Landroid/graphics/Bitmap;)V",
      .fnPtr = drawImage,
    }, {
      .name = "nativeSetAnimShader",
      .signature = "(I)Z",
      .fnPtr = setAnimShader,
    }, {
      .name = "nativeSetCopyShader",
      .signature = "(I)Z",
      .fnPtr = setCopyShader,
    }, {
      .name = "nativeSetPointShader",
      .signature = "(I)Z",
      .fnPtr = setPointShader,
    }, {
      .name = "nativeSetBrushTexture",
      .signature = "(I)V",
      .fnPtr = setBrushTexture,
    }, {
      .name = "exportPixels",
      .signature = "()Landroid/graphics/Bitmap;",
      .fnPtr = exportPixels,
    }, {
      .name = "nativeSetInterpolator",
      .signature = "(I)V",
      .fnPtr = jniLuaSetInterpolator,
    }
  };
  jclass textureclass = (*env)->FindClass(env, "com/github/wartman4404/gldraw/TextureSurfaceThread");
  (*env)->RegisterNatives(env, textureclass, texturemethods, sizeof(texturemethods)/sizeof(JNINativeMethod));
  JNINativeMethod pointshaderstaticmethods[] = {
    {
      .name = "compile",
      .signature = "(Ljava/lang/String;Ljava/lang/String;)I",
      .fnPtr = compilePointShader,
    },
  };
  JNINativeMethod copyshaderstaticmethods[] = {
    {
      .name = "compile",
      .signature = "(Ljava/lang/String;Ljava/lang/String;)I",
      .fnPtr = compileCopyShader,
    },
  };
  JNINativeMethod texturestaticmethods[] = {
    {
      .name = "init",
      .signature = "(Landroid/graphics/Bitmap;)I",
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
      .name = "swap",
      .signature = "()V",
      .fnPtr = jniEglSwap,
    }, {
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
      .signature = "(Ljava/lang/String;)I",
      .fnPtr = jniLuaCompileScript,
    }
  };
  jclass luastatic = (*env)->FindClass(env, "com/github/wartman4404/gldraw/LuaScript$");
  (*env)->RegisterNatives(env, luastatic, luastaticmethods, sizeof(luastaticmethods)/sizeof(JNINativeMethod));

  /*jclass luahelper = (*env)->FindClass(env, "com/github/wartman4404/gldraw/LuaHelper$");*/
  /*(*env)->RegisterNatives(env, luahelper, luahelpermethods, sizeof(luahelpermethods)/sizeof(JNINativeMethod));*/

  return JNI_VERSION_1_2;
}
