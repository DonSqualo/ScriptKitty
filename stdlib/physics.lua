-- Mittens Standard Library: Physics
-- Physics simulation setup and configuration

local Physics = {}

-- Registry of studies
Physics._studies = {}

--- Base study constructor
local function Study(type_name, config)
  local study = {
    _type = "study",
    _study_type = type_name,
    _config = config or {},
    _domains = {},
    _boundary_conditions = {},
    _initial_conditions = {},
    _mesh_settings = {},
  }

  setmetatable(study, {__index = {
    domain = function(self, shapes)
      if type(shapes) ~= "table" or shapes._type then
        shapes = {shapes}
      end
      for _, s in ipairs(shapes) do
        table.insert(self._domains, s)
      end
      return self
    end,

    boundary = function(self, name, condition)
      self._boundary_conditions[name] = condition
      return self
    end,

    initial = function(self, name, value)
      self._initial_conditions[name] = value
      return self
    end,

    mesh = function(self, settings)
      for k, v in pairs(settings) do
        self._mesh_settings[k] = v
      end
      return self
    end,

    serialize = function(self)
      return {
        type = "study",
        study_type = self._study_type,
        config = self._config,
        boundary_conditions = self._boundary_conditions,
        initial_conditions = self._initial_conditions,
        mesh_settings = self._mesh_settings,
      }
    end
  }})

  table.insert(Physics._studies, study)
  return study
end

--- Magnetostatic study
-- Backend: field.rs implements Biot-Savart for Helmholtz coil pattern
-- Pattern-matching: Looks for "helmholtz" keyword and coil_mean_radius in config
-- @param config {solver, sources}
-- @return Magnetostatic study
function Physics.magnetostatic(config)
  config = config or {}
  local study = Study("magnetostatic", {
    solver = config.solver or "direct",
    formulation = config.formulation or "vector_potential",
    sources = config.sources or {},
  })
  return study
end

--- Acoustic pressure study
-- Backend: acoustic.rs implements Rayleigh-Sommerfeld for piston transducers
-- Pattern-matching: Looks for Transducer and Medium globals
-- @param config {frequency, transducer, medium, boundaries}
-- @return Acoustic study
function Physics.acoustic(config)
  config = config or {}
  local study = Study("acoustic", {
    analysis = config.analysis or "frequency_domain",
    frequency = config.frequency or 1e6,
    drive_current = config.drive_current or 0.1,
    drive_voltage = config.drive_voltage,
    transducer = config.transducer,
    medium = config.medium,
    boundaries = config.boundaries or {},
  })
  return study
end

--- Acoustic source definition (piezo transducer)
-- @param geometry Transducer geometry reference
-- @param config {frequency, current, voltage, phase}
-- @return Acoustic source object
function Physics.acoustic_source(geometry, config)
  config = config or {}
  return {
    _type = "acoustic_source",
    geometry = geometry,
    frequency = config.frequency or 1e6,
    drive_current = config.drive_current,
    drive_voltage = config.drive_voltage,
    phase = config.phase or 0,
  }
end

--- Acoustic boundary condition
-- @param surface Surface geometry
-- @param config {type, impedance, reflection_coeff}
-- @return Boundary condition object
function Physics.acoustic_boundary(surface, config)
  config = config or {}
  return {
    _type = "acoustic_boundary",
    surface = surface,
    boundary_type = config.type or "impedance",
    impedance = config.impedance,
    reflection_coeff = config.reflection_coeff,
    absorption_coeff = config.absorption_coeff,
  }
end

--- Define a current source for magnetostatic
-- @param geometry Conductor geometry
-- @param config {current, turns, direction}
-- @return Current source object
function Physics.current_source(geometry, config)
  config = config or {}
  return {
    _type = "current_source",
    geometry = geometry,
    current = config.current or 1.0,
    turns = config.turns or 1,
    direction = config.direction or "ccw",
  }
end

--- Generate a linearly spaced array
-- @param start Start value
-- @param stop End value
-- @param count Number of points
-- @return Array of values
function Physics.linspace(start, stop, count)
  local result = {}
  local step = (stop - start) / (count - 1)
  for i = 0, count - 1 do
    result[i + 1] = start + i * step
  end
  return result
end

--- Generate a logarithmically spaced array
-- @param start Start value (will use log10)
-- @param stop End value
-- @param count Number of points
-- @return Array of values
function Physics.logspace(start, stop, count)
  local log_start = math.log10(start)
  local log_stop = math.log10(stop)
  local result = {}
  local step = (log_stop - log_start) / (count - 1)
  for i = 0, count - 1 do
    result[i + 1] = 10 ^ (log_start + i * step)
  end
  return result
end

--- Get all studies
function Physics.get_studies()
  return Physics._studies
end

--- Clear all studies
function Physics.clear()
  Physics._studies = {}
end

-- Export shortcuts
magnetostatic = Physics.magnetostatic
acoustic = Physics.acoustic
acoustic_source = Physics.acoustic_source
acoustic_boundary = Physics.acoustic_boundary
current_source = Physics.current_source
linspace = Physics.linspace
logspace = Physics.logspace

return Physics
