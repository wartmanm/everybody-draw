#include <android/log.h>

#include "lua/lua.h"
#include "lua/lauxlib.h"
#include "lua/lualib.h"
#include "lua/luajit.h"
#include "point.h"

#define  LOG_TAG    "mylua"
#define  LOGI(...)  __android_log_print(ANDROID_LOG_INFO,LOG_TAG,__VA_ARGS__)
#define  LOGE(...)  __android_log_print(ANDROID_LOG_ERROR,LOG_TAG,__VA_ARGS__)

static const char *lua_ffi_script =
"ffi = require(\"ffi\")\n"
"ffi.cdef[[\n"
"  struct ShaderPaintPoint {\n"
"    float x;\n"
"    float y;\n"
"    float time;\n"
"    float size;\n"
"    float speed;\n"
"    float distance;\n"
"    float counter;\n"
"  };\n"
"\n"
"  void pushrustvec(void *vec, struct ShaderPaintPoint *point);\n"
"  char next_point_from_lua(struct ShaderPaintPoint *points);\n"
"  void loglua(const char *message);\n"
"\n"
"]]\n"
"function runmain(x, y, points, main)\n"
"  local pointpair = ffi.new(\"struct ShaderPaintPoint[2]\")\n"
"  while ffi.C.next_point_from_lua(pointpair) ~= 0 do\n"
"    main(pointpair[0], pointpair[1], x, y, points)\n"
"  end\n"
"end\n"
"pushpoint=ffi.C.pushrustvec\n"
"ShaderPaintPoint=ffi.typeof(\"struct ShaderPaintPoint\")\n"
"return {}\n"
"\n";

static const char *defaultscript =
"function main(a, b, x, y, points)\n"
"  pushpoint(points, a)\n"
"  pushpoint(points, b)\n"
"end\n";

static lua_State *L = NULL;

void loglua(char *message) {
  __android_log_print(ANDROID_LOG_INFO, "lua", "%s", message);
}

lua_State *initLua() {
  lua_State *L = luaL_newstate();
  luaL_openlibs(L);
  
  luaJIT_setmode(L, 0, LUAJIT_MODE_ENGINE|LUAJIT_MODE_ON);

  if (1 == luaL_dostring(L, lua_ffi_script)) {
    LOGE("ffi init script failed to load: %s", lua_tostring(L, -1));
    return NULL;
  }
  LOGI("ffi init script loaded :)");

  return L;
}

void finishLua(lua_State *L) {
  lua_close(L);
}

void loadLuaScript(const char *script) {
  if (script == NULL) {
    script = defaultscript;
  }

  if (L == NULL) {
    L = initLua();
  }
  LOGI("lua inited");

  LOGI("loading script:\n%s", script);

  if (1 == luaL_dostring(L, script)) {
    LOGE("script failed to load: %s", lua_tostring(L, -1));
    return;
  }
  LOGI("script loaded :)");

  lua_getglobal(L, "main");
  if (!lua_isfunction(L, -1)) {
    LOGE("no main function defined :(");
    return;
  }
  luaJIT_setmode(L, -1, LUAJIT_MODE_ALLFUNC|LUAJIT_MODE_ON);
  LOGI("main function defined :)");
}

// TODO: would it be better to register a callback from lua?
static void interpolateLua(lua_State *L, int x, int y, void *output) {
  lua_getglobal(L, "runmain");

  lua_pushnumber(L, (float)x);
  lua_pushnumber(L, (float)y);
  lua_pushlightuserdata(L, output);
  lua_getglobal(L, "main");

  if (lua_pcall(L, 4, 0, 0) != 0) {
    LOGE("script failed to run :(");
    const char *msg = lua_tostring(L, -1);
    LOGE("got error message: %s", msg);
    return;
  }
}

void doInterpolateLua(int x, int y, void *output) {
  if (L == NULL) return;
  interpolateLua(L, x, y, output);
}
