#include <string.h>
#include <android/log.h>

#include "lua/lua.h"
#include "lua/lauxlib.h"
#include "lua/lualib.h"
#include "lua/luajit.h"
#include "point.h"

#define  LOG_TAG    "luageom"
#define  LOGI(...)  __android_log_print(ANDROID_LOG_INFO,LOG_TAG,__VA_ARGS__)
#define  LOGE(...)  __android_log_print(ANDROID_LOG_ERROR,LOG_TAG,__VA_ARGS__)

int glstuff_lua_key = 0;

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
"  void pushrustvec(void *output, int queue, struct ShaderPaintPoint *point);\n"
"  char next_point_from_lua(void *output, struct ShaderPaintPoint *points);\n"
"  void loglua(const char *message);\n"
"\n"
"]]\n"
"\n"
"pushpoint=ffi.C.pushrustvec\n"
"ShaderPaintPoint=ffi.typeof(\"struct ShaderPaintPoint\")\n";

static const char *lua_runner =
"local _main = main\n"
"local _onframe = onframe\n"
"if type(main) ~= \"function\" then\n"
"  loglua(\"main not defined for runmain()!!\")\n"
"  return\n"
"end\n"
"\n"
"function runmain(x, y, output)\n"
"  if type(_onframe) == \"function\" then\n"
"    onframe(x, y, output)\n"
"  end\n"
"  if type(_main) ~= \"function\" then\n"
"    loglua(\"main doesn't exist!!\")\n"
"    return\n"
"  end\n"
"  local pointpair = ffi.new(\"struct ShaderPaintPoint[2]\")\n"
"  while ffi.C.next_point_from_lua(output, pointpair) ~= 0 do\n"
"    _main(pointpair[0], pointpair[1], x, y, output)\n"
"  end\n"
"end\n"
"\n";

static const char *defaultscript =
"function main(a, b, x, y, points)\n"
"  pushpoint(points, 0, a)\n"
"  pushpoint(points, 0, b)\n"
"end\n";

static lua_State *L = NULL;

void loglua(char *message) {
  __android_log_print(ANDROID_LOG_INFO, "luascript", "%s", message);
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

struct sized_script {
  const char *data;
  int len;
  char done;
};

const char* plainreader(lua_State *L, void *data, size_t *size) {
  struct sized_script* scriptdata = (struct sized_script*) data;
  if (scriptdata->done) return NULL;
  *size = scriptdata->len;
  scriptdata->done = 1;
  return scriptdata->data;
}

int loadLuaScript(const char *scriptchars, int len) {
  if (scriptchars == NULL) {
    scriptchars = defaultscript;
    len = strlen(defaultscript);
  }
  struct sized_script script = {
    .data = scriptchars,
    .len = len,
    .done = 0,
  };

  if (L == NULL) {
    L = initLua();
  }
  LOGI("lua inited");

  lua_pushnil(L);
  lua_setglobal(L, "main");
  lua_pushnil(L);
  lua_setglobal(L, "onframe");

  int key = ((int) &glstuff_lua_key) + glstuff_lua_key;
  lua_pushlightuserdata(L, (void*)key);

  LOGI("loading script:\n%s", scriptchars);

  if (0 != lua_load(L, plainreader, &script, "loadLuaScript() input")) {
    LOGE("script failed to load: %s", lua_tostring(L, -1));
    lua_pop(L, 1);
    return -1;
  }

  if (0 != lua_pcall(L, 0, LUA_MULTRET, 0)) {
    LOGE("script failed to run: %s", lua_tostring(L, -1));
    lua_pop(L, 1);
    return -1;
  }

  LOGI("script loaded :)");

  lua_getglobal(L, "main");
  if (!lua_isfunction(L, -1)) {
    LOGE("no main function defined :(");
    lua_pop(L, 2);
    return -1;
  }
  luaJIT_setmode(L, -1, LUAJIT_MODE_ALLFUNC|LUAJIT_MODE_ON);
  LOGI("main function defined :)");
  lua_pop(L, 1);

  // FIXME compile runner once
  if (1 == luaL_dostring(L, lua_runner)) {
    LOGE("lua runner failed to load: %s", lua_tostring(L, -1));
    lua_pop(L, 1);
    return -1;
  }

  lua_getglobal(L, "runmain");
  if (!lua_isfunction(L, -1)) {
    LOGE("runmain not defined :(");
    lua_pop(L, 2);
    return -1;
  }
  luaJIT_setmode(L, -1, LUAJIT_MODE_ALLFUNC|LUAJIT_MODE_ON);

  lua_settable(L, LUA_REGISTRYINDEX);
  glstuff_lua_key += 1;
  return key;
}

void unloadLuaScript(int key) {
  lua_pushlightuserdata(L, (void*)key);
  lua_pushnil(L);
  lua_settable(L, LUA_REGISTRYINDEX);
}

void useLuaScript(int key) {
  lua_pushlightuserdata(L, (void*)key);
  lua_gettable(L, LUA_REGISTRYINDEX);
  lua_setglobal(L, "runmain");
}

// TODO: would it be better to register a callback from lua?
static void interpolateLua(lua_State *L, int x, int y, void *output) {
  lua_getglobal(L, "runmain");

  lua_pushnumber(L, (float)x);
  lua_pushnumber(L, (float)y);
  lua_pushlightuserdata(L, output);

  if (lua_pcall(L, 3, 0, 0) != 0) {
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
