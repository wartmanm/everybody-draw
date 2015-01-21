local queue_layer_save = false
local downcount = 0
function default_ondown(pointer)
  downcount = downcount + 1
  print(downcount .. " pointers down")
end
function default_onup(pointer)
  downcount = downcount - 1
  print(downcount .. " pointers down")
  if downcount == 0 then
    queue_layer_save = true
  end
end
function default_onframe(x, y)
  if queue_layer_save == true then
    print("saving layers")
    savelayers()
    saveundo()
    queue_layer_save = false
  end
end
function default_ondone()
  print("saving layers")
  savelayers()
end

