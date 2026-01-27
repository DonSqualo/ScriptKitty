-- Mittens Standard Library: Transforms
-- Transformation operations for shapes

local Transforms = {}

-- 3D Vector operations
local Vec3 = {}

function Vec3.new(x, y, z)
  return {x = x or 0, y = y or 0, z = z or 0}
end

function Vec3.add(a, b)
  return {x = a.x + b.x, y = a.y + b.y, z = a.z + b.z}
end

function Vec3.sub(a, b)
  return {x = a.x - b.x, y = a.y - b.y, z = a.z - b.z}
end

function Vec3.mul(v, s)
  return {x = v.x * s, y = v.y * s, z = v.z * s}
end

function Vec3.dot(a, b)
  return a.x * b.x + a.y * b.y + a.z * b.z
end

function Vec3.cross(a, b)
  return {
    x = a.y * b.z - a.z * b.y,
    y = a.z * b.x - a.x * b.z,
    z = a.x * b.y - a.y * b.x
  }
end

function Vec3.length(v)
  return math.sqrt(v.x * v.x + v.y * v.y + v.z * v.z)
end

function Vec3.normalize(v)
  local len = Vec3.length(v)
  if len == 0 then return {x = 0, y = 0, z = 0} end
  return Vec3.mul(v, 1 / len)
end

Transforms.Vec3 = Vec3

-- 4x4 Matrix operations
local Mat4 = {}

function Mat4.identity()
  return {
    1, 0, 0, 0,
    0, 1, 0, 0,
    0, 0, 1, 0,
    0, 0, 0, 1
  }
end

function Mat4.translation(x, y, z)
  return {
    1, 0, 0, x,
    0, 1, 0, y,
    0, 0, 1, z,
    0, 0, 0, 1
  }
end

function Mat4.scale(sx, sy, sz)
  return {
    sx, 0,  0,  0,
    0,  sy, 0,  0,
    0,  0,  sz, 0,
    0,  0,  0,  1
  }
end

function Mat4.rotationX(angle)
  local c, s = math.cos(angle), math.sin(angle)
  return {
    1, 0,  0, 0,
    0, c, -s, 0,
    0, s,  c, 0,
    0, 0,  0, 1
  }
end

function Mat4.rotationY(angle)
  local c, s = math.cos(angle), math.sin(angle)
  return {
     c, 0, s, 0,
     0, 1, 0, 0,
    -s, 0, c, 0,
     0, 0, 0, 1
  }
end

function Mat4.rotationZ(angle)
  local c, s = math.cos(angle), math.sin(angle)
  return {
    c, -s, 0, 0,
    s,  c, 0, 0,
    0,  0, 1, 0,
    0,  0, 0, 1
  }
end

function Mat4.multiply(a, b)
  local result = {}
  for i = 0, 3 do
    for j = 0, 3 do
      local sum = 0
      for k = 0, 3 do
        sum = sum + a[i * 4 + k + 1] * b[k * 4 + j + 1]
      end
      result[i * 4 + j + 1] = sum
    end
  end
  return result
end

function Mat4.transformPoint(m, p)
  return {
    x = m[1] * p.x + m[2] * p.y + m[3] * p.z + m[4],
    y = m[5] * p.x + m[6] * p.y + m[7] * p.z + m[8],
    z = m[9] * p.x + m[10] * p.y + m[11] * p.z + m[12]
  }
end

Transforms.Mat4 = Mat4

--- Translate a shape
-- @param shape Shape to translate
-- @param x X offset
-- @param y Y offset
-- @param z Z offset
-- @return New translated shape
function Transforms.translate(shape, x, y, z)
  local new_shape = {}
  for k, v in pairs(shape) do
    if k == "_ops" then
      -- Deep copy ops array
      new_shape[k] = {}
      for i, op in ipairs(v) do
        new_shape[k][i] = op
      end
    else
      new_shape[k] = v
    end
  end
  -- Initialize _ops if not present
  new_shape._ops = new_shape._ops or {}
  table.insert(new_shape._ops, {op = "translate", x = x, y = y, z = z})
  setmetatable(new_shape, getmetatable(shape))
  return new_shape
end

--- Rotate a shape (Euler angles in degrees)
-- @param shape Shape to rotate
-- @param rx Rotation around X axis
-- @param ry Rotation around Y axis
-- @param rz Rotation around Z axis
-- @return New rotated shape
function Transforms.rotate(shape, rx, ry, rz)
  local new_shape = {}
  for k, v in pairs(shape) do
    if k == "_ops" then
      -- Deep copy ops array
      new_shape[k] = {}
      for i, op in ipairs(v) do
        new_shape[k][i] = op
      end
    else
      new_shape[k] = v
    end
  end
  -- Initialize _ops if not present
  new_shape._ops = new_shape._ops or {}
  table.insert(new_shape._ops, {op = "rotate", x = rx, y = ry, z = rz})
  setmetatable(new_shape, getmetatable(shape))
  return new_shape
end

--- Scale a shape uniformly or non-uniformly
-- @param shape Shape to scale
-- @param sx Scale factor X (or uniform if sy/sz not provided)
-- @param sy Scale factor Y (optional)
-- @param sz Scale factor Z (optional)
-- @return New scaled shape
function Transforms.scale(shape, sx, sy, sz)
  sy = sy or sx
  sz = sz or sx
  local new_shape = {}
  for k, v in pairs(shape) do
    if k == "_ops" then
      -- Deep copy ops array
      new_shape[k] = {}
      for i, op in ipairs(v) do
        new_shape[k][i] = op
      end
    else
      new_shape[k] = v
    end
  end
  -- Initialize _ops if not present
  new_shape._ops = new_shape._ops or {}
  table.insert(new_shape._ops, {op = "scale", x = sx, y = sy, z = sz})
  setmetatable(new_shape, getmetatable(shape))
  return new_shape
end

--- Mirror a shape across a plane
-- @param shape Shape to mirror
-- @param plane "XY", "XZ", or "YZ"
-- @return New mirrored shape
function Transforms.mirror(shape, plane)
  local sx, sy, sz = 1, 1, 1
  if plane == "YZ" then sx = -1
  elseif plane == "XZ" then sy = -1
  elseif plane == "XY" then sz = -1
  end
  return Transforms.scale(shape, sx, sy, sz)
end

--- Create an array of shapes in a linear pattern
-- @param shape Base shape
-- @param count Number of copies
-- @param dx X spacing
-- @param dy Y spacing
-- @param dz Z spacing
-- @return Group of shapes
function Transforms.linear_pattern(shape, count, dx, dy, dz)
  dy = dy or 0
  dz = dz or 0
  local shapes = {}
  for i = 0, count - 1 do
    shapes[i + 1] = Transforms.translate(shape, i * dx, i * dy, i * dz)
  end
  return {
    _type = "group",
    _children = shapes,
    _name = "linear_pattern"
  }
end

--- Create an array of shapes in a circular pattern
-- @param shape Base shape
-- @param count Number of copies
-- @param radius Distance from center
-- @param axis Rotation axis ("X", "Y", or "Z")
-- @return Group of shapes
function Transforms.circular_pattern(shape, count, radius, axis)
  axis = axis or "Z"
  local shapes = {}
  local angle_step = 360 / count

  for i = 0, count - 1 do
    local angle = math.rad(i * angle_step)
    local x, y, z = 0, 0, 0
    local rx, ry, rz = 0, 0, 0

    if axis == "Z" then
      x = radius * math.cos(angle)
      y = radius * math.sin(angle)
      rz = i * angle_step
    elseif axis == "X" then
      y = radius * math.cos(angle)
      z = radius * math.sin(angle)
      rx = i * angle_step
    elseif axis == "Y" then
      x = radius * math.cos(angle)
      z = radius * math.sin(angle)
      ry = i * angle_step
    end

    local translated = Transforms.translate(shape, x, y, z)
    shapes[i + 1] = Transforms.rotate(translated, rx, ry, rz)
  end

  return {
    _type = "group",
    _children = shapes,
    _name = "circular_pattern"
  }
end

return Transforms
