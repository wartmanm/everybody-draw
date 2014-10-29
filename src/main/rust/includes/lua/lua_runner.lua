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
  local queue_layer_save = false
  loglua("setting default pointer callbacks")
  local downcount = 0
  function default_ondown(pointer, output)
    downcount = downcount + 1
  end
  function default_onup(pointer, output)
    downcount = downcount - 1
    if downcount == 0 then
      queue_layer_save = true
    end
  end
  function default_onframe(x, y, output)
    if queue_layer_save == true then
      savelayers(output)
      queue_layer_save = false
    end
  end
  function default_ondone(output)
    savelayers(output)
  end
  _ondown = default_ondown
  _onup = default_onup
  _onframe = default_onframe
  _ondone = default_ondone
elseif _ondone == nil then
  _ondone = function(output) end
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
      if type(_ondown) == "function" then _ondown(pointpair[0], output) end
    else -- pointer up
      if type(_onup) == "function" then
        local pointer = bit.band(0x00ff, pointstatus)
        _onup(pointer, output)
      end
    end
  end
end
