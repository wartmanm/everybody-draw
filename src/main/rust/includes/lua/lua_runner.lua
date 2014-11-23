local _main = callbacks.main
local _onframe = callbacks.onframe
local _ondown = callbacks.ondown
local _onup = callbacks.onup
local _ondone = callbacks.ondone
if type(_main) ~= "function" then
  loglua("main not defined for runmain()!!")
  return
end

local queue_layer_save = false
local downcount = 0
local function default_ondown(pointer, output)
  downcount = downcount + 1
  loglua("new pointer, count is " .. downcount)
end
local function default_onup(pointer, output)
  downcount = downcount - 1
  loglua("lifted pointer, count is " .. downcount)
  if downcount == 0 then
    queue_layer_save = true
  end
end
local function default_onframe(x, y, output)
  if queue_layer_save == true then
    loglua("saving layers")
    savelayers(output)
    saveundo(output)
    queue_layer_save = false
  end
end
local function default_ondone(output)
  loglua("in ondone callback")
  savelayers(output)
end

callbacks.default_ondone = default_ondone
callbacks.default_onframe = default_onframe
callbacks.default_onup = default_onup
callbacks.default_ondown = default_ondown

if _onup == nil and _ondown == nil and _onframe == nil and _ondone == nil then
  loglua("setting default pointer callbacks")
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
      loglua("got down evt")
      if type(_ondown) == "function" then _ondown(pointpair[0], output) end
    else -- pointer up
      loglua("got up evt")
      if type(_onup) == "function" then
        local pointer = bit.band(0x00ff, pointstatus)
        _onup(pointer, output)
      end
    end
  end
end
