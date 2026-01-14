-- helmholtz_coil.lua
-- Helmholtz coil pair for uniform magnetic field generation
--
-- This example demonstrates:
-- - Parametric coil design with real wire dimensions
-- - Coil cross-section calculated from windings and packing
-- - Magnetostatic simulation for field uniformity analysis
-- - Field visualization in the uniform region

-- Load standard library
local ScriptCAD = require("stdlib")

-- ============================================================================
-- Configuration Parameters
-- ============================================================================

config = {
  -- Coil geometry
  coil_mean_radius = 50, -- mean radius of coils (mm)
  gap = 40,              -- distance between inner surfaces of coils (mm)
  -- This is the clear space between the two coil windings

  -- Wire and winding parameters
  wire_diameter = 0.8,   -- copper wire diameter (mm), e.g. AWG 20 ≈ 0.81mm
  windings = 120,        -- total number of turns per coil
  layers = 12,           -- number of radial layers
  packing_factor = 0.82, -- wire packing efficiency (0.7-0.9 typical for hand wound)

  -- Derived coil cross-section will be calculated below

  -- Electrical parameters
  current = 2.0, -- coil current (A)

  -- Resonator tube parameters
  resonator_wall = 2.0, -- wall thickness (mm)

  -- Simulation domain
  domain_radius = 200, -- simulation domain radius (mm)
  domain_height = 300, -- simulation domain height (mm)
}

-- ============================================================================
-- Calculate Coil Dimensions from Wire Parameters
-- ============================================================================

-- Turns per layer (axial direction)
local turns_per_layer = math.ceil(config.windings / config.layers)

-- Effective wire pitch accounting for packing
local wire_pitch = config.wire_diameter / config.packing_factor

-- Coil cross-section dimensions
local coil_width = turns_per_layer * wire_pitch -- axial extent (mm)
local coil_height = config.layers * wire_pitch  -- radial extent (mm)

-- Inner and outer radii
local coil_inner_r = config.coil_mean_radius - coil_height / 2
local coil_outer_r = config.coil_mean_radius + coil_height / 2

-- Coil Z positions: gap is between inner surfaces
-- Cylinder base on XY plane, extends along +Z
-- Upper coil: base at z = gap/2, extends to gap/2 + coil_width
-- Lower coil: base at z = -gap/2 - coil_width, extends to -gap/2
local upper_coil_z = config.gap / 2
local lower_coil_z = -config.gap / 2 - coil_width

-- Center-to-center distance (for Helmholtz condition check)
local center_distance = config.gap + coil_width

-- Print calculated dimensions
print("=== Helmholtz Coil Configuration ===")
print(string.format("Wire: %.2f mm diameter, packing factor %.2f", config.wire_diameter, config.packing_factor))
print(string.format("Windings: %d total (%d layers x %d turns/layer)",
  config.layers * turns_per_layer, config.layers, turns_per_layer))
print(string.format("Coil cross-section: %.1f mm (axial) x %.1f mm (radial)", coil_width, coil_height))
print(string.format("Coil radii: inner=%.1f mm, mean=%.1f mm, outer=%.1f mm",
  coil_inner_r, config.coil_mean_radius, coil_outer_r))
print(string.format("Gap between coils: %.1f mm", config.gap))
print(string.format("Center-to-center distance: %.1f mm", center_distance))
print(string.format("Helmholtz ratio (d/R): %.3f (ideal = 1.0)", center_distance / config.coil_mean_radius))

-- ============================================================================
-- Materials
-- ============================================================================

local copper = material("copper", {
  conductivity = 5.8e7, -- S/m
  relative_permeability = 1.0,
})

local air = material("air", {
  relative_permeability = 1.0,
})

-- ============================================================================
-- Geometry: Coil Windings
-- ============================================================================

-- Each coil is modeled as a tube (hollow cylinder) representing the
-- rectangular cross-section winding bundle

-- Upper coil (base at z = gap/2)
local upper_coil_outer = cylinder(coil_outer_r, coil_width)
local upper_coil_inner = cylinder(coil_inner_r, coil_width + 1)

local upper_coil = difference(upper_coil_outer, upper_coil_inner)
    :at(0, 0, upper_coil_z)
    :material(copper)
    :name("upper_coil")

-- Lower coil (base at z = -gap/2 - coil_width)
local lower_coil_outer = cylinder(coil_outer_r, coil_width)
local lower_coil_inner = cylinder(coil_inner_r, coil_width + 1)

local lower_coil = difference(lower_coil_outer, lower_coil_inner)
    :at(0, 0, lower_coil_z)
    :material(copper)
    :name("lower_coil")

-- ============================================================================
-- Geometry: Bridge Gap Resonator Tube
-- ============================================================================

-- Tube in the center, axis along Z, outer diameter = gap width
local resonator_outer_r = config.gap / 2
local resonator_inner_r = resonator_outer_r - config.resonator_wall
local resonator_height = 90 -- spans the gap

local resonator_outer = cylinder(resonator_outer_r, resonator_height)
local resonator_inner = cylinder(resonator_inner_r, resonator_height + 1)

local resonator = difference(resonator_outer, resonator_inner)
    :at(0, 0, -resonator_height / 2)
    :material(copper)
    :rotate(90, 0, 0)
    :name("resonator")

print(string.format("Resonator tube: OD=%.1f mm, ID=%.1f mm, height=%.1f mm",
  resonator_outer_r * 2, resonator_inner_r * 2, resonator_height))

-- ============================================================================
-- Assembly
-- ============================================================================

local assembly = group("helmholtz_coil", {
  upper_coil,
  lower_coil,
  resonator,
})

-- Register with scene
ScriptCAD.register(assembly)

-- ============================================================================
-- Physics Setup: Magnetostatic Study
-- ============================================================================

-- Total ampere-turns per coil
local ampere_turns = config.current * config.windings

local mag_study = magnetostatic({
      solver = "direct",
      formulation = "vector_potential", -- A-formulation for magnetostatics

      -- Current excitation for each coil
      -- Both coils carry current in same direction for Helmholtz configuration
      sources = {
        current_source(upper_coil, {
          current = config.current,
          turns = config.windings,
          direction = "ccw", -- counter-clockwise when viewed from +Z
        }),
        current_source(lower_coil, {
          current = config.current,
          turns = config.windings,
          direction = "ccw", -- same direction as upper coil
        }),
      },
    })
    :domain(assembly)
    :boundary("outer", {
      type = "magnetic_insulation", -- n × A = 0 (tangential A = 0)
      distance = config.domain_radius,
    })
    :mesh({
      type = "tetrahedral",
      max_element_size = 10,
      min_element_size = 1.0,
      curvature_factor = 0.25,
      refinement_regions = {
        -- Fine mesh in coils for current density
        { region = upper_coil, size = 2.0 },
        { region = lower_coil, size = 2.0 },
        -- Very fine mesh in uniform field region (center)
        { region = "sphere",   center = { 0, 0, 0 }, radius = config.gap * 0.4, size = 1.5 },
        -- Medium mesh in gap region
        {
          region = "cylinder",
          axis = "z",
          radius = coil_inner_r * 0.8,
          z_min = -config.gap / 2,
          z_max = config.gap / 2,
          size = 3.0
        },
      }
    })

-- ============================================================================
-- Virtual Instruments
-- ============================================================================

-- Measure B-field at center (should be uniform region)
GaussMeter({ 0, 0, 0 }, {
  range = "mT",
  component = "z",
  label = "B_center"
})

-- Measure B-field along Z axis (axial uniformity)
local coil_center_z = config.gap / 2 + coil_width / 2
Probe("B_z_axis", {
  type = "B_field",
  line = { { 0, 0, -coil_center_z }, { 0, 0, coil_center_z } },
  points = 101,
  component = "z",
  export = "results/B_axial.csv"
})

-- Measure B-field along X axis at z=0 (radial uniformity)
Probe("B_x_axis", {
  type = "B_field",
  line = { { -coil_inner_r, 0, 0 }, { coil_inner_r, 0, 0 } },
  points = 51,
  component = "z",
  export = "results/B_radial.csv"
})

-- Field uniformity metric in central region
Probe("B_uniformity", {
  type = "B_field",
  volume = { center = { 0, 0, 0 }, radius = config.gap * 0.3 },
  samples = 1000,
  statistics = { "mean", "std", "min", "max" },
})

-- Magnetic field visualization on XZ plane (through coil axes)
MagneticFieldPlane("XZ", 0, {
  quantity = "B",
  style = "streamlines",
  scale = "linear",
  resolution = 50,
  streamline_density = 2.0,
  color_map = "viridis"
})

-- |B| magnitude on XZ plane
MagneticFieldPlane("XZ_mag", 0, {
  quantity = "B_magnitude",
  style = "colormap",
  scale = "linear",
  resolution = 80,
  color_map = "plasma",
  range = "auto"
})

-- ============================================================================
-- Theoretical Calculations
-- ============================================================================

-- Theoretical B-field at center of ideal Helmholtz coil:
-- B = (4/5)^(3/2) * μ0 * N * I / R
-- where N = turns per coil, I = current, R = mean coil radius
-- Valid when coil separation d = R (center-to-center)

local mu0 = 4 * math.pi * 1e-7 -- H/m (permeability of free space)
local N = config.windings
local I = config.current
local R = config.coil_mean_radius * 1e-3 -- convert to meters

-- Helmholtz factor for ideal configuration
local helmholtz_factor = math.pow(4 / 5, 1.5) -- ≈ 0.7155

-- Theoretical field (assuming ideal Helmholtz condition)
local B_ideal = helmholtz_factor * mu0 * N * I / R

-- Actual calculation using Biot-Savart for our geometry
-- For a single loop: B_z = μ0 * I * R² / (2 * (R² + z²)^(3/2))
-- For coil at z = d/2: evaluate at z = 0
local d = center_distance * 1e-3  -- center-to-center in meters
local B_single_coil = mu0 * N * I * R ^ 2 / (2 * math.pow(R ^ 2 + (d / 2) ^ 2, 1.5))
local B_total = 2 * B_single_coil -- two coils

print("")
print("=== Theoretical Field Estimates ===")
print(string.format("Ampere-turns per coil: %.0f A·turns", ampere_turns))
print(string.format("Ideal Helmholtz B-field (d=R): %.3f mT", B_ideal * 1000))
print(string.format("Actual B-field estimate (d=%.1fmm): %.3f mT", center_distance, B_total * 1000))
print(string.format("Deviation from ideal: %.1f%%", (B_total / B_ideal - 1) * 100))

-- ============================================================================
-- View Configuration
-- ============================================================================

view({
  camera = "isometric",
  distance = config.coil_mean_radius * 4,
  target = { 0, 0, 0 },

  -- Cross-section to see field inside
  clip = {
    plane = "XZ",
    offset = 0,
    show_caps = true
  },

  show = {
    "helmholtz_coil",
  },

  theme = "dark",
  axes = { show = true, size = config.coil_mean_radius * 0.5 },

  render = {
    quality = "high",
    shadows = false,
  }
})

-- ============================================================================
-- Return scene for renderer
-- ============================================================================

return ScriptCAD.serialize()
