-- Mittens Standard Library: Circuits
-- Circuit diagram components for ultrasound drive electronics
-- Backend: server/src/circuit.rs generates SVG schematic overlay

local Circuits = {}

--- Create a signal generator component
-- Represents RF source with sine wave symbol
-- @param config {frequency, amplitude}
-- @return Circuit component
function Circuits.SignalGenerator(config)
  config = config or {}
  return {
    type = "circuit_component",
    component = "signal_generator",
    config = {
      frequency = config.frequency or 1e6,
      amplitude = config.amplitude or 1.0,
    },
    position = {0, 0},
  }
end

--- Create an amplifier component
-- Triangle symbol representing power amplifier
-- @param config {gain}
-- @return Circuit component
function Circuits.Amplifier(config)
  config = config or {}
  return {
    type = "circuit_component",
    component = "amplifier",
    config = {
      gain = config.gain or 10,
    },
    position = {0, 0},
  }
end

--- Create a matching network component
-- L-network: series inductor + shunt capacitor
-- Calculates component values from load impedance:
--   L = |X_load| / omega
--   C = 1 / (omega * R_load)
-- @param config {impedance_real, impedance_imag, frequency}
-- @return Circuit component
function Circuits.MatchingNetwork(config)
  config = config or {}
  return {
    type = "circuit_component",
    component = "matching_network",
    config = {
      impedance_real = config.impedance_real or 50,
      impedance_imag = config.impedance_imag or 0,
      frequency = config.frequency or 1e6,
    },
    position = {0, 0},
  }
end

--- Create a transducer load component
-- Rectangle with diagonal line (piezo symbol) connected to ground
-- @param config {impedance_real, impedance_imag}
-- @return Circuit component
function Circuits.TransducerLoad(config)
  config = config or {}
  return {
    type = "circuit_component",
    component = "transducer_load",
    config = {
      impedance_real = config.impedance_real or 50,
      impedance_imag = config.impedance_imag or 0,
    },
    position = {0, 0},
  }
end

--- Create a circuit diagram from components
-- Lays out components horizontally with wires and ground rail
-- Backend pattern-matches for _circuit_data global
-- @param config {components, size}
-- @return Circuit diagram specification
function Circuits.Circuit(config)
  config = config or {}
  local components = config.components or {}
  local size = config.size or {400, 100}

  -- Store in global for backend pattern matching
  _circuit_data = {
    type = "circuit",
    components = components,
    size = size,
  }

  return _circuit_data
end

return Circuits
