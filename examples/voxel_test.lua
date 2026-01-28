-- voxel_test.lua
-- Test voxelized MEEP export

local Mittens = require("stdlib")

-- Config triggers voxel export
config = {
  voxel_size = 0.5,    -- 0.5mm voxels
  freq_start = 1e9,    -- 1 GHz
  freq_stop = 10e9,    -- 10 GHz
}

-- Materials
local copper = material("copper")
local fr4 = material("fr4")

-- Simple bridge gap resonator
local left_pad = box(10, 5, 0.5)
    :at(-6, 0, 0)
    :material(copper)
    :name("left_pad")

local right_pad = box(10, 5, 0.5)
    :at(6, 0, 0)
    :material(copper)
    :name("right_pad")

local bridge = box(4, 3, 0.25)
    :at(0, 0, 0.5)
    :material(copper)
    :name("bridge")

local assembly = group("resonator", {
    left_pad,
    right_pad,
    bridge
})

Mittens.register(assembly)

view({
  camera = {
    position = {25, 20, 15},
    target = {0, 0, 0},
    fov = 50
  }
})

return Mittens.serialize()
