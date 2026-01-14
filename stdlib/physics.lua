-- ScriptCAD Standard Library: Physics
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

--- Electromagnetic frequency domain study
-- @param config {frequencies, ports, ...}
-- @return EM study object
function Physics.electromagnetic(config)
  config = config or {}
  local study = Study("electromagnetic", {
    analysis = config.type or "frequency_domain",  -- frequency_domain, time_domain, eigenfrequency
    frequencies = config.frequencies or {1e9},
    ports = config.ports or {},
    solver = config.solver or "direct",
    formulation = config.formulation or "full_wave",  -- full_wave, quasi_static
  })
  return study
end

--- Electrostatic study
-- @param config Configuration
-- @return Electrostatic study
function Physics.electrostatic(config)
  config = config or {}
  local study = Study("electrostatic", {
    solver = config.solver or "direct",
  })
  return study
end

--- Magnetostatic study
-- @param config Configuration
-- @return Magnetostatic study
function Physics.magnetostatic(config)
  config = config or {}
  local study = Study("magnetostatic", {
    solver = config.solver or "direct",
    nonlinear = config.nonlinear or false,  -- for saturation
  })
  return study
end

--- Thermal steady-state study
-- @param config Configuration
-- @return Thermal study
function Physics.thermal(config)
  config = config or {}
  local study = Study("thermal", {
    analysis = config.type or "steady_state",  -- steady_state, transient
    ambient_temperature = config.ambient or 293.15,  -- Kelvin
  })
  return study
end

--- Thermal transient study
-- @param config {duration, timestep, ...}
-- @return Thermal transient study
function Physics.thermal_transient(config)
  config = config or {}
  local study = Study("thermal", {
    analysis = "transient",
    duration = config.duration or 1.0,
    timestep = config.timestep or 0.01,
    ambient_temperature = config.ambient or 293.15,
  })
  return study
end

--- Structural mechanics study
-- @param config Configuration
-- @return Structural study
function Physics.structural(config)
  config = config or {}
  local study = Study("structural", {
    analysis = config.type or "static",  -- static, eigenfrequency, transient
    large_deformation = config.large_deformation or false,
  })
  return study
end

--- Coupled multiphysics study
-- @param studies Table of studies to couple
-- @param config Coupling configuration
-- @return Multiphysics study
function Physics.multiphysics(studies, config)
  config = config or {}
  local study = Study("multiphysics", {
    studies = studies,
    coupling = config.coupling or "sequential",  -- sequential, fully_coupled
    iterations = config.iterations or 10,
    tolerance = config.tolerance or 1e-6,
  })
  return study
end

--- Define a port for S-parameter extraction
-- @param positive Positive terminal shape/surface
-- @param negative Negative terminal shape/surface
-- @param config {impedance, ...}
-- @return Port object
function Physics.port(positive, negative, config)
  config = config or {}
  return {
    _type = "port",
    positive = positive,
    negative = negative,
    impedance = config.impedance or 50,
    type = config.type or "lumped",  -- lumped, wave
  }
end

--- Define a current source
-- @param positive Positive terminal
-- @param negative Negative terminal
-- @param value Current value (A) or function
-- @return Current source object
function Physics.current_source(positive, negative, value)
  return {
    _type = "current_source",
    positive = positive,
    negative = negative,
    value = value,
  }
end

--- Define a voltage source
-- @param positive Positive terminal
-- @param negative Negative terminal
-- @param value Voltage value (V) or function
-- @return Voltage source object
function Physics.voltage_source(positive, negative, value)
  return {
    _type = "voltage_source",
    positive = positive,
    negative = negative,
    value = value,
  }
end

--- Define a heat source
-- @param domain Shape or region
-- @param power Power in Watts or W/m³
-- @param config {type: "total" or "density"}
-- @return Heat source object
function Physics.heat_source(domain, power, config)
  config = config or {}
  return {
    _type = "heat_source",
    domain = domain,
    power = power,
    power_type = config.type or "total",  -- total (W) or density (W/m³)
  }
end

--- Define a fixed constraint
-- @param surfaces Surfaces to constrain
-- @return Constraint object
function Physics.fixed(surfaces)
  return {
    _type = "constraint",
    constraint_type = "fixed",
    surfaces = surfaces,
  }
end

--- Define an applied force
-- @param surfaces Surfaces to apply force to
-- @param force {fx, fy, fz} in Newtons
-- @return Force object
function Physics.force(surfaces, force)
  return {
    _type = "force",
    surfaces = surfaces,
    force = force,
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
electromagnetic = Physics.electromagnetic
electrostatic = Physics.electrostatic
magnetostatic = Physics.magnetostatic
thermal = Physics.thermal
thermal_transient = Physics.thermal_transient
structural = Physics.structural
multiphysics = Physics.multiphysics
port = Physics.port
current_source = Physics.current_source
voltage_source = Physics.voltage_source
heat_source = Physics.heat_source
fixed = Physics.fixed
force = Physics.force
linspace = Physics.linspace
logspace = Physics.logspace

return Physics
