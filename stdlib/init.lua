-- ScriptCAD Standard Library
-- Main entry point that loads all modules

local ScriptCAD = {}

-- Load all modules
ScriptCAD.primitives = require("stdlib.primitives")
ScriptCAD.transforms = require("stdlib.transforms")
ScriptCAD.materials = require("stdlib.materials")
ScriptCAD.csg = require("stdlib.csg")
ScriptCAD.groups = require("stdlib.groups")
ScriptCAD.instruments = require("stdlib.instruments")
ScriptCAD.physics = require("stdlib.physics")
ScriptCAD.view = require("stdlib.view")
ScriptCAD.export = require("stdlib.export")
ScriptCAD.circuits = require("stdlib.circuits")

-- Export primitive functions to global scope
box = ScriptCAD.primitives.box
cylinder = ScriptCAD.primitives.cylinder
sphere = ScriptCAD.primitives.sphere
torus = ScriptCAD.primitives.torus
ring = ScriptCAD.primitives.ring
wedge = ScriptCAD.primitives.wedge

-- Export transform functions
translate = ScriptCAD.transforms.translate
rotate = ScriptCAD.transforms.rotate
scale = ScriptCAD.transforms.scale
mirror = ScriptCAD.transforms.mirror
linear_pattern = ScriptCAD.transforms.linear_pattern
circular_pattern = ScriptCAD.transforms.circular_pattern
Vec3 = ScriptCAD.transforms.Vec3
Mat4 = ScriptCAD.transforms.Mat4

-- Export CSG functions
union = ScriptCAD.csg.union
difference = ScriptCAD.csg.difference
intersect = ScriptCAD.csg.intersect

-- Export group functions
group = ScriptCAD.groups.group
assembly = ScriptCAD.groups.assembly
component = ScriptCAD.groups.component

-- Export material functions
material = ScriptCAD.materials.material

-- Export view functions
view = ScriptCAD.view.view

-- Export physics functions
magnetostatic = ScriptCAD.physics.magnetostatic
acoustic = ScriptCAD.physics.acoustic
acoustic_source = ScriptCAD.physics.acoustic_source
acoustic_boundary = ScriptCAD.physics.acoustic_boundary
current_source = ScriptCAD.physics.current_source
linspace = ScriptCAD.physics.linspace
logspace = ScriptCAD.physics.logspace

-- Export instrument functions
Probe = ScriptCAD.instruments.Probe
GaussMeter = ScriptCAD.instruments.GaussMeter
MagneticFieldPlane = ScriptCAD.instruments.MagneticFieldPlane
AcousticPressurePlane = ScriptCAD.instruments.AcousticPressurePlane
Hydrophone = ScriptCAD.instruments.Hydrophone

-- Export file functions
export_stl = ScriptCAD.export.export_stl
export_3mf = ScriptCAD.export.export_3mf

-- Export circuit functions
SignalGenerator = ScriptCAD.circuits.SignalGenerator
Amplifier = ScriptCAD.circuits.Amplifier
MatchingNetwork = ScriptCAD.circuits.MatchingNetwork
TransducerLoad = ScriptCAD.circuits.TransducerLoad
Circuit = ScriptCAD.circuits.Circuit


-- Scene registry
ScriptCAD._scene = {
  objects = {},
  instruments = {},
  studies = {},
  exports = {},
}

--- Register an object in the scene
-- @param obj Object to register
function ScriptCAD.register(obj)
  if obj._type == "instrument" then
    table.insert(ScriptCAD._scene.instruments, obj)
  elseif obj._type == "study" then
    table.insert(ScriptCAD._scene.studies, obj)
  elseif obj._type == "visualization" then
    table.insert(ScriptCAD._scene.instruments, obj)
  else
    table.insert(ScriptCAD._scene.objects, obj)
  end
end

--- Get the full scene for rendering
-- @return Scene data
function ScriptCAD.get_scene()
  return {
    objects = ScriptCAD._scene.objects,
    instruments = ScriptCAD.instruments.serialize_all(),
    studies = ScriptCAD.physics.get_studies(),
    exports = ScriptCAD.export.serialize(),
    view = ScriptCAD.view.serialize(),
  }
end

--- Serialize the entire scene to JSON-compatible format
-- @return Serialized scene
function ScriptCAD.serialize()
  local scene = ScriptCAD.get_scene()

  -- Serialize objects
  local objects_serialized = {}
  for i, obj in ipairs(scene.objects) do
    if obj.serialize then
      objects_serialized[i] = obj:serialize()
    end
  end

  -- Serialize studies
  local studies_serialized = {}
  for i, study in ipairs(scene.studies) do
    if study.serialize then
      studies_serialized[i] = study:serialize()
    end
  end

  return {
    objects = objects_serialized,
    instruments = scene.instruments,
    studies = studies_serialized,
    exports = scene.exports,
    view = scene.view,
  }
end

--- Clear the scene
function ScriptCAD.clear()
  ScriptCAD._scene = {
    objects = {},
    instruments = {},
    studies = {},
    exports = {},
  }
  ScriptCAD.instruments.clear()
  ScriptCAD.physics.clear()
  ScriptCAD.export.clear()
  ScriptCAD.view.reset()
end

-- Version info
ScriptCAD.VERSION = "0.1.0"
ScriptCAD.NAME = "ScriptCAD"

return ScriptCAD
