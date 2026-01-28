-- meep_test.lua
-- Simple test for MEEP export

local Mittens = require("stdlib")

-- This triggers MEEP generation
local config = {
  freq_start = 1e9,  -- 1 GHz
  freq_stop = 10e9,  -- 10 GHz
}

-- Simple geometry
local copper = material("copper")
local fr4 = material("FR4", { permittivity = 4.4 })

-- Left pad
local left_pad = box(25, 8, 0.5)
    :at(-13.75, 0, 0.25)
    :material(copper)
    :name("left_pad")

-- Right pad
local right_pad = box(25, 8, 0.5)
    :at(13.75, 0, 0.25)
    :material(copper)
    :name("right_pad")

-- Bridge
local bridge = box(6.5, 4.8, 0.25)
    :at(0, 0, 0.625)
    :material(copper)
    :name("bridge")

-- Group
local assembly = group("resonator", {
    left_pad,
    right_pad,
    bridge
})

Mittens.register(assembly)

return Mittens.serialize()
