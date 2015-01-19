local positions = {}
local bycounter = {}
local active = 0
local total = 0
function ondown(a)
  default_ondown(a)
  postable = {x = a.x, y = a.y, size=2, counter=a.counter, active = true}
  for i = 1,#positions + 1 do
    if positions[i] == nil or positions[i].active == false then
      positions[i] = postable
      print("added " .. a.counter .. " as point " .. i)
      bycounter[a.counter] = postable
      if i <= 4 then
        active = active + 1
      end
      total = total + 1
      print("active: " .. active .. ", total: " .. total)
      return
    end
  end
  print("active: " .. active .. ", total: " .. total)
end
function onup(a)
  default_onup(a)
  print("got lifted pointer " .. a)
  local pos = bycounter[a]
  if pos ~= nil then
    bycounter[a] = nil -- counters are unique, so no point keeping it
    local idx = 1
    while positions[idx] ~= pos do
      idx = idx + 1
    end
    if total > 4 then
      if idx <= 4 then
        positions[idx] = positions[5]
        print("replaced pointer " .. idx .. " with 5")
        table.remove(positions, 5)
      else
        print("removed pointer " .. idx)
        table.remove(positions, idx)
      end
    elseif idx <= 4 then
      print("removed pointer " .. idx .. " with no replacement")
      pos.active = false
      active = active - 1
    end
    total = total - 1
  else
    print("no match!")
  end
  print("active: " .. active .. ", total: " .. total)
end

ondone = default_ondone

function onmove(a, b)
  local position = bycounter[b.counter]
  if position ~= nil then
    position.x = b.x
    position.y = b.y
  end
end
function onframe()
  default_onframe()
  if active < 4 then return end
  clearlayer(1)
  local out1 = ShaderPaintPoint(positions[1])
  local out2 = ShaderPaintPoint(positions[2])
  local out3 = ShaderPaintPoint(positions[3])
  local out4 = ShaderPaintPoint(positions[4])
  pushcubicbezier(1, out1, out2, out3, out4)
end
