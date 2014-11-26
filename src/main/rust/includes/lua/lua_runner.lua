local _main = callbacks.main
local _onframe = callbacks.onframe
local _ondown = callbacks.ondown
local _onup = callbacks.onup
local _ondone = callbacks.ondone
if type(_main) ~= "function" then
  loglua("main not defined for runmain()!!")
  return
end

if _onup == nil and _ondown == nil and _onframe == nil and _ondone == nil then
  loglua("setting default pointer callbacks")
  _ondown = callbacks.default_ondown
  _onup = callbacks.default_onup
  _onframe = callbacks.default_onframe
  _ondone = callbacks.default_ondone
end
if _ondone == nil then -- script could have set callbacks.default_ondone to nil
  _ondone = function(output) end
  setfenv(_ondone, callbacks)
end
callbacks.ondone = _ondone

function runmain(x, y, output)
  if type(_onframe) == "function" then
    _onframe(x, y, output)
  end
  local pointpair = ffi.new("struct ShaderPaintPoint[2]")
  while true do
    local pointstatus = ffi.C.lua_nextpoint(output, pointpair)
    local status = bit.band(0xff00, pointstatus)
    if status == 0x0000 then -- pointer move
      _main(pointpair[0], pointpair[1], x, y, output)
    elseif status == 0x0100 then -- no more points
      break
    elseif status == 0x0200 then -- pointer down
      loglua("got down evt")
      if type(_ondown) == "function" then _ondown(pointpair[0], output) end
    else -- pointer up
      loglua("got up evt")
      if type(_onup) == "function" then
        _onup(pointpair[0].counter, output)
      end
    end
  end
end
