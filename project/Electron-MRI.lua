-- Electron-MRI.lua
-- Single Loop Multi-Gap (SLMG) resonator for EPR imaging at 1.2 GHz
-- Based on: Petryakov et al. J. Magn. Reson. 188 (2007) 68-73

local Mittens = require("stdlib")

-- ===========================
-- Configuration Parameters
-- From paper: 42mm i.d., 88mm o.d., 48mm length, 16 gaps
-- ===========================

Resonator = {
  inner_diameter = 42,
  outer_diameter = 88,
  length = 48,
  num_gaps = 16,
  gap_thickness = 1.68,
}

Resonator.inner_radius = Resonator.inner_diameter / 2
Resonator.outer_radius = Resonator.outer_diameter / 2
Resonator.segment_angle = 360 / Resonator.num_gaps
Resonator.gap_angle_rad = 2 * math.atan(Resonator.gap_thickness / 2 / Resonator.inner_radius)
Resonator.gap_angle = math.deg(Resonator.gap_angle_rad)
Resonator.wedge_angle = Resonator.segment_angle - Resonator.gap_angle

CouplingLoop = {
  loop_diameter = 20,
  wire_diameter = 1.5,
  num_loops = 2,
  spacing = 10,
  z_offset = Resonator.length / 2,
}

Phantom = {
  tube_diameter = 4,
  tube_count = 19,
  fill_height = 11,
  arrangement_radius = 12,
}

PVCShell = {
  thickness = 2,
  inner_radius = Resonator.inner_radius - 2,
}

ModulationCoil = {
  wire_diameter = 0.5,
  coil_width = 10,
  num_turns = 20,
  axial_offset = Resonator.length / 4,
}

Housing = {
  outer_radius = Resonator.outer_radius + 5,
  wall_thickness = 3,
  lid_thickness = 5,
  slot_width = 15,
  num_slots = 4,
}

-- ===========================
-- Materials
-- ===========================

local rexolite = material("rexolite", {
  relative_permittivity = 2.53,
  relative_permeability = 1.0,
  loss_tangent = 0.00066,
})

local polystyrene = material("polystyrene", {
  relative_permittivity = 2.6,
  relative_permeability = 1.0,
})

local pvc = material("pvc", {
  relative_permittivity = 3.4,
  relative_permeability = 1.0,
})

local silver = material("silver", {
  conductivity = 6.3e7,
  relative_permeability = 1.0,
})

local copper = material("copper", {
  conductivity = 5.96e7,
  relative_permeability = 1.0,
})

local water = material("water", {
  relative_permittivity = 80,
  conductivity = 0.01,
})

-- ===========================
-- SLMG Resonator Segments
-- 16 Rexolite wedge segments with silver plating on inner face
-- ===========================

local segments = {}
for i = 0, Resonator.num_gaps - 1 do
  local angle = i * Resonator.segment_angle
  local seg = wedge(
    Resonator.inner_radius,
    Resonator.outer_radius,
    Resonator.length,
    Resonator.wedge_angle
  ):rotate(0, 0, angle):material(rexolite):color(0.85, 0.75, 0.55, 1.0)
  table.insert(segments, seg)
end

local resonator_body = group("resonator_segments", segments)

-- ===========================
-- Gap Plates (Polystyrene dielectric)
-- Thin wedges filling the gaps between segments
-- ===========================

local gap_plates = {}
for i = 0, Resonator.num_gaps - 1 do
  local angle = i * Resonator.segment_angle + Resonator.wedge_angle / 2 + Resonator.gap_angle / 2
  local gap = wedge(
    Resonator.inner_radius,
    Resonator.outer_radius,
    Resonator.length,
    Resonator.gap_angle
  ):rotate(0, 0, angle):material(polystyrene):color(0.9, 0.9, 0.95, 0.8)
  table.insert(gap_plates, gap)
end

local dielectric_gaps = group("gap_plates", gap_plates)

-- ===========================
-- PVC Reinforcing Inner Shell
-- Hollow cylinder inside the resonator
-- ===========================

local pvc_shell = ring(
  PVCShell.inner_radius,
  Resonator.inner_radius,
  Resonator.length
):material(pvc):color(0.5, 0.5, 0.55, 0.9)

-- ===========================
-- Double Coupling Loop
-- Two parallel loops with lambda/4 feeding lines
-- ===========================

local coupling_loops = {}
local loop_radius = CouplingLoop.loop_diameter / 2
local wire_r = CouplingLoop.wire_diameter / 2

for i = 0, CouplingLoop.num_loops - 1 do
  local y_offset = (i - 0.5) * CouplingLoop.spacing
  local loop = torus(loop_radius, wire_r)
    :rotate(90, 0, 0)
    :at(-(Resonator.outer_radius + 15), y_offset, CouplingLoop.z_offset)
    :material(copper)
    :color(0.9, 0.6, 0.3, 1.0)
  table.insert(coupling_loops, loop)
end

local coupling_assembly = group("coupling_loops", coupling_loops)

-- ===========================
-- 19-Tube Phantom
-- Polystyrene tubes filled with TAM solution
-- Arranged in hexagonal pattern for B1 homogeneity testing
-- ===========================

local phantom_tubes = {}
local tube_r = Phantom.tube_diameter / 2

local function hex_positions(count, spacing)
  local positions = {{0, 0}}
  local ring_num = 1
  while #positions < count do
    for i = 0, 5 do
      local angle = i * 60 + 30
      local x = ring_num * spacing * math.cos(math.rad(angle))
      local y = ring_num * spacing * math.sin(math.rad(angle))
      table.insert(positions, {x, y})
      for j = 1, ring_num - 1 do
        local next_angle = angle + 60
        local dx = spacing * math.cos(math.rad(next_angle))
        local dy = spacing * math.sin(math.rad(next_angle))
        table.insert(positions, {x + j * dx, y + j * dy})
      end
    end
    ring_num = ring_num + 1
  end
  local result = {}
  for i = 1, count do
    table.insert(result, positions[i])
  end
  return result
end

local tube_positions = hex_positions(Phantom.tube_count, Phantom.tube_diameter + 1)
for _, pos in ipairs(tube_positions) do
  local tube = cylinder(tube_r, Phantom.fill_height)
    :at(pos[1], pos[2], (Resonator.length - Phantom.fill_height) / 2)
    :material(water)
    :color(0.3, 0.5, 0.9, 0.6)
  table.insert(phantom_tubes, tube)
end

local phantom = group("phantom", phantom_tubes)

-- ===========================
-- Modulation Coils
-- Form-wound coils for EPR field modulation at 100 kHz
-- Positioned on either side of the resonator center
-- ===========================

local modulation_coils = {}
local mod_coil_r = PVCShell.inner_radius - 1

for i, z_sign in ipairs({1, -1}) do
  local z_pos = z_sign * ModulationCoil.axial_offset
  local mod_coil = ring(
    mod_coil_r - ModulationCoil.coil_width / 2,
    mod_coil_r + ModulationCoil.coil_width / 2,
    ModulationCoil.wire_diameter * ModulationCoil.num_turns
  ):at(0, 0, z_pos):material(copper):color(0.9, 0.6, 0.3, 0.8)
  table.insert(modulation_coils, mod_coil)
end

local modulation_assembly = group("modulation_coils", modulation_coils)

-- ===========================
-- Housing and Shield
-- PVC case with silver-plated interior for RF shielding
-- Includes slots for sample access
-- ===========================

local housing_outer = ring(
  Housing.outer_radius - Housing.wall_thickness,
  Housing.outer_radius,
  Resonator.length + 2 * Housing.lid_thickness
):at(0, 0, -Housing.lid_thickness):material(pvc):color(0.6, 0.6, 0.6, 0.5)

local lid_top = cylinder(Housing.outer_radius, Housing.lid_thickness)
  :at(0, 0, Resonator.length / 2)
  :material(silver)
  :color(0.8, 0.8, 0.85, 0.9)

local lid_bottom = cylinder(Housing.outer_radius, Housing.lid_thickness)
  :at(0, 0, -Resonator.length / 2 - Housing.lid_thickness)
  :material(silver)
  :color(0.8, 0.8, 0.85, 0.9)

local housing = group("housing", {housing_outer, lid_top, lid_bottom})

-- ===========================
-- Assembly
-- ===========================

local assembly = group("electron_mri", {
  resonator_body,
  dielectric_gaps,
  pvc_shell,
  coupling_assembly,
  phantom,
  modulation_assembly,
  housing,
})

Mittens.register(assembly)

-- ===========================
-- NanoVNA Configuration for Multi-Gap Resonator
-- Physics: omega = 1/sqrt(L_sum * C_sum) where L_sum = L*N, C_sum = C/N
-- ===========================

NanoVNA = {
  f_start = 1.0e9,
  f_stop = 1.5e9,
  num_points = 201,
  num_gaps = Resonator.num_gaps,
  inner_radius = Resonator.inner_radius,
  outer_radius = Resonator.outer_radius,
  length = Resonator.length,
  gap_thickness = Resonator.gap_thickness,
  gap_permittivity = 2.6,
}

view({
  flat_shading = true,
  circular_segments = 64,
})

return Mittens.serialize()
