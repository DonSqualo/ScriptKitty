-- ScriptCAD Standard Library: Circuits
-- 2D circuit diagram components for ultrasound electronics

local Circuits = {}

local function make_component_metatable()
  return {__index = {
    at = function(self, x, z)
      self._position = {x, z}
      return self
    end,

    serialize = function(self)
      return {
        type = "circuit_component",
        component = self._component,
        config = self._config,
        position = self._position,
      }
    end
  }}
end

function Circuits.SignalGenerator(config)
  config = config or {}
  local comp = {
    _type = "circuit_component",
    _component = "signal_generator",
    _config = {
      frequency = config.frequency or 1e6,
      amplitude = config.amplitude or 1.0,
    },
    _position = {0, 0},
  }
  setmetatable(comp, make_component_metatable())
  return comp
end

function Circuits.Amplifier(config)
  config = config or {}
  local comp = {
    _type = "circuit_component",
    _component = "amplifier",
    _config = {
      gain = config.gain or 10,
    },
    _position = {0, 0},
  }
  setmetatable(comp, make_component_metatable())
  return comp
end

function Circuits.MatchingNetwork(config)
  config = config or {}
  local comp = {
    _type = "circuit_component",
    _component = "matching_network",
    _config = {
      transducer_impedance_real = config.impedance_real or 50,
      transducer_impedance_imag = config.impedance_imag or 0,
      frequency = config.frequency or 1e6,
    },
    _position = {0, 0},
  }
  setmetatable(comp, make_component_metatable())
  return comp
end

function Circuits.TransducerLoad(config)
  config = config or {}
  local comp = {
    _type = "circuit_component",
    _component = "transducer",
    _config = {
      impedance_real = config.impedance_real or 50,
      impedance_imag = config.impedance_imag or 0,
    },
    _position = {0, 0},
  }
  setmetatable(comp, make_component_metatable())
  return comp
end

function Circuits.Circuit(config)
  config = config or {}
  local circuit = {
    _type = "circuit",
    _components = config.components or {},
    _size = config.size or {300, 80},
  }

  setmetatable(circuit, {__index = {
    size = function(self, w, h)
      self._size = {w, h}
      return self
    end,

    serialize = function(self)
      local components_serialized = {}
      for i, comp in ipairs(self._components) do
        if comp.serialize then
          components_serialized[i] = comp:serialize()
        end
      end
      return {
        type = "circuit",
        components = components_serialized,
        size = self._size,
      }
    end
  }})

  return circuit
end

function SignalGenerator(config)
  return Circuits.SignalGenerator(config)
end

function Amplifier(config)
  return Circuits.Amplifier(config)
end

function MatchingNetwork(config)
  return Circuits.MatchingNetwork(config)
end

function TransducerLoad(config)
  return Circuits.TransducerLoad(config)
end

function Circuit(config)
  return Circuits.Circuit(config)
end

return Circuits
