-- ScriptCAD Standard Library: Instruments
-- Virtual measurement and visualization instruments

local Instruments = {}

-- Registry of active instruments
Instruments._active = {}

--- Base instrument constructor
local function Instrument(type_name, position, config)
  local inst = {
    _type = "instrument",
    _instrument_type = type_name,
    _position = position or {0, 0, 0},
    _config = config or {},
    _data = {},
    _visible = true,
  }

  setmetatable(inst, {__index = {
    at = function(self, x, y, z)
      self._position = {x, y, z}
      return self
    end,

    configure = function(self, cfg)
      for k, v in pairs(cfg) do
        self._config[k] = v
      end
      return self
    end,

    show = function(self)
      self._visible = true
      return self
    end,

    hide = function(self)
      self._visible = false
      return self
    end,

    serialize = function(self)
      return {
        type = "instrument",
        instrument_type = self._instrument_type,
        position = self._position,
        config = self._config,
        visible = self._visible
      }
    end
  }})

  table.insert(Instruments._active, inst)
  return inst
end

--- Line or volume probe for field values
-- @param name Probe identifier
-- @param config {type, line, volume, points, component, export}
-- @return Probe instrument
function Instruments.Probe(name, config)
  config = config or {}
  local probe = Instrument("probe", config.position or {0, 0, 0}, {
    name = name,
    type = config.type or "B_field",
    line = config.line,
    volume = config.volume,
    points = config.points or 51,
    component = config.component or "magnitude",
    statistics = config.statistics,
    export = config.export,
  })
  probe._name = name
  return probe
end

--- Gauss meter for magnetic field measurement at a point
-- Backend: field.rs computes B at probe location for Helmholtz coils
-- @param position {x, y, z} position
-- @param config {range, component, label}
-- @return GaussMeter instrument
function Instruments.GaussMeter(position, config)
  config = config or {}
  local meter = Instrument("gaussmeter", position, {
    range = config.range or "mT",
    component = config.component or "magnitude",
    label = config.label,
  })
  return meter
end

--- Magnetic field plane visualization
-- Backend: field.rs generates colormap and arrow data for XZ plane
-- Limitation: Only XZ plane at Y=0 is implemented
-- @param plane "XZ" (only supported value)
-- @param offset Distance from origin (ignored, always Y=0)
-- @param config {quantity, style, resolution, color_map}
-- @return MagneticFieldPlane instrument
function Instruments.MagneticFieldPlane(plane, offset, config)
  config = config or {}
  local field_plane = Instrument("field_plane", {0, 0, 0}, {
    plane = "XZ",
    offset = 0,
    quantity = config.quantity or "B",
    style = config.style or "arrows",
    resolution = config.resolution or 20,
    color_map = "jet",
  })
  return field_plane
end

--- Get all active instruments
function Instruments.get_active()
  return Instruments._active
end

--- Clear all instruments
function Instruments.clear()
  Instruments._active = {}
end

--- Serialize all instruments for renderer
function Instruments.serialize_all()
  local result = {}
  for i, inst in ipairs(Instruments._active) do
    result[i] = inst:serialize()
  end
  return result
end

-- Export shortcuts for global use
Probe = Instruments.Probe
GaussMeter = Instruments.GaussMeter
MagneticFieldPlane = Instruments.MagneticFieldPlane

return Instruments
