-- fdtd_resonator.lua
-- Integration test for internal FDTD electromagnetic solver
--
-- This example demonstrates:
-- - Simple loop-gap resonator geometry
-- - FdtdStudy API for internal Rust FDTD simulation
-- - Time-domain field visualization via oscilloscope widget

local Mittens = require("stdlib")

-- ============================================================================
-- FDTD Study Configuration
-- ============================================================================

-- This global triggers the FDTD solver
FdtdStudy = {
  -- Center frequency and bandwidth (Hz)
  freq_center = 450e6,    -- 450 MHz center
  freq_width = 200e6,     -- 200 MHz bandwidth for broadband excitation

  -- Grid resolution (mm)
  cell_size = 2.0,        -- 2mm cells (coarse for fast iteration)

  -- PML absorbing boundary layers
  pml_thickness = 6,      -- 6 cells of PML

  -- Simulation time (ns)
  max_time_ns = 50.0,     -- 50 ns maximum simulation

  -- Source position relative to center (mm)
  source_offset = {0, 0, 5},  -- 5mm above center

  -- Monitor position relative to center (mm)
  monitor_offset = {0, 0, 0}, -- At center

  -- Field plane for visualization
  field_plane = "XZ",     -- XZ slice through center
}

-- ============================================================================
-- Materials
-- ============================================================================

local copper = material("copper")

-- ============================================================================
-- Geometry: Simple Loop-Gap Resonator
--
-- Structure:
--   - Cylindrical shell (loop) with a gap
--   - Resonates at a frequency determined by loop inductance and gap capacitance
-- ============================================================================

-- Loop parameters (mm)
local loop = {
  outer_radius = 20,   -- outer radius
  inner_radius = 16,   -- inner radius (wall thickness = 4mm)
  height = 10,         -- height of loop
  gap_angle = 30,      -- gap angle in degrees
}

-- Create the loop as a cylinder with a wedge removed
local outer = cylinder(loop.outer_radius, loop.height)
local inner = cylinder(loop.inner_radius, loop.height + 2)

-- Gap wedge (remove a sector)
local gap_size = loop.outer_radius * 2.5
local gap_wedge = box(gap_size, gap_size, loop.height + 2)
    :at(gap_size / 2, 0, 0)
    :rotate(0, 0, -loop.gap_angle / 2)

-- Build the loop: outer - inner - gap
local loop_shell = difference(outer, inner)
local loop_gap = difference(loop_shell, gap_wedge)
    :material(copper)
    :name("loop_gap_resonator")

-- ============================================================================
-- Assembly
-- ============================================================================

local assembly = group("resonator_assembly", {
  loop_gap
})

Mittens.register(assembly)

-- ============================================================================
-- Virtual Instruments (optional - for Helmholtz coil mode)
-- ============================================================================

-- GaussMeter at center to measure B-field
GaussMeter({0, 0, 0}, {
  range = "mT",
  label = "B_center"
})

-- ============================================================================
-- View Configuration
-- ============================================================================

view({
  camera = {
    position = {80, -60, 40},
    target = {0, 0, 0},
    fov = 45
  }
})

-- ============================================================================
-- Return scene for renderer
-- ============================================================================

return Mittens.serialize()
