function main(a, b, x, y, points)
  local statex = math.random(4) - 1
  local statey = math.random(8) - 1
  local out = ShaderPaintPoint(a.x, a.y, a.time, a.size, a.speed, a.distance, a.counter)
  out.size = statex / 4
  out.speed = statey / 9
  pushpoint(points, 0, out)
end
