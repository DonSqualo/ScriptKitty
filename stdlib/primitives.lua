-- ScriptCAD Standard Library: Primitives
-- Basic geometric primitives using Signed Distance Functions (SDF)

local Primitives = {}

-- Create a shape object with SDF and metadata
local function Shape(sdf_func, bounds, metadata)
  local shape = {
    _type = "shape",
    _sdf = sdf_func,
    _bounds = bounds or {min = {-1e6, -1e6, -1e6}, max = {1e6, 1e6, 1e6}},
    _transform = {position = {0, 0, 0}, rotation = {0, 0, 0}, scale = {1, 1, 1}},
    _material = nil,
    _metadata = metadata or {}
  }

  setmetatable(shape, {__index = {
    at = function(self, x, y, z)
      self._transform.position = {x, y, z}
      return self
    end,

    rotate = function(self, rx, ry, rz)
      self._transform.rotation = {rx, ry, rz}
      return self
    end,

    scale = function(self, sx, sy, sz)
      sy = sy or sx
      sz = sz or sx
      self._transform.scale = {sx, sy, sz}
      return self
    end,

    material = function(self, mat)
      self._material = mat
      return self
    end,

    color = function(self, r, g, b, a)
      self._color = {r, g, b, a or 1.0}
      return self
    end,

    name = function(self, n)
      self._name = n
      return self
    end,

    eval = function(self, x, y, z)
      return self._sdf(x, y, z)
    end,

    serialize = function(self)
      return {
        type = self._metadata.primitive,
        params = self._metadata.params,
        transform = self._transform,
        material = self._material,
        color = self._color,
        name = self._name
      }
    end
  }})

  return shape
end

-- SDF for box
local function box_sdf(hw, hd, hh)
  return function(x, y, z)
    local qx = math.abs(x) - hw
    local qy = math.abs(y) - hd
    local qz = math.abs(z) - hh
    local outside = math.sqrt(
      math.max(qx, 0)^2 +
      math.max(qy, 0)^2 +
      math.max(qz, 0)^2
    )
    local inside = math.min(math.max(qx, qy, qz), 0)
    return outside + inside
  end
end

-- SDF for cylinder (base on XY plane, extends along +Z)
local function cylinder_sdf(r, h)
  return function(x, y, z)
    local d_radial = math.sqrt(x*x + y*y) - r
    local d_bottom = -z
    local d_top = z - h
    local d_vertical = math.max(d_bottom, d_top)
    local outside = math.sqrt(
      math.max(d_radial, 0)^2 +
      math.max(d_vertical, 0)^2
    )
    local inside = math.min(math.max(d_radial, d_vertical), 0)
    return outside + inside
  end
end

--- Create a box/cuboid
-- @param w Width (X dimension)
-- @param d Depth (Y dimension), defaults to w
-- @param h Height (Z dimension), defaults to w
-- @return Shape object
function Primitives.box(w, d, h)
  d = d or w
  h = h or w
  local hw, hd, hh = w/2, d/2, h/2

  return Shape(box_sdf(hw, hd, hh),
    {min = {-hw, -hd, -hh}, max = {hw, hd, hh}},
    {primitive = "box", params = {w = w, d = d, h = h}}
  )
end

--- Create a cylinder with base on XY plane, extending along +Z
-- @param r Radius
-- @param h Height
-- @return Shape object
function Primitives.cylinder(r, h)
  return Shape(cylinder_sdf(r, h),
    {min = {-r, -r, 0}, max = {r, r, h}},
    {primitive = "cylinder", params = {r = r, h = h}}
  )
end

return Primitives
