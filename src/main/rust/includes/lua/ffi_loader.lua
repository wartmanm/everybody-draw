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
  void lua_pushcatmullrom(void *output, int queue, struct ShaderPaintPoint *a, struct ShaderPaintPoint *b, struct ShaderPaintPoint *c, struct ShaderPaintPoint *d);
  void lua_pushcubicbezier(void *output, int queue, struct ShaderPaintPoint *a, struct ShaderPaintPoint *b, struct ShaderPaintPoint *c, struct ShaderPaintPoint *d);
  void lua_clearlayer(void *output, int layer);
  void lua_savelayers(void *output);
  void lua_saveundobuffer(void *output);
]]

loglua=ffi.C.lua_log

ShaderPaintPoint=ffi.typeof("struct ShaderPaintPoint")

local function copytable(t)
  out = {}
  for k,v in pairs(t) do
    out[k] = v
  end
  return out
end

function create_sandbox()
  local stringbox = copytable(string)
  local mathbox = copytable(math)
  local tablebox = copytable(table)
  local sandboxed = {
    assert = assert,
    error = error,
    ipairs = ipairs,
    next = next,
    pairs = pairs,
    pcall = pcall,
    print = loglua,
    select = select,
    tonumber = tonumber,
    tostring = tostring,
    type = type,
    unpack = unpack,
    string = stringbox,
    math = mathbox,
    table = tablebox,
    pushpoint = pushpoint,
    pushline = pushline,
    pushcatmullrom = pushcatmullrom,
    pushcubicbezier = pushcubicbezier,
    clearlayer = clearlayer,
    savelayers = savelayers,
    saveundo = saveundo,
    ShaderPaintPoint = ShaderPaintPoint,
  }
  return sandboxed
end
