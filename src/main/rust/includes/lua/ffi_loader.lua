ffi = require("ffi")
ffi.cdef[[
  struct ShaderPaintPoint {
    float x;
    float y;
    float time;
    float size;
    float speed;
    float distance;
    float counter;
  };

  void pushrustvec(void *output, int queue, struct ShaderPaintPoint *point);
  char next_point_from_lua(void *output, struct ShaderPaintPoint *points);
  void loglua(const char *message);
]]

pushpoint=ffi.C.pushrustvec
loglua=ffi.C.loglua
ShaderPaintPoint=ffi.typeof("struct ShaderPaintPoint")
