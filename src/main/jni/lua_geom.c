#include <android/log.h>

#include "lua/lua.h"
#include "lua/lauxlib.h"
#include "lua/lualib.h"
#include "point.h"

#define  LOG_TAG    "mylua"
#define  LOGI(...)  __android_log_print(ANDROID_LOG_INFO,LOG_TAG,__VA_ARGS__)
#define  LOGE(...)  __android_log_print(ANDROID_LOG_ERROR,LOG_TAG,__VA_ARGS__)

#define PUSH_LUA_FLOAT(name, value)\
  lua_pushstring(L, name);\
  lua_pushnumber(L, value);\
  lua_rawset(L, -3);

#define READ_LUA_FLOAT(name, result)\
  lua_pushstring(L, name);\
  lua_gettable(L, -2);\
  result = (float)lua_tonumber(L, -1);\
  lua_pop(L, 1);

static lua_State *L = NULL;

lua_State *initLua() {
  lua_State *L = luaL_newstate();
  luaL_openlibs(L);

  return L;
}

void finishLua(lua_State *L) {
  lua_close(L);
}

void loadLuaScript(const char *script) {

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
  LOGI("main function defined :)");
}


/*void initLuaShaderPaintPoint(lua_State *L) {*/
  /*static const struct luaL_reg shaderpointlib [] = {*/
    /*{"new", makeLuaShaderPaintPoint},*/
  /*};*/
  /*static const struct luaL_reg shaderpointlib_m [] = {*/
    /*{*/
    /*}*/
  /*};*/
  /*luaL_newmetatable(L, "glstuff.shaderpaintpoint");*/
  /*luaL_openlib(L, "shaderpaintpoint", shaderpointlib, 0);*/
/*}*/

/*void makeLuaShaderPaintPoint(lua_State *L) {*/
  /*struct ShaderPaintPoint *p = (* struct ShaderPaintPoint) lua_newuserdata(L, sizeof(struct ShaderPaintPoint));*/
  /*lua_getmetatable(L, "glstuff.shaderpaintpoint");*/
  /*lua_setmetatable(L, -2);*/
/*}*/

static void pushShaderPoint(lua_State *L, struct ShaderPaintPoint *point) {
  lua_newtable(L);
  PUSH_LUA_FLOAT("x", point->pos.x);
  PUSH_LUA_FLOAT("y", point->pos.y);
  PUSH_LUA_FLOAT("time", point->time);
  PUSH_LUA_FLOAT("size", point->size);
  PUSH_LUA_FLOAT("distance", point->distance);
  PUSH_LUA_FLOAT("counter", point->counter);
}

// TODO: would it be better to register a callback from lua?
static void interpolateLua(lua_State *L, struct ShaderPaintPoint *startpoint, struct ShaderPaintPoint *endpoint, void *output, ShaderCallback callback) {
  lua_getglobal(L, "main");
  
  pushShaderPoint(L, startpoint);
  pushShaderPoint(L, endpoint);

  if (lua_pcall(L, 2, 1, 0) != 0) {
    LOGE("script failed to run :(");
    const char *msg = lua_tostring(L, -1);
    LOGE("got error message: %s", msg);
    return;
  }

  if (!lua_istable(L, -1)) {
    LOGE("result must be table :(");
    return;
  }

  int length = lua_objlen(L, -1);

  struct ShaderPaintPoint points[length];
  /*lua_pushnil(L);*/
  for (int i = 0; i < length; i++) {
    /*lua_next(L, -2);*/
    lua_rawgeti(L, -1, i+1);

    READ_LUA_FLOAT("x", points[i].pos.x);
    READ_LUA_FLOAT("y", points[i].pos.y);
    READ_LUA_FLOAT("time", points[i].time);
    READ_LUA_FLOAT("size", points[i].size);
    READ_LUA_FLOAT("distance", points[i].distance);
    READ_LUA_FLOAT("counter", points[i].counter);

    lua_pop(L, 1);
  }
  callback(points, length, output);
}

void doInterpolateLua(struct ShaderPaintPoint *startpoint, struct ShaderPaintPoint *endpoint, void *output, ShaderCallback callback) {
  if (L == NULL) return;
  interpolateLua(L, startpoint, endpoint, output, callback);
}
