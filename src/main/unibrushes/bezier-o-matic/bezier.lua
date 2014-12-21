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
      loglua("added " .. a.counter .. " as point " .. i)
      bycounter[a.counter] = postable
      if i <= 4 then
        active = active + 1
      end
      total = total + 1
      loglua("active: " .. active .. ", total: " .. total)
      return
    end
  end
  loglua("active: " .. active .. ", total: " .. total)
end
function onup(a)
  default_onup(a)
  loglua("got lifted pointer " .. a)
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
        loglua("replaced pointer " .. idx .. " with 5")
        table.remove(positions, 5)
      else
        loglua("removed pointer " .. idx)
        table.remove(positions, idx)
      end
    elseif idx <= 4 then
      loglua("removed pointer " .. idx .. " with no replacement")
      pos.active = false
      active = active - 1
    end
    total = total - 1
  else
    loglua("no match!")
  end
  loglua("active: " .. active .. ", total: " .. total)
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
  local outpoints = ShaderPaintPointArray(4)
  outpoints[0] = positions[1]
  outpoints[1] = positions[2]
  outpoints[2] = positions[3]
  outpoints[3] = positions[4]
  pushcubicbezier(1, outpoints)
end
