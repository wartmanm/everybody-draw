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

void set_brush_texture(int width, int height, const char *pixels, int format);

void clear_buffer();

int compile_point_shader(const char *vec, const char *frag);
int compile_copy_shader(const char *vec, const char *frag);

void deinit_copy_shader(const void *shader);
void deinit_point_shader(const void *shader);

void draw_image(int w, int h, const char *pixels);

struct withpixels_tuple {
  int x;
  int y;
  const char *pixels;
};

struct withpixels_tuple with_pixels();
void release_pixels(const char *pixels);

void egl_init(void *window);
void egl_swap();
void egl_finish();

lua_State *initLua();
void loadLuaScript(const char *script);
