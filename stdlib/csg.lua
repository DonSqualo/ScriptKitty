-- ScriptCAD Standard Library: CSG (Constructive Solid Geometry)
-- Boolean operations on shapes

local CSG = {}

-- SDF factory for union operation
local function make_union_sdf(shapes)
  return function(x, y, z)
    local min_d = math.huge
    for _, shape in ipairs(shapes) do
      local d = shape:eval(x, y, z)
      if d < min_d then min_d = d end
    end
    return min_d
  end
end

-- SDF factory for difference operation
local function make_difference_sdf(base, cutters)
  return function(x, y, z)
    local d = base:eval(x, y, z)
    for _, cutter in ipairs(cutters) do
      local cut_d = -cutter:eval(x, y, z)
      if cut_d > d then d = cut_d end
    end
    return d
  end
end

-- SDF factory for intersection operation
local function make_intersect_sdf(shapes)
  return function(x, y, z)
    local max_d = -math.huge
    for _, shape in ipairs(shapes) do
      local d = shape:eval(x, y, z)
      if d > max_d then max_d = d end
    end
    return max_d
  end
end

-- Smooth minimum helper
local function smooth_min(a, b, k)
  local h = math.max(k - math.abs(a - b), 0) / k
  return math.min(a, b) - h * h * k * 0.25
end

-- SDF factory for smooth union operation
local function make_smooth_union_sdf(shapes, k)
  return function(x, y, z)
    local d = shapes[1]:eval(x, y, z)
    for i = 2, #shapes do
      d = smooth_min(d, shapes[i]:eval(x, y, z), k)
    end
    return d
  end
end

-- SDF factory for shell operation
local function make_shell_sdf(shape, thickness)
  return function(x, y, z)
    return math.abs(shape:eval(x, y, z)) - thickness / 2
  end
end

-- Shared metatable methods for CSG results
local function make_csg_metatable()
  return {__index = {
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
    name = function(self, n)
      self._name = n
      return self
    end,
    eval = function(self, x, y, z)
      return self._sdf(x, y, z)
    end,
    serialize = function(self)
      local children_serialized = {}
      for i, child in ipairs(self._children) do
        children_serialized[i] = child:serialize()
      end
      return {
        type = "csg",
        operation = self._operation,
        blend = self._blend,
        thickness = self._thickness,
        children = children_serialized,
        transform = self._transform,
        material = self._material,
        name = self._name
      }
    end
  }}
end

--- Union of multiple shapes (additive)
-- @param ... Shapes to combine
-- @return Combined shape
function CSG.union(...)
  local shapes = {...}

  -- Handle table argument
  if #shapes == 1 and type(shapes[1]) == "table" and shapes[1]._type ~= "shape" then
    shapes = shapes[1]
  end

  if #shapes == 0 then
    error("union requires at least one shape")
  end

  if #shapes == 1 then
    return shapes[1]
  end

  -- Combined bounds
  local min_bounds = {math.huge, math.huge, math.huge}
  local max_bounds = {-math.huge, -math.huge, -math.huge}

  for _, shape in ipairs(shapes) do
    for i = 1, 3 do
      min_bounds[i] = math.min(min_bounds[i], shape._bounds.min[i])
      max_bounds[i] = math.max(max_bounds[i], shape._bounds.max[i])
    end
  end

  local result = {
    _type = "csg",
    _operation = "union",
    _children = shapes,
    _sdf = make_union_sdf(shapes),
    _bounds = {min = min_bounds, max = max_bounds},
    _transform = {position = {0, 0, 0}, rotation = {0, 0, 0}, scale = {1, 1, 1}},
    _material = nil,
  }

  setmetatable(result, make_csg_metatable())
  return result
end

--- Difference of shapes (subtractive)
-- First shape minus all subsequent shapes
-- @param base Base shape
-- @param ... Shapes to subtract
-- @return Resulting shape
function CSG.difference(base, ...)
  local cutters = {...}

  if #cutters == 0 then
    return base
  end

  local result = {
    _type = "csg",
    _operation = "difference",
    _children = {base, table.unpack(cutters)},
    _sdf = make_difference_sdf(base, cutters),
    _bounds = base._bounds,
    _transform = {position = {0, 0, 0}, rotation = {0, 0, 0}, scale = {1, 1, 1}},
    _material = nil,
  }

  setmetatable(result, make_csg_metatable())
  return result
end

--- Intersection of shapes
-- @param ... Shapes to intersect
-- @return Intersection shape
function CSG.intersect(...)
  local shapes = {...}

  if #shapes == 1 and type(shapes[1]) == "table" and shapes[1]._type ~= "shape" then
    shapes = shapes[1]
  end

  if #shapes == 0 then
    error("intersect requires at least one shape")
  end

  if #shapes == 1 then
    return shapes[1]
  end

  -- Intersection bounds (smallest box containing intersection)
  local min_bounds = {-math.huge, -math.huge, -math.huge}
  local max_bounds = {math.huge, math.huge, math.huge}

  for _, shape in ipairs(shapes) do
    for i = 1, 3 do
      min_bounds[i] = math.max(min_bounds[i], shape._bounds.min[i])
      max_bounds[i] = math.min(max_bounds[i], shape._bounds.max[i])
    end
  end

  local result = {
    _type = "csg",
    _operation = "intersect",
    _children = shapes,
    _sdf = make_intersect_sdf(shapes),
    _bounds = {min = min_bounds, max = max_bounds},
    _transform = {position = {0, 0, 0}, rotation = {0, 0, 0}, scale = {1, 1, 1}},
    _material = nil,
  }

  setmetatable(result, make_csg_metatable())
  return result
end

--- Smooth union with blending
-- @param k Blend factor (larger = smoother)
-- @param ... Shapes to blend
-- @return Blended shape
function CSG.smooth_union(k, ...)
  local shapes = {...}

  local min_bounds = {math.huge, math.huge, math.huge}
  local max_bounds = {-math.huge, -math.huge, -math.huge}

  for _, shape in ipairs(shapes) do
    for i = 1, 3 do
      min_bounds[i] = math.min(min_bounds[i], shape._bounds.min[i]) - k
      max_bounds[i] = math.max(max_bounds[i], shape._bounds.max[i]) + k
    end
  end

  local result = {
    _type = "csg",
    _operation = "smooth_union",
    _blend = k,
    _children = shapes,
    _sdf = make_smooth_union_sdf(shapes, k),
    _bounds = {min = min_bounds, max = max_bounds},
    _transform = {position = {0, 0, 0}, rotation = {0, 0, 0}, scale = {1, 1, 1}},
    _material = nil,
  }

  setmetatable(result, make_csg_metatable())
  return result
end

--- Shell/hollow a shape
-- @param shape Shape to hollow
-- @param thickness Wall thickness
-- @return Hollow shape
function CSG.shell(shape, thickness)
  local result = {
    _type = "csg",
    _operation = "shell",
    _thickness = thickness,
    _children = {shape},
    _sdf = make_shell_sdf(shape, thickness),
    _bounds = {
      min = {shape._bounds.min[1] - thickness, shape._bounds.min[2] - thickness, shape._bounds.min[3] - thickness},
      max = {shape._bounds.max[1] + thickness, shape._bounds.max[2] + thickness, shape._bounds.max[3] + thickness}
    },
    _transform = {position = {0, 0, 0}, rotation = {0, 0, 0}, scale = {1, 1, 1}},
    _material = nil,
  }

  setmetatable(result, make_csg_metatable())
  return result
end

return CSG
