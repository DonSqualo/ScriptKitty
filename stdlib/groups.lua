-- ScriptCAD Standard Library: Groups
-- Hierarchical grouping of shapes

local Groups = {}

-- Recursive collector for flatten operation
local function collect_children(grp, result)
  for _, child in ipairs(grp._children) do
    if child._type == "group" then
      collect_children(child, result)
    else
      table.insert(result, child)
    end
  end
end

--- Create a named group of shapes
-- @param name Group name (for visibility control, selection)
-- @param children Table of shapes/groups
-- @return Group object
function Groups.group(name, children)
  if type(name) == "table" then
    -- Called as group({...}) without name
    children = name
    name = "unnamed_group"
  end

  local grp = {
    _type = "group",
    _name = name,
    _children = children or {},
    _transform = {position = {0, 0, 0}, rotation = {0, 0, 0}, scale = {1, 1, 1}},
    _visible = true,
    _locked = false,
  }

  -- Calculate combined bounds
  local min_bounds = {math.huge, math.huge, math.huge}
  local max_bounds = {-math.huge, -math.huge, -math.huge}

  for _, child in ipairs(grp._children) do
    if child._bounds then
      for i = 1, 3 do
        min_bounds[i] = math.min(min_bounds[i], child._bounds.min[i])
        max_bounds[i] = math.max(max_bounds[i], child._bounds.max[i])
      end
    end
  end

  grp._bounds = {min = min_bounds, max = max_bounds}

  setmetatable(grp, {__index = {
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

    hide = function(self)
      self._visible = false
      return self
    end,

    show = function(self)
      self._visible = true
      return self
    end,

    lock = function(self)
      self._locked = true
      return self
    end,

    unlock = function(self)
      self._locked = false
      return self
    end,

    add = function(self, child)
      table.insert(self._children, child)
      -- Update bounds
      if child._bounds then
        for i = 1, 3 do
          self._bounds.min[i] = math.min(self._bounds.min[i], child._bounds.min[i])
          self._bounds.max[i] = math.max(self._bounds.max[i], child._bounds.max[i])
        end
      end
      return self
    end,

    remove = function(self, child_or_name)
      for i, c in ipairs(self._children) do
        if c == child_or_name or c._name == child_or_name then
          table.remove(self._children, i)
          break
        end
      end
      return self
    end,

    find = function(self, name)
      for _, child in ipairs(self._children) do
        if child._name == name then
          return child
        end
        if child._type == "group" then
          local found = child:find(name)
          if found then return found end
        end
      end
      return nil
    end,

    flatten = function(self)
      local all = {}
      collect_children(self, all)
      return all
    end,

    serialize = function(self)
      local children_serialized = {}
      for i, child in ipairs(self._children) do
        if child.serialize then
          children_serialized[i] = child:serialize()
        end
      end
      return {
        type = "group",
        name = self._name,
        children = children_serialized,
        transform = self._transform,
        visible = self._visible,
        locked = self._locked
      }
    end
  }})

  return grp
end

--- Create an assembly (top-level group with metadata)
-- @param name Assembly name
-- @param children Child shapes/groups
-- @param metadata Optional metadata (author, version, etc.)
-- @return Assembly object
function Groups.assembly(name, children, metadata)
  local asm = Groups.group(name, children)
  asm._type = "assembly"
  asm._metadata = metadata or {}
  asm._metadata.created = os.date("%Y-%m-%d %H:%M:%S")
  return asm
end

--- Create a component (reusable part)
-- @param name Component name
-- @param children Child shapes
-- @return Component object
function Groups.component(name, children)
  local comp = Groups.group(name, children)
  comp._type = "component"
  comp._instances = {}

  -- Add instance method
  getmetatable(comp).__index.instance = function(self)
    local inst = {
      _type = "instance",
      _component = self._name,
      _transform = {position = {0, 0, 0}, rotation = {0, 0, 0}, scale = {1, 1, 1}},
    }
    setmetatable(inst, {__index = {
      at = function(s, x, y, z)
        s._transform.position = {x, y, z}
        return s
      end,
      rotate = function(s, rx, ry, rz)
        s._transform.rotation = {rx, ry, rz}
        return s
      end,
      scale = function(s, sx, sy, sz)
        sy = sy or sx
        sz = sz or sx
        s._transform.scale = {sx, sy, sz}
        return s
      end,
      serialize = function(s)
        return {
          type = "instance",
          component = s._component,
          transform = s._transform
        }
      end
    }})
    table.insert(self._instances, inst)
    return inst
  end

  return comp
end

-- Shortcut for global use
function group(name, children)
  return Groups.group(name, children)
end

function assembly(name, children, metadata)
  return Groups.assembly(name, children, metadata)
end

function component(name, children)
  return Groups.component(name, children)
end

return Groups
