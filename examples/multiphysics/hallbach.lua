-- hallbach.lua
-- Halbach array magnet holder system for cell culture studies

local ScriptCAD = require("stdlib")

-- ===========================
-- Configuration Parameters
-- ===========================

local cylinder_height = 30
local cylinder_outer_radius = 50
local center_hole_diameter = 25
local micro_slide_l = 75
local micro_slide_w = 25
local micro_slide_h = 1.1
local center_hole_radius = center_hole_diameter / 2
local square_size = 12.8
local inch = 25.4
local num_squares = 8
local num_squares_outer = 12
local square_hypo = square_size * math.sqrt(2)
local rim = 0.4
local outer_radius = center_hole_radius + square_hypo + rim
local outer_outer_radius = outer_radius + square_hypo + 1.5
local gap = 0.2
local total_hole_width = 110
local overhang = 10
local cutout_height = cylinder_height - inch
local platform_holder_height = 0.5
local overhang_h = 2
local cap_height = 2
local cover_slip_r = 20 / 2
local cover_slip_h = 0.17 + 0.13
local motor_w = 42.3 + 0.2 + overhang_h
local m_wall = 5
local m_insert_h = 1
local motor_h = 38 + 0.4
local cable_insert = 16 + 2
local screw_r = 3.2 / 2
local axle_r = 22 / 2 + 0.2
local screw_off = 31 / 2
local vertical_offset_of_overhang = 5

local liquid_wall = 1
local transducer_diameter = 64
local focal_distance = 50
local total_height = cylinder_height + cap_height
local top_container_offset = focal_distance - total_height
local transducer_height_val = 5

local well_wall = 1.1
local well_h = 5
local deep_well_h = cylinder_height + platform_holder_height + cap_height + 2 * liquid_wall
local deep_well_r = cover_slip_r + gap + well_wall

local single_transducer_r = 5
local transducer_position = 5 + 1
local tol = 0.5
local well_cutout = cover_slip_r + gap + well_wall + tol
local bottom_wall = liquid_wall * 2
local setter_w = 4 * liquid_wall + center_hole_radius
local setter_h = 5
local num_setters = 5
local offset_setters = 2
local angle_setters = 70 / num_setters
local trans_h = 4
local trans_r = 10 / 2
local trans_holder_w = trans_r + tol + well_wall
local cutout_h = 3 * well_h
local transducer_r_cutout = trans_r * 1.5

local belt_offset = 3 + (vertical_offset_of_overhang + motor_h + m_insert_h - cylinder_height)
local belt_width = 6 + 1
local belt_height = 1 + belt_width + belt_offset

local cable_w = 4
local transducer_offset = 0.2

-- ===========================
-- Helper: Oval shape
-- ===========================

local function oval(r, w, h)
  return union(
    cylinder(r, h):at(w, 0, 0),
    cylinder(r, h):at(-w, 0, 0),
    box(2 * w, 2 * r, h):centerXY():at(0, 0, h / 2)
  )
end

-- ===========================
-- Geometry: Ring Cylinder (Inner)
-- ===========================

local ring_cylinder_body = difference(
  cylinder(outer_radius, cylinder_height),
  cylinder(center_hole_radius, cylinder_height + 1):at(0, 0, -0.5)
)

local cutout_radius = center_hole_radius + square_hypo / 2

local inner_magnet_cutouts = {}
for i = 0, num_squares - 1 do
  local angle = i * (360 / num_squares)
  local is_rotated = (i % 2 == 1)
  local rotation = is_rotated and 45 or 0
  table.insert(inner_magnet_cutouts,
    box(square_size, square_size, cylinder_height + 1):center(true, true, true)
    :rotate(0, 0, rotation)
    :at(cutout_radius, 0, cylinder_height / 2)
    :rotate(0, 0, angle)
  )
end

local inner_slot_cutouts = {}
for i = 0, num_squares - 1 do
  local angle = i * (360 / num_squares)
  local is_rotated = (i % 2 == 1)
  local rotation = is_rotated and 45 or 0
  table.insert(inner_slot_cutouts,
    box(square_size / 2, square_size + 2, cutout_height):center(true, true, true)
    :rotate(0, 0, rotation)
    :at(cutout_radius, 0, cylinder_height - cutout_height / 2)
    :rotate(0, 0, angle)
  )
end

RingCylinder = {}
RingCylinder.body = ring_cylinder_body
for _, cutout in ipairs(inner_magnet_cutouts) do
  RingCylinder.body = difference(RingCylinder.body, cutout)
end
for _, cutout in ipairs(inner_slot_cutouts) do
  RingCylinder.body = difference(RingCylinder.body, cutout)
end

RingCylinder.platform = difference(
  cylinder((total_hole_width - gap) / 2, platform_holder_height):at(0, 0, -platform_holder_height),
  cylinder(center_hole_radius, cylinder_height + 1):at(0, 0, -platform_holder_height - 1)
)

RingCylinder.overhang = difference(
  cylinder((total_hole_width + overhang) / 2, vertical_offset_of_overhang + overhang_h),
  cylinder(outer_outer_radius + 1, vertical_offset_of_overhang + overhang_h),
  difference(
    cylinder(total_hole_width, vertical_offset_of_overhang),
    cylinder((total_hole_width - gap) / 2, vertical_offset_of_overhang)
  )
)

-- ===========================
-- Geometry: Ring Cylinder (Outer)
-- ===========================

local ring_cylinder_outer_body = difference(
  cylinder(outer_outer_radius, cylinder_height),
  cylinder(outer_radius + gap, cylinder_height + 1):at(0, 0, -0.5)
)

local outer_cutout_radius = outer_radius + square_hypo / 2 + gap

local outer_magnet_cutouts = {}
for i = 0, num_squares_outer - 1 do
  local angle = i * (360 / num_squares_outer)
  local rotation = (i % 3 == 0) and 0 or ((i % 3 == 1) and 30 or -30)
  table.insert(outer_magnet_cutouts,
    box(square_size, square_size, cylinder_height + 1):center(true, true, true)
    :rotate(0, 0, rotation)
    :at(outer_cutout_radius, 0, cylinder_height / 2)
    :rotate(0, 0, angle)
  )
end

local outer_slot_cutouts = {}
for i = 0, num_squares_outer - 1 do
  local angle = i * (360 / num_squares_outer)
  local rotation = (i % 3 == 0) and 0 or ((i % 3 == 1) and 30 or -30)
  table.insert(outer_slot_cutouts,
    box(square_size / 2, square_size + 2, cutout_height * 2):center(true, true, true)
    :rotate(0, 0, rotation)
    :at(outer_cutout_radius, 0, cylinder_height - cutout_height / 2)
    :rotate(0, 0, angle)
  )
end

RingCylinderOuter = {}
RingCylinderOuter.model = ring_cylinder_outer_body
for _, cutout in ipairs(outer_magnet_cutouts) do
  RingCylinderOuter.model = difference(RingCylinderOuter.model, cutout)
end
for _, cutout in ipairs(outer_slot_cutouts) do
  RingCylinderOuter.model = difference(RingCylinderOuter.model, cutout)
end

-- ===========================
-- Geometry: Cap Inner
-- ===========================

CapInner = {}

CapInner.top = difference(
  cylinder(outer_radius, cap_height):at(0, 0, cylinder_height),
  cylinder(center_hole_radius, cap_height + 1):at(0, 0, cylinder_height - 0.2)
)

local cap_inner_magnets = {}
for i = 0, num_squares - 1 do
  local angle_deg = i * (360 / num_squares)
  local angle_rad = math.rad(angle_deg)
  local is_rotated = (i % 2 == 1)
  local cutout_radius = center_hole_radius + square_hypo / 2
  local rotation = is_rotated and 45 or 0
  local x = cutout_radius * math.cos(angle_rad)
  local y = cutout_radius * math.sin(angle_rad)
  table.insert(cap_inner_magnets,
    box(square_size, square_size, cutout_height):center(true, true, true)
    :rotate(0, 0, rotation + angle_deg)
    :at(x, y, cylinder_height - cutout_height / 2)
  )
end

local cap_inner_slots = {}
for i = 0, num_squares - 1 do
  local angle_deg = i * (360 / num_squares)
  local angle_rad = math.rad(angle_deg)
  local is_rotated = (i % 2 == 1)
  local cutout_radius = center_hole_radius + square_hypo / 2
  local rotation = is_rotated and 45 or 0
  local x = cutout_radius * math.cos(angle_rad)
  local y = cutout_radius * math.sin(angle_rad)
  table.insert(cap_inner_slots,
    box(square_size / 2, square_size + 2 - gap, cutout_height):center(true, true, true)
    :rotate(0, 0, rotation + angle_deg)
    :at(x, y, cylinder_height - cutout_height / 2)
  )
end

local cap_inner_parts = { CapInner.top }
for _, v in ipairs(cap_inner_magnets) do table.insert(cap_inner_parts, v) end
for _, v in ipairs(cap_inner_slots) do table.insert(cap_inner_parts, v) end
CapInner.model = group("cap_inner", cap_inner_parts)

-- ===========================
-- Geometry: Cap Outer
-- ===========================

CapOuter = {}
CapOuter.body = difference(
  cylinder(outer_outer_radius, belt_height):at(0, 0, cylinder_height),
  cylinder(outer_radius + gap, belt_height):at(0, 0, cylinder_height)
)

local cap_outer_magnets = {}
for i = 0, num_squares_outer - 1 do
  local angle_deg = i * (360 / num_squares_outer)
  local angle_rad = math.rad(angle_deg)
  local rotation = (i % 3 == 0) and 0 or ((i % 3 == 1) and 30 or -30)
  local cutout_radius = outer_radius + square_hypo / 2 + gap
  local x = cutout_radius * math.cos(angle_rad)
  local y = cutout_radius * math.sin(angle_rad)
  table.insert(cap_outer_magnets,
    box(square_size - tol, square_size - tol, cutout_height - tol):center(true, true, true)
    :rotate(0, 0, rotation + angle_deg)
    :at(x, y, cylinder_height - cutout_height / 2 + tol)
  )
end

local cap_outer_slots = {}
for i = 0, num_squares_outer - 1 do
  local angle_deg = i * (360 / num_squares_outer)
  local angle_rad = math.rad(angle_deg)
  local rotation = (i % 3 == 0) and 0 or ((i % 3 == 1) and 30 or -30)
  local cutout_radius = outer_radius + square_hypo / 2 + gap
  local x = cutout_radius * math.cos(angle_rad)
  local y = cutout_radius * math.sin(angle_rad)
  table.insert(cap_outer_slots,
    box(square_size / 2 - tol, square_size + 2 - tol, cutout_height - tol):center(true, true, true)
    :rotate(0, 0, rotation + angle_deg)
    :at(x, y, cylinder_height - cutout_height / 2 + tol)
  )
end

local cap_outer_parts = { CapOuter.body }
for _, v in ipairs(cap_outer_magnets) do table.insert(cap_outer_parts, v) end
for _, v in ipairs(cap_outer_slots) do table.insert(cap_outer_parts, v) end
CapOuter.model = group("cap_outer", cap_outer_parts)

-- ===========================
-- Geometry: Well
-- ===========================

Well = {}
Well.model = difference(
  cylinder(cover_slip_r + gap + well_wall, well_h),
  cylinder(cover_slip_r + gap, well_h):at(0, 0, well_wall / 2),
  cylinder(cover_slip_r - well_wall, well_h / 2)
)

-- ===========================
-- Geometry: Deep Well
-- ===========================

DeepWell = {}
DeepWell.body = difference(
  cylinder(deep_well_r, deep_well_h):at(0, 0, -platform_holder_height),
  cylinder(cover_slip_r + gap, deep_well_h):at(0, 0, -platform_holder_height + well_wall / 2),
  cylinder(cover_slip_r - 2 * well_wall, well_h / 2):at(0, 0, -platform_holder_height)
)

DeepWell.top = difference(
  cylinder(deep_well_r + 2 * liquid_wall, liquid_wall):at(0, 0, deep_well_h - platform_holder_height),
  cylinder(cover_slip_r + gap, 2 * liquid_wall):at(0, 0, deep_well_h - platform_holder_height)
)

DeepWell.model = group("deep_well", { DeepWell.body, DeepWell.top })

-- ===========================
-- Geometry: Well Holder
-- ===========================

WellHolder = {}

WellHolder.wall = difference(
  cylinder(center_hole_radius - tol, total_height),
  cylinder(center_hole_radius - liquid_wall, total_height + liquid_wall),
  box(well_cutout * 2, center_hole_radius, cutout_h):at(-well_cutout, 0, 0)
)

WellHolder.bottom = difference(
  cylinder(center_hole_radius - tol, bottom_wall):at(0, 0, -bottom_wall),
  cylinder(cover_slip_r - well_wall, bottom_wall):at(0, 0, -bottom_wall),
  cylinder(cover_slip_r + gap + well_wall, well_h):at(0, 0, -2 * tol)
)

WellHolder.top_ring = difference(
  cylinder(2 * liquid_wall + center_hole_radius, liquid_wall):at(0, 0, total_height),
  cylinder(trans_holder_w, liquid_wall):at(0, 0, total_height)
)

local setter_slots = {}
for i = 0, num_setters * 2 - 1 do
  local angle = -5 + angle_setters * i
  table.insert(setter_slots,
    box(setter_w * 2, setter_w / 3, setter_h * 4):center(true, true, true)
    :at(0, 0, i * offset_setters)
    :rotate(0, 0, angle)
  )
end

WellHolder.setter = difference(
  cylinder(2 * liquid_wall + center_hole_radius, setter_h + num_setters * offset_setters):at(0, 0, total_height),
  cylinder(center_hole_radius - liquid_wall, total_height):at(0, 0, total_height),
  group("setter_slots", setter_slots):at(0, 0, total_height + setter_h * 2 + liquid_wall)
)

WellHolder.model = group("well_holder", {
  WellHolder.wall,
  WellHolder.bottom,
  WellHolder.top_ring,
  WellHolder.setter
})

-- ===========================
-- Geometry: Deep Well Holder
-- ===========================

DeepWellHolder = {}

DeepWellHolder.top_ring = difference(
  cylinder(setter_w, liquid_wall):at(0, 0, total_height),
  cylinder(deep_well_r + tol, liquid_wall):at(0, 0, total_height)
)

local deep_setter_slots = {}
for i = 0, num_setters * 2 - 1 do
  local angle = -5 + angle_setters * i
  table.insert(deep_setter_slots,
    box(setter_w * 2, setter_w / 3, setter_h * 4):center(true, true, true)
    :at(0, 0, i * offset_setters)
    :rotate(0, 0, angle)
  )
end

DeepWellHolder.setter = difference(
  cylinder(setter_w, setter_h + num_setters * offset_setters):at(0, 0, total_height + liquid_wall),
  cylinder(center_hole_radius + 2 * liquid_wall, total_height):at(0, 0, total_height + liquid_wall),
  group("deep_setter_slots", deep_setter_slots):at(0, 0, total_height + liquid_wall + setter_h * 2 + liquid_wall)
)

DeepWellHolder.model = group("deep_well_holder", { DeepWellHolder.top_ring, DeepWellHolder.setter })

-- ===========================
-- Geometry: Transducer
-- ===========================

Transducer = {}
Transducer.model = cylinder(trans_r, trans_h):at(0, 0, transducer_position + 0.1)

-- ===========================
-- Geometry: Transducer Holder
-- ===========================

TransHolder = {}

TransHolder.body = difference(
  cylinder(trans_holder_w, liquid_wall - tol + total_height - transducer_offset):at(0, 0, transducer_offset + tol),
  cylinder(trans_r + tol, trans_h + tol / 2):at(0, 0, transducer_offset + tol),
  box(cable_w, 3 * well_wall, total_height + tol):at(-cable_w / 2, trans_holder_w - 3 * well_wall,
    transducer_offset + tol)
)

local trans_setter_bars = {}
for i = 0, num_setters - 1 do
  local angle = angle_setters * i
  table.insert(trans_setter_bars,
    box(setter_w * 2, setter_w / 4, setter_h):center(true, true, true)
    :at(0, 0, i * offset_setters)
    :rotate(0, 0, angle)
  )
end

TransHolder.setter = group("trans_setter", trans_setter_bars)
    :at(0, 0, total_height + setter_h / 2 + liquid_wall)

TransHolder.model = group("trans_holder", { TransHolder.body, TransHolder.setter })

-- ===========================
-- Motor Holder
-- ===========================

MotorHolder = {}

local motor_screw_holes = {}
for x = -1, 1, 2 do
  for y = -1, 1, 2 do
    table.insert(motor_screw_holes, oval(screw_r, 1, 2 * cylinder_height):at(x * screw_off + 1, y * screw_off, 0))
  end
end

MotorHolder.model = box(motor_w + 2 * m_wall, motor_w + 2 * m_wall, motor_h + m_insert_h):center(true, true, true)
MotorHolder.model = difference(MotorHolder.model,
  box(motor_w, motor_w, motor_h):center(true, true, true):at(0, 0, -m_insert_h))
for _, hole in ipairs(motor_screw_holes) do
  MotorHolder.model = difference(MotorHolder.model, hole)
end
MotorHolder.model = difference(MotorHolder.model, oval(axle_r, 2, 2 * motor_h):at(1, 0, 0))
MotorHolder.model = difference(MotorHolder.model,
  box(motor_w, motor_w, motor_h):center(true, true, true):at(motor_w / 2, 0, -m_insert_h))

-- ===========================
-- Assembly: Ring Cylinder with Motor Holder
-- ===========================

RingCylinder.motor = MotorHolder.model
    :at((total_hole_width + motor_w + overhang) / 2, 0, vertical_offset_of_overhang + (motor_h + m_insert_h) / 2)

RingCylinder.model = group("ring_cylinder", {
  RingCylinder.body,
  RingCylinder.platform,
  RingCylinder.overhang,
  RingCylinder.motor
})

-- ===========================
-- Magnet Insert Helper
-- ===========================

MagnetInsertHelper = {}
MagnetInsertHelper.model = difference(
  union(
    cylinder(outer_outer_radius, cap_height),
    box(square_size / 2, square_size + 1.8, cutout_height):center(true, true, true):at(0, 0, -cutout_height / 2)
  ),
  box(square_size, square_size, cylinder_height + 1):center(true, true, true)
)

-- ===========================
-- FUS Holder (Focused Ultrasound)
-- ===========================

FUSHolder = {}

FUSHolder.tube = difference(
  cylinder(center_hole_radius, total_height),
  cylinder(center_hole_radius - liquid_wall, total_height + liquid_wall)
)

FUSHolder.top = difference(
  cylinder(liquid_wall + transducer_diameter / 2, top_container_offset + transducer_height_val):at(0, 0, total_height),
  cylinder((transducer_diameter / 2) + 2 * liquid_wall, top_container_offset):at(0, 0, total_height),
  cylinder(transducer_diameter / 2, transducer_height_val):at(0, 0, total_height + top_container_offset)
)

FUSHolder.flange = difference(
  cylinder(2 * liquid_wall + center_hole_radius, 0.6 * liquid_wall):at(0, 0, total_height),
  cylinder(center_hole_radius - liquid_wall, liquid_wall):at(0, 0, total_height)
)

FUSHolder.bottom = difference(
  cylinder(center_hole_radius, liquid_wall):at(0, 0, -liquid_wall),
  cylinder(cover_slip_r - liquid_wall / 2, liquid_wall):at(0, 0, -liquid_wall),
  cylinder(cover_slip_r, cover_slip_h):at(0, 0, -liquid_wall)
)

FUSHolder.model = group("fus_holder", {
  FUSHolder.tube,
  FUSHolder.top,
  FUSHolder.flange,
  FUSHolder.bottom
})

-- ===========================
-- Active Model (as per original: cap_inner)
-- ===========================

local assembly = group("assembly", {
  RingCylinder.body,
  RingCylinderOuter.model,
  CapInner.model:material(material("water")),
})

ScriptCAD.register(assembly)

export_stl("cap_inner.stl", CapInner.model)

-- ===========================
-- View Configuration
-- ===========================

view({
  camera = "isometric",
  distance = 150,
  target = { 0, 0, cylinder_height / 2 },
  theme = "dark",
  axes = { show = true, size = 20 },
})

return ScriptCAD.serialize()
