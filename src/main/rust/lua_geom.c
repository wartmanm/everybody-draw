#include <android/log.h>

#include <lua.h>
#include <lauxlib.h>
#include <lualib.h>

#define PUSH_LUA_FLOAT(name, value)\
  lua_pushstring(name);\
  lua_pushnumber(value);\
  lua_rawset(L, -3);

#define READ_LUA_FLOAT(name, result)\
  lua_pushstring(name);\
  lua_gettable(L, -2);\
  result = (float)lua_tonumber(L, -1);\
  lua_pop(L, 1);

lua_State *createLua(const char *script) {
  lua_State *L = lua_open();
  luaL_openlibs(L);

  if (1 == luaL_dostring(L, script)) {
    error(L, "script failed to load");
  }

  lua_getglobal("main");
  if (!lua_isfunction(L, -1)) {
    error(L, "no main function defined :(");
  }
  return L;
}

void finishLua(lua_State *L) {
  //TODO: this
}

void pushShaderPoint(lua_State *L, struct ShaderPaintPoint point) {
  lua_newtable(L);
  PUSH_LUA_FLOAT("x", point.pos.x);
  PUSH_LUA_FLOAT("y", point.pos.y);
  PUSH_LUA_FLOAT("time", point.pos.time);
  PUSH_LUA_FLOAT("size", point.pos.size);
  PUSH_LUA_FLOAT("distance", point.pos.distance);
  PUSH_LUA_FLOAT("counter", point.pos.counter);
}

// TODO: would it be better to register a callback from lua?
void interpolateLua(lua_State *L, ShaderPaintPoint startpoint, ShaderPaintPoint endpoint, ShaderCallback callback) {
  lua_getglobal(L, "main");
  
  pushShaderPoint(L, startpoint);
  pushShaderPoint(L, endpoint);

  if (lua_pcall(L, 2, 1, 0) != 0) {
    error(L, "script failed to run");
  }

  if (!lua_istable(L, -1)) {
    error(L, "result must be table");
  }

  int length = lua_objlen(L, -1);

  ShaderPaintPoint points[length];
  /*lua_pushnil(L);*/
  for (int i = 0; i < length; i++) {
    /*lua_next(L, -2);*/
    lua_rawgeti(L, -1, i);

    READ_LUA_FLOAT("x", points[i].pos.x);
    READ_LUA_FLOAT("y", points[i].pos.y);
    READ_LUA_FLOAT("time", points[i].time);
    READ_LUA_FLOAT("size", points[i].size);
    READ_LUA_FLOAT("distance", points[i].distance);
    READ_LUA_FLOAT("counter", points[i].counter);

    lua_pop(1);
  }
  callback(points, length);
}

