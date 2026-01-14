-- Minimal parametric tube example using stdlib

local ScriptCAD = require("stdlib")

local config = {
  outer_radius = 15,
  inner_radius = 13,
  height = 20,
}

-- Create tube as cylinder difference (base at z=0)
local tube_outer = cylinder(config.outer_radius, config.height)
local tube_inner = cylinder(config.inner_radius, config.height + 2)

local tube = difference(tube_outer, tube_inner)
    :name("tube")

ScriptCAD.register(tube)

return ScriptCAD.serialize()
