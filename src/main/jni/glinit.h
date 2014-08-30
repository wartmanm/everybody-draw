// TODO: clean up
#include <android/input.h>
#include <android/bitmap.h>
#include <android/native_window_jni.h>
#include "lua/lua.h"

typedef void* GLInit;
typedef void* MotionEventConsumer;
typedef void* MotionEventProducer;
struct MotionEventHandlerPair {
  MotionEventConsumer consumer;
  MotionEventProducer producer;
};

GLInit setup_graphics(int w, int h);
void render_frame(GLInit data);
void deinit_gl(GLInit data);
struct MotionEventHandlerPair create_motion_event_handler();
void destroy_motion_event_handler(MotionEventConsumer consumer, MotionEventProducer producer);

void jni_append_motion_event(MotionEventProducer h, AInputEvent* evt);
void draw_queued_points(GLInit data, MotionEventConsumer handler, float *matrix);

void set_anim_shader(GLInit data, int shader);
void set_copy_shader(GLInit data, int shader);
void set_point_shader(GLInit data, int shader);
void set_interpolator(GLInit data, int script);

void set_brush_texture(GLInit data, int texture);
int load_texture(GLInit data, int width, int height, const char *pixels, int format);

void clear_buffer(GLInit data);

int compile_point_shader(GLInit data, const char *vec, const char *frag);
int compile_copy_shader(GLInit data, const char *vec, const char *frag);
int compile_luascript(GLInit data, const char *script);

void draw_image(GLInit data, int w, int h, const char *pixels);

void set_separate_brushlayer(GLInit data, char separate_layer);

typedef void* (*pixel_callback)(int, int, const char*, void *env);
void* with_pixels(GLInit data, pixel_callback callback, void *env);

void egl_init(void *window);
void egl_swap();
void egl_finish();

lua_State *initLua();
int loadLuaScript(const char *script);
void useLuaScript(int key);
