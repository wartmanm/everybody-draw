local _main = main
local _onframe = onframe
local _ondown = ondown
local _onup = onup
if type(main) ~= "function" then
  loglua("main not defined for runmain()!!")
  return
end

if onup == nil and ondown == nil then
  local downcount = 0
  function default_ondown(pointer)
    downcount = downcount + 1
  end
  function default_onup(pointer)
    downcount = downcount - 1
    if downcount == 0 then
      savelayers()
    end
  end
  _ondown = default_ondown
  _onup = default_onup
end

function runmain(x, y, output)
  if type(_onframe) == "function" then
    _onframe(x, y, output)
  end
  if type(_main) ~= "function" then
    loglua("main doesn't exist!!")
    return
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
      downcount = downcount + 1
      if type(_ondown) == "function" then _ondown(pointpair[0]) end
    else -- pointer up
      downcount = downcount - 1
      if type(_onup) == "function" then
        local pointer = bit.band(0x00ff, pointstatus)
        _onup(pointer)
      end
    end
  end
end
