local kStop = 0 -- Jare is cleaning.
local kJare = 1
local kKaki = 2 -- Kaki is scratching the ear.
local kAkubi = 3 -- Akubi is yawning.
local kSleep = 4
local kAwake = 5
local kMoveUp = 6
local kMoveDown = 7
local kMoveLeft = 8
local kMoveRight = 9
local kMoveUpLeft = 10
local kMoveUpRight = 11
local kMoveDownLeft = 12
local kMoveDownRight = 13
-- Togi is scratching the borders.
local kTogiUp = 14
local kTogiDown = 15
local kTogiLeft = 16
local kTogiRight = 17

local kSpeed = 16 -- Originally 16
local mHalfImage = 16
local kSinEighthPIM = 383 -- sin(pi/8) * 1000
local kSinThreeEighthsPIM = 924 -- sin(pi * 3/8) * 1000

local mNeko = {x = 200, y = 200, mTickCount = 0, mStateCount = 0, mState = 0}
local mMickey = {x = 200, y = 200}

local mStateNames = { [0] =
		"kStop", -- 0; -- Jare is cleaning.
		"kJare", -- 1; -- Kaki is scratching the ear.
		"kKaki", -- 2; -- Akubi is yawning.
		"kAkubi", -- 3;
		"kSleep", -- 4;
		"kAwake", -- 5;
		"kMoveUp", -- 6;
		"kMoveDown", -- 7
		"kMoveLeft", -- 8
		"kMoveRight", -- 9
		"kMoveUpLeft", -- 10
		"kMoveUpRight", -- 11
		"kMoveDownLeft", -- 12
		"kMoveDownRight", -- 13
		-- Togi is scratching the borders.
		"kTogiUp", -- 14
		"kTogiDown", -- 15
		"kTogiLeft", -- 16
		"kTogiRight" -- 17;
  }

local kImageIndexes = { [0] =
  0, 1, 2, 4, 5, 7,
  8, 10, 12, 14,
  16, 18, 20, 22,
  24, 26, 28, 30
}

local kStateCountLimits = { [0] =
  4, 10, 4, 3, 0, 3,
  0, 0, 0, 0, -- Moves
  0, 0, 0, 0, -- Diagonal moves
  10, 10, 10, 10 -- Togis
}

function calculateDeltas(neko, mickey)
  local deltaX = mickey.x - neko.x
  local deltaY = mickey.y - neko.y

  local lengthSquared = deltaX * deltaX + deltaY * deltaY
  if (lengthSquared == 0) then
    return 0, 0
  else
    local length = math.sqrt(lengthSquared)
    if (length <= kSpeed) then
      return deltaX, deltaY
    else
      local mMoveDeltaX = math.floor((kSpeed * deltaX) / length)
      local mMoveDeltaY = math.floor((kSpeed * deltaY) / length)
      return mMoveDeltaX, mMoveDeltaY
    end
  end
end

function direction(neko, mMoveDeltaX, mMoveDeltaY)
  local newState

  if (mMoveDeltaX == 0 and mMoveDeltaY == 0) then
    newState = kStop
  else
    local length = math.floor(math.sqrt(
    mMoveDeltaX * mMoveDeltaX + mMoveDeltaY * mMoveDeltaY))
    local sinThetaM = math.floor(-(mMoveDeltaY * 1000) / length)

    if (mMoveDeltaX >= 0) then
      if (sinThetaM > kSinThreeEighthsPIM) then newState = kMoveUp
      elseif (sinThetaM <= kSinThreeEighthsPIM and  sinThetaM > kSinEighthPIM) then newState = kMoveUpRight
      elseif (sinThetaM <= kSinEighthPIM and  sinThetaM > -kSinEighthPIM) then newState = kMoveRight
      elseif (sinThetaM <= -kSinEighthPIM and sinThetaM > -kSinThreeEighthsPIM) then newState = kMoveDownRight
      else newState = kMoveDown end
    else
      if (sinThetaM > kSinThreeEighthsPIM) then newState = kMoveUp
      elseif (sinThetaM <= kSinThreeEighthsPIM and sinThetaM > kSinEighthPIM) then newState = kMoveUpLeft
      elseif (sinThetaM <= kSinEighthPIM and sinThetaM > -kSinEighthPIM) then newState = kMoveLeft
      elseif (sinThetaM <= -kSinEighthPIM and sinThetaM > -kSinThreeEighthsPIM) then newState = kMoveDownLeft
      else newState = kMoveDown end
    end
  end
  if not (newState == neko.mState) then
    setState(neko, newState)
  end
end

function setState(neko, state)
  print("setting state to " .. mStateNames[state])
  neko.mTickCount = 0
  neko.mStateCount = 0
  neko.mState = state
end

function bound()
  return false
end

function getImageIndex(mState, mTickCount)
  local index = kImageIndexes[mState]
  if (mState == kJare) then
    return index - (mTickCount % 2)
  elseif not (mState == kStop
    or mState == kAkubi
    or mState == kAwake) then
      return index + mTickCount % 2
  end
  return index
end

function think(neko, mickey, mWidth, mHeight)
  neko.mTickCount = neko.mTickCount + 1
  if (neko.mTickCount % 2 == 0) then
    neko.mStateCount = neko.mStateCount + 1
  end
  local mMoveDeltaX, mMoveDeltaY = calculateDeltas(neko, mickey)
  local mState = neko.mState

  if mState == kStop then
    if not (mMoveDeltaX == 0 and mMoveDeltaY == 0) then
      setState(neko, kAwake)
    elseif (neko.mStateCount < kStateCountLimits[mState]) then
      -- will it parse??
    else
      if (--[[mMoveDeltaX < 0 and ]] neko.x <= mHalfImage) then
        setState(neko, kTogiLeft)
      elseif (--[[mMoveDeltaX > 0 and ]] neko.x >= (mWidth - 1 - mHalfImage)) then
        setState(neko, kTogiRight)
      elseif (--[[mMoveDeltaY < 0 and ]] neko.y <= mHalfImage) then
        setState(neko, kTogiUp)
      elseif (--[[mMoveDeltaY > 0 and ]] neko.y >= (mHeight - 1 - mHalfImage)) then
        setState(neko, kTogiDown)
      else
        setState(neko, kJare)
      end
    end
  elseif mState == kJare then
    if not (mMoveDeltaX == 0 and mMoveDeltaY == 0) then
      setState(neko, kAwake)
    elseif (neko.mStateCount >= kStateCountLimits[mState]) then
      setState(neko, kKaki)
    end
  elseif mState == kKaki then
    if not (mMoveDeltaX == 0 and mMoveDeltaY == 0) then
      setState(neko, kAwake)
    elseif (neko.mStateCount >= kStateCountLimits[mState]) then
      setState(neko, kAkubi)
    end
  elseif mState == kAkubi then
    if not (mMoveDeltaX == 0 and mMoveDeltaY == 0) then
      setState(neko, kAwake)
    elseif (neko.mStateCount >= kStateCountLimits[mState]) then
      setState(neko, kSleep)
    end
  elseif mState == kSleep then
    if not (mMoveDeltaX == 0 and mMoveDeltaY == 0) then
      setState(neko, kAwake)
    end
  elseif mState == kAwake then
    if (neko.mStateCount >= kStateCountLimits[mState]) then
      direction(neko, mMoveDeltaX, mMoveDeltaY)
    end
  elseif mState == kMoveUp
    or mState == kMoveDown
    or mState == kMoveUp
    or mState == kMoveDown
    or mState == kMoveLeft
    or mState == kMoveRight
    or mState == kMoveUpLeft
    or mState == kMoveUpRight
    or mState == kMoveDownLeft
    or mState == kMoveDownRight then
    neko.x = neko.x + mMoveDeltaX
    neko.y = neko.y + mMoveDeltaY
    direction(neko, mMoveDeltaX, mMoveDeltaY)
    if (bound()) then
      setState(neko, kStop)
    end
  elseif mState == kTogiUp
    or mState == kTogiDown
    or mState == kTogiLeft
    or mState == kTogiRight then
    if not (mMoveDeltaX == 0 and mMoveDeltaY == 0) then
      setState(neko, kAwake)
    elseif (neko.mStateCount >= kStateCountLimits[mState]) then
      setState(neko, kKaki)
    end
  end
  local imgIndex = getImageIndex(neko.mState, neko.mTickCount)
  return neko.x, neko.y, imgIndex
end

function onframe()
  local nekoX, nekoY, imgIndex = think(mNeko, mMickey, x, y)
  local imgX = imgIndex % 4
  local imgY = math.floor(imgIndex / 4)
  local nekopoint = ShaderPaintPoint(nekoX, nekoY, 0, 0, imgX / 4, imgY / 9, 0, 0)
  local mickeypoint = ShaderPaintPoint(mMickey.x, mMickey.y, 0, 0, 3/4, 8/9, 0, 0)
  --print("neko is at " .. mNeko.x .. ", " .. mNeko.y .. "; state: " .. mStateNames[mNeko.mState])
  clearlayer(1)
  pushpoint(1, mickeypoint)
  pushpoint(1, nekopoint)
end

function onmove(a, b)
  mMickey.x = a.x
  mMickey.y = a.y
end

function onup(pointer)
  -- don't copy neko down
end
