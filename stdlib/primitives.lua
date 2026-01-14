-- ScriptCAD Standard Library: Primitives
-- Basic geometric primitives using Signed Distance Functions (SDF)

local Primitives = {}

-- Create a shape object with SDF and metadata
local function Shape(sdf_func, bounds, metadata)
  local shape = {
    _type = "shape",
    _sdf = sdf_func,
    _bounds = bounds or {min = {-1e6, -1e6, -1e6}, max = {1e6, 1e6, 1e6}},
    _ops = {},
    _material = nil,
    _metadata = metadata or {}
  }

  setmetatable(shape, {__index = {
    at = function(self, x, y, z)
      table.insert(self._ops, {op = "translate", x = x, y = y, z = z})
      return self
    end,

    rotate = function(self, rx, ry, rz)
      table.insert(self._ops, {op = "rotate", x = rx, y = ry, z = rz})
      return self
    end,

    scale = function(self, sx, sy, sz)
      sy = sy or sx
      sz = sz or sx
      table.insert(self._ops, {op = "scale", x = sx, y = sy, z = sz})
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

    center = function(self, cx, cy, cz)
      local bounds = self._bounds
      local dx = cx and -((bounds.min[1] + bounds.max[1]) / 2) or 0
      local dy = cy and -((bounds.min[2] + bounds.max[2]) / 2) or 0
      local dz = cz and -((bounds.min[3] + bounds.max[3]) / 2) or 0
      table.insert(self._ops, {op = "translate", x = dx, y = dy, z = dz})
      return self
    end,

    centerXY = function(self)
      return self:center(true, true, false)
    end,

    eval = function(self, x, y, z)
      return self._sdf(x, y, z)
    end,

    serialize = function(self)
      return {
        type = self._metadata.primitive,
        params = self._metadata.params,
        ops = self._ops,
        material = self._material,
        color = self._color,
        name = self._name
      }
    end
  }})

  return shape
end

-- SDF for box (corner at origin, extends to +w, +d, +h)
local function box_sdf(w, d, h)
  return function(x, y, z)
    local qx = math.max(-x, x - w)
    local qy = math.max(-y, y - d)
    local qz = math.max(-z, z - h)
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

--- Create a box/cuboid (corner at origin)
-- @param w Width (X dimension)
-- @param d Depth (Y dimension), defaults to w
-- @param h Height (Z dimension), defaults to w
-- @return Shape object
function Primitives.box(w, d, h)
  d = d or w
  h = h or w

  return Shape(box_sdf(w, d, h),
    {min = {0, 0, 0}, max = {w, d, h}},
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

--- Create a colormap visualization plane
-- @param config {plane, bounds, resolution, colormap_data, color_map}
-- @return Visualization plane object
function Primitives.colormap_plane(config)
  config = config or {}
  local plane = config.plane or "XZ"
  local bounds = config.bounds or {x_min = -10, x_max = 10, z_min = 0, z_max = 10}
  local resolution = config.resolution or 50
  local offset = config.offset or 0

  local viz = {
    _type = "visualization",
    _viz_type = "colormap_plane",
    _plane = plane,
    _bounds = bounds,
    _offset = offset,
    _resolution = resolution,
    _colormap_data = config.colormap_data,
    _color_map = config.color_map or "jet",
    _opacity = config.opacity or 0.9,
  }

  setmetatable(viz, {__index = {
    at = function(self, x, y, z)
      self._position = {x, y, z}
      return self
    end,

    opacity = function(self, a)
      self._opacity = a
      return self
    end,

    serialize = function(self)
      return {
        type = "colormap_plane",
        plane = self._plane,
        bounds = self._bounds,
        offset = self._offset,
        resolution = self._resolution,
        colormap_data = self._colormap_data,
        color_map = self._color_map,
        opacity = self._opacity,
        position = self._position,
      }
    end
  }})

  return viz
end

return Primitives
