-- pure_acoustics.lua
-- Ultrasound cell culture study setup
--
-- Components:
-- 1. Interchangeable Coverslip Dish (ICD) system
--    - Metal threaded base (stainless steel)
--    - Polycarbonate tube
--    - Silicone O-ring
--    - 30mm glass coverslip
-- 2. 96-well plate holder adapter
-- 3. Custom lid with ultrasound transducer holder

local ScriptCAD = require("stdlib")

-- ============================================================================
-- Configuration Parameters
-- ============================================================================

Coverslip = {
  diameter = 30,
  thickness = 0.17, -- #1.5 thickness
  clear_aperture = 25,
}

Oring = {
  outer_diameter = 28,
  inner_diameter = 24,
  thickness = 1.5,
}

MetalBase = {
  outer_diameter = 35,
  inner_diameter = 30.5,
  height = 5,
  lip_height = 1.5,
  lip_inner_diameter = 26,
}

PolyTube = {
  outer_diameter = 30,
  inner_diameter = 26,
  height = 10,
}

HolderAdapter = {
  length = 127.76,
  width = 85.48,
  height = 8,
  dish_cutout = 36,
}

Lid = {
  diameter = 34,
  height = 15,
  wall_thickness = 1.5,
}

Transducer = {
  diameter = 20,
  thickness = 2,
  holder_depth = 5,
}
Coverslip.model = cylinder(Coverslip.diameter / 2, Coverslip.thickness)

local oring_z = Coverslip.thickness
Oring.model = difference(
      cylinder(Oring.outer_diameter / 2, Oring.thickness),
      cylinder(Oring.inner_diameter / 2, Oring.thickness)
    )
    :at(0, 0, oring_z)

-- ============================================================================
-- Geometry: Metal Threaded Base (Stainless Steel)
-- ============================================================================

local base_z = oring_z + Oring.thickness

MetalBase.body = difference(
      cylinder(MetalBase.outer_diameter / 2, MetalBase.height),
      cylinder(MetalBase.inner_diameter / 2, MetalBase.height + 1)
    )
    :at(0, 0, base_z)

MetalBase.lip = difference(
      cylinder(MetalBase.inner_diameter / 2, MetalBase.lip_height),
      cylinder(MetalBase.lip_inner_diameter / 2, MetalBase.lip_height + 1)
    )
    :at(0, 0, base_z)

MetalBase.model = group("metal_base", { MetalBase.body, MetalBase.lip })

-- ============================================================================
-- Geometry: Polycarbonate Tube
-- ============================================================================

local tube_z = base_z + MetalBase.height

PolyTube.model = difference(
      cylinder(PolyTube.outer_diameter / 2, PolyTube.height),
      cylinder(PolyTube.inner_diameter / 2, PolyTube.height + 1)
    )
    :at(0, 0, tube_z)

-- ============================================================================
-- Assembly: Interchangeable Coverslip Dish
-- ============================================================================

local icd_assembly = group("icd_assembly", {
  Coverslip.model,
  Oring.model,
  MetalBase.model,
  PolyTube.model,
})

-- ============================================================================
-- Geometry: 96-Well Plate Holder Adapter
-- ============================================================================

local holder_z = -HolderAdapter.height

HolderAdapter.body = box(HolderAdapter.length, HolderAdapter.width, HolderAdapter.height)
    :at(0, 0, HolderAdapter.height / 2)

HolderAdapter.dish_cutout = cylinder(HolderAdapter.dish_cutout / 2, HolderAdapter.height + 2)

HolderAdapter.edge_cutout = box(HolderAdapter.length / 3, HolderAdapter.width * 0.6, HolderAdapter.height + 2)
    :at(HolderAdapter.length / 3, 0, HolderAdapter.height / 2)

HolderAdapter.model = difference(
      HolderAdapter.body,
      HolderAdapter.dish_cutout,
      HolderAdapter.edge_cutout
    )
    :at(0, 0, holder_z)

-- ============================================================================
-- Geometry: Custom Lid with Ultrasound Transducer Holder
-- ============================================================================

local lid_z = tube_z + PolyTube.height

Lid.outer = cylinder(Lid.diameter / 2, Lid.height)

Lid.inner = cylinder(
      Lid.diameter / 2 - Lid.wall_thickness,
      Lid.height - Lid.wall_thickness
    )

Lid.transducer_pocket = cylinder(
      Transducer.diameter / 2 + 0.5,
      Transducer.holder_depth
    )
    :at(0, 0, Lid.height - Transducer.holder_depth)

Lid.model = difference(
      Lid.outer,
      Lid.inner,
      Lid.transducer_pocket
    )
    :at(0, 0, lid_z)

-- ============================================================================
-- Geometry: Ultrasound Transducer Disk
-- ============================================================================

local transducer_z = lid_z + Lid.height - Transducer.holder_depth

Transducer.model = cylinder(Transducer.diameter / 2, Transducer.thickness)
    :at(0, 0, transducer_z)

-- ============================================================================
-- Full Assembly
-- ============================================================================

local assembly = group("pure_acoustics", {
  icd_assembly,
  HolderAdapter.model,
  Lid.model,
  Transducer.model,
})

ScriptCAD.register(assembly)

-- ============================================================================
-- View Configuration
-- ============================================================================

view({
  camera = "isometric",
  distance = 150,
  target = { 0, 0, PolyTube.height / 2 },
  theme = "dark",
  axes = { show = true, size = 20 },
})

return ScriptCAD.serialize()
