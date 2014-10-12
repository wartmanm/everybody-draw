local _main = main
local _onframe = onframe
if type(main) ~= "function" then
  loglua("main not defined for runmain()!!")
  return
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
  while ffi.C.next_point_from_lua(output, pointpair) ~= 0 do
    _main(pointpair[0], pointpair[1], x, y, output)
  end
end
