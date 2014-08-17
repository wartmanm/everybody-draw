#include <jni.h>
#include <android/log.h>
#include <stdlib.h>
#include "eglinit.h"

#define  LOG_TAG    "myegl"
#define  LOGI(...)  __android_log_print(ANDROID_LOG_INFO,LOG_TAG,__VA_ARGS__)
#define  LOGE(...)  __android_log_print(ANDROID_LOG_ERROR,LOG_TAG,__VA_ARGS__)

EGLint* default_egl_config() {
  static EGLint config_attribs[] = {
    EGL_RENDERABLE_TYPE, EGL_OPENGL_ES2_BIT,
    EGL_RED_SIZE, 8,
    EGL_GREEN_SIZE, 8,
    EGL_BLUE_SIZE, 8,
    EGL_ALPHA_SIZE, 8,
    EGL_DEPTH_SIZE, 0,
    EGL_STENCIL_SIZE, 0,
    EGL_NONE
  };
  return config_attribs;
}

EGLConfig chooseEglConfig(EGLDisplay ed) {
  int configCount;
  EGLConfig config;
  if (EGL_TRUE != eglChooseConfig(ed, default_egl_config(), &config, 1, &configCount)) {
    LOGE("eglChooseConfig returned false :(");
    return 0;
  } else if (configCount == 0) {
    LOGE("no matching configs found :(");
    return 0;
  }
  return config;
}

EGLint* default_context_attribs() {
  static EGLint context_attribs[] = {
    EGL_CONTEXT_CLIENT_VERSION, 2,
    EGL_NONE
  };
  return context_attribs;
}

EGLContext createContext(EGLDisplay ed, EGLConfig config) {
  return eglCreateContext(ed, config, EGL_NO_CONTEXT, default_context_attribs());
}

EGLSurface surface;
EGLDisplay display;
EGLContext context;

int initContext(jobject surfaceTexture) {
  EGLDisplay ed = eglGetDisplay(EGL_DEFAULT_DISPLAY);
  if (ed == EGL_NO_DISPLAY) {
    LOGE("failed to get display :(\n");
    return 0;
  }
  int vermajor, verminor;
  if (EGL_TRUE != eglInitialize(ed, &vermajor, &verminor)) {
    LOGE("failed to initialize display :(\n");
    return 0;
  }
  LOGE("egl %d.%d\n", vermajor, verminor);
  LOGE("extensions: %s\n", eglQueryString(ed, EGL_EXTENSIONS));
  LOGE("vendor: %s\n", eglQueryString(ed, EGL_VENDOR));

  EGLConfig config = chooseEglConfig(ed);
  LOGI("got config: 0x%x", (unsigned int) config);
  EGLContext ctxt = createContext(ed, config);
  LOGI("got context: 0x%x", (unsigned int) config);
  EGLSurface surf = eglCreateWindowSurface(ed, config, surfaceTexture, NULL);
  LOGI("got surface: 0x%x", (unsigned int) config);
  surface = surf;
  display = ed;
  context = ctxt;
  return 1;
}

int eglUpdate() {
  // no-op for offscreen pixmaps, must use eglCopyBuffers instead
  if (EGL_TRUE != eglSwapBuffers(display, surface)) {
    LOGE("failed to swap buffers??");
    return 0;
  }
  return 1;
}

int destroyContext() {
  eglDestroyContext(display, context);
  if (EGL_TRUE != eglTerminate(EGL_DEFAULT_DISPLAY)) {
    LOGE("failed to destroy context :(\n");
    // eglmakecurrent, eglreleasethread?
    return 0;
  }
  return 1;
}
