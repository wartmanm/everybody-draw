function onmove(a, b)
  local statex = math.random(4) - 1
  local statey = math.random(8) - 1
  local out = ShaderPaintPoint(a.x, a.y, a.time, a.size, a.speedx, a.speedy, a.distance, a.counter)
  out.speedx = statex / 4
  out.speedy = statey / 9
  pushpoint(0, out)
end
