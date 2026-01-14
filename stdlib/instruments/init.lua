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

--- Electric field probe
-- @param position {x, y, z} position
-- @param config Configuration table
-- @return Probe instrument
function Instruments.Probe(name, config)
  config = config or {}
  local probe = Instrument("probe", config.position or {0, 0, 0}, {
    name = name,
    type = config.type or "E_field",  -- E_field, H_field, voltage, current
    component = config.component or "magnitude",  -- x, y, z, magnitude
    log_scale = config.log_scale or false,
  })
  probe._name = name
  return probe
end

--- Oscilloscope virtual instrument
-- @param position {x, y, z} position
-- @param config {range, timebase, channels}
-- @return Oscilloscope instrument
function Instruments.Oscilloscope(position, config)
  config = config or {}
  local scope = Instrument("oscilloscope", position, {
    voltage_range = config.range or 5,        -- ±V
    timebase = config.timebase or 0.001,       -- seconds/div
    channels = config.channels or 1,
    trigger_level = config.trigger or 0,
    trigger_edge = config.edge or "rising",
  })
  return scope
end

--- Gauss meter for magnetic field measurement
-- @param position {x, y, z} position
-- @param config {range, component}
-- @return GaussMeter instrument
function Instruments.GaussMeter(position, config)
  config = config or {}
  local meter = Instrument("gaussmeter", position, {
    range = config.range or "mT",  -- T, mT, uT, G
    component = config.component or "magnitude",  -- x, y, z, magnitude
    averaging = config.averaging or 1,
  })
  return meter
end

--- Thermometer for temperature measurement
-- @param position {x, y, z} position
-- @param config {unit, range}
-- @return Thermometer instrument
function Instruments.Thermometer(position, config)
  config = config or {}
  local therm = Instrument("thermometer", position, {
    unit = config.unit or "C",  -- C, K, F
    range_min = config.range_min or -50,
    range_max = config.range_max or 200,
  })
  return therm
end

--- Force sensor
-- @param position {x, y, z} position
-- @param direction {dx, dy, dz} measurement direction
-- @param config Configuration
-- @return ForceSensor instrument
function Instruments.ForceSensor(position, direction, config)
  config = config or {}
  local sensor = Instrument("force_sensor", position, {
    direction = direction or {0, 0, 1},
    range = config.range or 100,  -- Newtons
    unit = config.unit or "N",
  })
  return sensor
end

--- Magnetic field plane visualization
-- @param plane "XY", "XZ", or "YZ"
-- @param offset Distance from origin along normal
-- @param config {quantity, style, scale, resolution}
-- @return FieldPlane instrument
function Instruments.MagneticFieldPlane(plane, offset, config)
  config = config or {}
  local field_plane = Instrument("field_plane", {0, 0, 0}, {
    plane = plane or "XZ",
    offset = offset or 0,
    quantity = config.quantity or "H",  -- H, B
    style = config.style or "arrows",   -- arrows, streamlines, colormap
    scale = config.scale or "linear",   -- linear, log
    resolution = config.resolution or 20,
    color_map = config.color_map or "viridis",
    arrow_scale = config.arrow_scale or 1.0,
  })
  return field_plane
end

--- Electric field plane visualization
-- @param plane "XY", "XZ", or "YZ"
-- @param offset Distance from origin
-- @param config Configuration
-- @return FieldPlane instrument
function Instruments.ElectricFieldPlane(plane, offset, config)
  config = config or {}
  local field_plane = Instrument("field_plane", {0, 0, 0}, {
    plane = plane or "XZ",
    offset = offset or 0,
    quantity = config.quantity or "E",
    style = config.style or "arrows",
    scale = config.scale or "linear",
    resolution = config.resolution or 20,
    color_map = config.color_map or "plasma",
    arrow_scale = config.arrow_scale or 1.0,
  })
  return field_plane
end

--- Temperature field visualization
-- @param plane "XY", "XZ", or "YZ"
-- @param offset Distance from origin
-- @param config Configuration
-- @return FieldPlane instrument
function Instruments.TemperaturePlane(plane, offset, config)
  config = config or {}
  local field_plane = Instrument("field_plane", {0, 0, 0}, {
    plane = plane or "XZ",
    offset = offset or 0,
    quantity = "temperature",
    style = "colormap",
    scale = config.scale or "linear",
    resolution = config.resolution or 50,
    color_map = config.color_map or "inferno",
    range_min = config.range_min,
    range_max = config.range_max,
  })
  return field_plane
end

--- Acoustic pressure field visualization
-- @param plane "XY", "XZ", or "YZ"
-- @param offset Distance from origin along normal
-- @param config {quantity, clip_to_domain, bounds, resolution, color_map}
-- @return AcousticPressurePlane instrument
function Instruments.AcousticPressurePlane(plane, offset, config)
  config = config or {}
  local field_plane = Instrument("acoustic_field_plane", {0, 0, 0}, {
    plane = plane or "XZ",
    offset = offset or 0,
    quantity = config.quantity or "pressure",
    clip_to_domain = config.clip_to_domain,
    bounds = config.bounds,
    style = "colormap",
    scale = config.scale or "linear",
    resolution = config.resolution or 50,
    color_map = config.color_map or "jet",
    range_min = config.range_min,
    range_max = config.range_max,
    show_nodal_lines = config.show_nodal_lines or false,
    render_in_scene = config.render_in_scene ~= false,
  })
  return field_plane
end

--- Acoustic energy density field visualization
-- @param plane "XY", "XZ", or "YZ"
-- @param offset Distance from origin along normal
-- @param config {clip_to_domain, resolution, color_map}
-- @return AcousticEnergyPlane instrument
function Instruments.AcousticEnergyPlane(plane, offset, config)
  config = config or {}
  local field_plane = Instrument("acoustic_field_plane", {0, 0, 0}, {
    plane = plane or "XZ",
    offset = offset or 0,
    quantity = "energy_density",
    clip_to_domain = config.clip_to_domain,
    style = "colormap",
    scale = config.scale or "linear",
    resolution = config.resolution or 50,
    color_map = config.color_map or "hot",
    range_min = config.range_min,
    range_max = config.range_max,
  })
  return field_plane
end

--- Acoustic intensity field visualization
-- @param plane "XY", "XZ", or "YZ"
-- @param offset Distance from origin along normal
-- @param config {clip_to_domain, resolution, color_map}
-- @return AcousticIntensityPlane instrument
function Instruments.AcousticIntensityPlane(plane, offset, config)
  config = config or {}
  local field_plane = Instrument("acoustic_field_plane", {0, 0, 0}, {
    plane = plane or "XZ",
    offset = offset or 0,
    quantity = "intensity",
    clip_to_domain = config.clip_to_domain,
    style = "colormap",
    scale = config.scale or "log",
    resolution = config.resolution or 50,
    color_map = config.color_map or "viridis",
    range_min = config.range_min,
    range_max = config.range_max,
  })
  return field_plane
end

--- Hydrophone probe for acoustic pressure measurement
-- @param position {x, y, z} position
-- @param config {bandwidth, sensitivity}
-- @return Hydrophone instrument
function Instruments.Hydrophone(position, config)
  config = config or {}
  local probe = Instrument("hydrophone", position, {
    bandwidth = config.bandwidth or 10e6,
    sensitivity = config.sensitivity or -220,
    component = config.component or "magnitude",
    output_unit = config.unit or "Pa",
  })
  return probe
end

--- Streamlines visualization for vector fields
-- @param config {field, seeds, length, color}
-- @return Streamlines instrument
function Instruments.Streamlines(config)
  config = config or {}
  local streams = Instrument("streamlines", {0, 0, 0}, {
    field = config.field or "H",  -- H, B, E, velocity
    seed_points = config.seeds or "grid",  -- grid, random, surface
    seed_count = config.count or 100,
    max_length = config.length or 100,
    step_size = config.step or 0.5,
    color_by = config.color_by or "magnitude",  -- magnitude, direction, constant
    color = config.color or {0.2, 0.6, 1.0, 1.0},
  })
  return streams
end

--- Isosurface visualization
-- @param field Field name to visualize
-- @param value Iso value
-- @param config Configuration
-- @return Isosurface instrument
function Instruments.Isosurface(field, value, config)
  config = config or {}
  local iso = Instrument("isosurface", {0, 0, 0}, {
    field = field,
    value = value,
    color = config.color or {0.5, 0.5, 1.0, 0.5},
    opacity = config.opacity or 0.5,
  })
  return iso
end

--- S-parameter output configuration
-- @param study Study object reference
-- @param config {plot, export}
-- @return SParams instrument
function Instruments.SParams(study, config)
  config = config or {}
  local sparams = Instrument("s_params", {0, 0, 0}, {
    study = study,
    plot = config.plot or {"S11_dB"},
    export = config.export,
    format = config.format or "touchstone",  -- touchstone, csv
  })
  return sparams
end

--- Get all active instruments
-- @return Table of active instruments
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
Oscilloscope = Instruments.Oscilloscope
GaussMeter = Instruments.GaussMeter
Thermometer = Instruments.Thermometer
ForceSensor = Instruments.ForceSensor
MagneticFieldPlane = Instruments.MagneticFieldPlane
ElectricFieldPlane = Instruments.ElectricFieldPlane
TemperaturePlane = Instruments.TemperaturePlane
AcousticPressurePlane = Instruments.AcousticPressurePlane
AcousticEnergyPlane = Instruments.AcousticEnergyPlane
AcousticIntensityPlane = Instruments.AcousticIntensityPlane
Hydrophone = Instruments.Hydrophone
Streamlines = Instruments.Streamlines
Isosurface = Instruments.Isosurface
SParams = Instruments.SParams

return Instruments
