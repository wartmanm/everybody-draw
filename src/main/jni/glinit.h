// TODO: clean up
#include <android/input.h>
#include <android/bitmap.h>
#include <android/native_window_jni.h>
#include "lua/lua.h"

int setup_graphics(int w, int h);
void render_frame();
void deinit_gl();
void jni_append_motion_event(AInputEvent* evt);
void draw_queued_points(float *matrix);

void set_anim_shader(int shader);
void set_copy_shader(int shader);
void set_point_shader(int shader);
void set_interpolator(int script);

void set_brush_texture(int texture);
int load_texture(int width, int height, const char *pixels, int format);

void clear_buffer();

int compile_point_shader(const char *vec, const char *frag);
int compile_copy_shader(const char *vec, const char *frag);
int compile_luascript(const char *script);

void deinit_copy_shader(const void *shader);
void deinit_point_shader(const void *shader);

void draw_image(int w, int h, const char *pixels);

void set_separate_brushlayer(char separate_layer);

typedef void* (*pixel_callback)(int, int, const char*, void *env);
void* with_pixels(pixel_callback callback, void *env);

void egl_init(void *window);
void egl_swap();
void egl_finish();

lua_State *initLua();
int loadLuaScript(const char *script);
void useLuaScript(int key);
