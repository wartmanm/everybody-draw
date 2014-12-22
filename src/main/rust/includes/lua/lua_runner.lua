local _main = callbacks.onmove
local _onframe = callbacks.onframe
local _ondown = callbacks.ondown
local _onup = callbacks.onup
local _ondone = callbacks.ondone
if type(_main) ~= "function" then
  loglua("main not defined for runmain()!!")
  return
end
--if type(output) ~= "userdata" then
  --error("output not defined for runmain()!!")
--end
callbacks.pushpoint = function(queue, point)
  ffi.C.lua_pushpoint(output, queue, point)
end
callbacks.pushline = function(queue, pointa, pointb)
  ffi.C.lua_pushline(output, queue, pointa, pointb)
end
callbacks.pushcatmullrom = function(queue, pointa, pointb, pointc, pointd)
  ffi.C.lua_pushcatmullrom(output, queue, pointa, pointb, pointc, pointd)
end
callbacks.pushcubicbezier = function(queue, pointa, pointb, pointc, pointd)
  ffi.C.lua_pushcubicbezier(output, queue, pointa, pointb, pointc, pointd)
end
callbacks.clearlayer = function(layer)
  ffi.C.lua_clearlayer(output, layer)
end
callbacks.savelayers = function()
  ffi.C.lua_savelayers(output)
end
callbacks.saveundo = function()
  ffi.C.lua_saveundobuffer(output)
end

if _onup == nil and _ondown == nil and _onframe == nil and _ondone == nil then
  loglua("setting default pointer callbacks")
  _ondown = callbacks.default_ondown
  _onup = callbacks.default_onup
  _onframe = callbacks.default_onframe
  _ondone = callbacks.default_ondone
end
if _ondone == nil then -- script could have set callbacks.default_ondone to nil
  _ondone = function() end
  setfenv(_ondone, callbacks)
end
callbacks.ondone = _ondone

function runmain()
  if type(_onframe) == "function" then
    _onframe()
  end
  local pointpair = ffi.new("struct ShaderPaintPoint[2]")
  while true do
    local pointstatus = ffi.C.lua_nextpoint(output, pointpair)
    local status = bit.band(0xff00, pointstatus)
    if status == 0x0000 then -- pointer move
      _main(pointpair[0], pointpair[1])
    elseif status == 0x0100 then -- no more points
      break
    elseif status == 0x0200 then -- pointer down
      loglua("got down evt")
      if type(_ondown) == "function" then _ondown(pointpair[0]) end
    else -- pointer up
      loglua("got up evt")
      if type(_onup) == "function" then
        _onup(pointpair[0].counter)
      end
    end
  end
end
