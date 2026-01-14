-- bridge_gap_resonator.lua
-- A bridge gap resonator with coupled resonance coil
--
-- This example demonstrates:
-- - Parametric design with variables
-- - Material assignment with physical properties
-- - CSG operations for complex geometry
-- - Electromagnetic simulation setup
-- - Virtual instruments for field visualization
-- - View configuration with clipping

-- Load standard library
local ScriptCAD = require("stdlib")

-- ============================================================================
-- Configuration Parameters
-- ============================================================================

local config = {
  -- Bridge gap resonator dimensions (mm)
  gap = 2.5,       -- bridge gap width
  bridge_w = 8,    -- bridge conductor width
  bridge_t = 0.5,  -- bridge thickness (copper)
  pad_length = 25, -- pad length on each side

  -- Substrate
  substrate_w = 30,  -- substrate width
  substrate_d = 60,  -- substrate depth
  substrate_h = 9.6, -- PCB substrate thickness

  -- Resonance coil parameters
  coil = {
    turns = 5,
    r_inner = 12,     -- inner radius
    r_outer = 20,     -- outer radius (for spiral)
    pitch = 1.05,     -- vertical spacing per turn
    wire_d = 0.8,     -- wire diameter
    style = "spiral", -- "spiral" (flat) or "solenoid"
    z_offset = -5,    -- distance below substrate
  },

  -- Simulation
  freq_start = 1e9, -- 1 GHz
  freq_stop = 10e9, -- 10 GHz
  freq_points = 101,
}

-- ============================================================================
-- Materials
-- ============================================================================

local copper = material("copper")

local fr4 = material("FR4", {
  permittivity = 4.4,
  loss_tangent = 0.02
})

local air = material("air")

-- ============================================================================
-- Geometry: Tube (base at z = -substrate_h, top at z = 0)
-- ============================================================================

local tube_outer = cylinder(config.substrate_w / 2, config.substrate_h)
local tube_inner = cylinder(config.substrate_w / 2 - 2, config.substrate_h + 1) -- wall thickness = 2mm

local tube = difference(tube_outer, tube_inner)
    :at(0, 0, -config.substrate_h)
    :material(fr4)
    :name("tube")

-- ============================================================================
-- Geometry: Bridge Gap Resonator
-- ============================================================================

-- Left pad
local left_pad = box(config.pad_length, config.bridge_w, config.bridge_t)
    :at(-config.gap / 2 - config.pad_length / 2, 0, config.bridge_t / 2)
    :material(copper)
    :name("left_pad")

-- Right pad
local right_pad = box(config.pad_length, config.bridge_w, config.bridge_t)
    :at(config.gap / 2 + config.pad_length / 2, 0, config.bridge_t / 2)
    :material(copper)
    :name("right_pad")

-- Bridge element (thin strip over gap)
local bridge = box(config.gap + 4, config.bridge_w * 0.6, config.bridge_t * 0.5)
    :at(0, 0, config.bridge_t * 1.25)
    :material(copper)
    :name("bridge")

-- Group the resonator
local resonator = group("resonator", {
  tube,
  left_pad,
  right_pad,
  bridge
})

-- ============================================================================
-- Geometry: Resonance Coil
-- ============================================================================

local coil_z = -config.substrate_h + config.coil.z_offset

-- Main coil
-- local coil = helix({
--       inner_radius = config.coil.r_inner,
--       outer_radius = config.coil.r_outer,
--       turns = config.coil.turns,
--       pitch = config.coil.pitch,
--       wire_diameter = config.coil.wire_d,
--       style = config.coil.style
--     })
--     :at(0, 0, coil_z)
--     :material(copper)
--     :name("coil")

-- Coil connection leads
local lead_length = 15
local lead_r = config.coil.wire_d / 2

local lead_in = cylinder(lead_r, lead_length)
    :at(config.coil.r_outer, 0, coil_z)
    :rotate(0, 90, 0)
    :material(copper)
    :name("lead_in")

local lead_out = cylinder(lead_r, lead_length)
    :at(config.coil.r_inner, 0, coil_z - config.coil.turns * config.coil.pitch)
    :rotate(0, 90, 0)
    :material(copper)
    :name("lead_out")

-- Group the coil assembly
local coil_assembly = group("coil_assembly", {
  -- coil,
  lead_in,
  lead_out
})

-- ============================================================================
-- Full Assembly
-- ============================================================================

local assembly = group("bridge_gap_resonator", {
  resonator,
  coil_assembly
})

-- Register with scene
ScriptCAD.register(assembly)

-- ============================================================================
-- Physics Setup: Electromagnetic Study
-- ============================================================================

local em_study = electromagnetic({
      type = "frequency_domain",
      frequencies = linspace(config.freq_start, config.freq_stop, config.freq_points),
      solver = "direct",
      formulation = "full_wave",

      -- Excitation port across gap
      ports = {
        port(left_pad, right_pad, { impedance = 50 })
      },
    })
    :domain(assembly)
    :boundary("outer", { type = "radiation" }) -- Open boundary for radiation
    :mesh({
      type = "tetrahedral",
      max_element_size = 2,   -- mm
      min_element_size = 0.2, -- mm, refined near gap
      curvature_factor = 0.3,
      refinement_regions = {
        { region = "gap",  size = 0.1 }, -- Extra refinement in gap
        { region = "coil", size = 0.3 }, -- Refinement on coil wire
      }
    })

-- ============================================================================
-- Virtual Instruments
-- ============================================================================

-- Measure E-field in the gap
Probe("E_gap", {
  type = "E_field",
  position = { 0, 0, config.bridge_t / 2 },
  component = "magnitude"
})

-- Measure H-field at coil center
GaussMeter({ 0, 0, coil_z }, {
  range = "mT",
  component = "z"
})

-- Magnetic field visualization on XZ plane through center
MagneticFieldPlane("XZ", 0, {
  quantity = "H",
  style = "arrows",
  scale = "log",
  resolution = 30,
  arrow_scale = 0.8
})

-- Electric field on XY plane just above substrate
ElectricFieldPlane("XY", config.bridge_t, {
  quantity = "E",
  style = "colormap",
  scale = "log",
  resolution = 50,
  color_map = "plasma"
})

-- S-parameter output
SParams(em_study, {
  plot = { "S11_dB", "S11_phase", "S11_smith" },
  export = "results/bridge_gap_s_params.s1p"
})

-- ============================================================================
-- View Configuration
-- ============================================================================

view({
  camera = "isometric",
  distance = 120,

  -- Cross-section to see coil under substrate
  clip = {
    plane = "XZ",
    offset = 0,
    show_caps = true
  },

  -- What to display
  show = {
    "bridge_gap_resonator",
    "MagneticFieldPlane",
    "ElectricFieldPlane"
  },

  -- Make substrate semi-transparent to see coil
  transparency = {
    substrate = 0.3
  },

  -- Visual settings
  theme = "dark",
  -- grid = { show = true, size = 80, divisions = 8 },
  axes = { show = true, size = 15 },

  render = {
    quality = "high",
    shadows = true,
    ambient_occlusion = true
  }
})

-- ============================================================================
-- Exports
-- ============================================================================

-- export_stl("exports/bridge_gap_resonator.stl", assembly)
-- export_step("exports/bridge_gap_resonator.step", assembly)
-- export_gltf("exports/bridge_gap_resonator.glb", assembly, { binary = true })

-- ============================================================================
-- Return scene for renderer
-- ============================================================================

return ScriptCAD.serialize()
