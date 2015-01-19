local queue_layer_save = false
local downcount = 0
function default_ondown(pointer, output)
  downcount = downcount + 1
  loglua("new pointer, count is " .. downcount)
end
function default_onup(pointer, output)
  downcount = downcount - 1
  loglua("lifted pointer, count is " .. downcount)
  if downcount == 0 then
    queue_layer_save = true
  end
end
function default_onframe(x, y, output)
  if queue_layer_save == true then
    loglua("saving layers")
    savelayers(output)
    saveundo(output)
    queue_layer_save = false
  end
end
function default_ondone(output)
  loglua("in ondone callback")
  savelayers(output)
end

