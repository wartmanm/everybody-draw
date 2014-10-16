ffi = require("ffi")
ffi.cdef[[
  struct ShaderPaintPoint {
    float x;
    float y;
    float time;
    float size;
    float speedx;
    float speedy;
    float distance;
    float counter;
  };

  void lua_pushpoint(void *output, int queue, struct ShaderPaintPoint *point);
  short lua_nextpoint(void *output, struct ShaderPaintPoint *points);
  void lua_log(const char *message);
  void lua_pushline(void *output, int queue, struct ShaderPaintPoint *pointa, struct ShaderPaintPoint *pointb);
  void lua_clearlayer(void *output, int layer);
]]

pushpoint=ffi.C.lua_pushpoint
pushline=ffi.C.lua_pushline
loglua=ffi.C.lua_log
clearlayer=ffi.C.lua_clearlayer
ShaderPaintPoint=ffi.typeof("struct ShaderPaintPoint")
