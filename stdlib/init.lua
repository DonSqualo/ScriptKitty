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

-- Export primitive functions to global scope
box = ScriptCAD.primitives.box
cylinder = ScriptCAD.primitives.cylinder

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
smooth_union = ScriptCAD.csg.smooth_union
shell = ScriptCAD.csg.shell

-- Export group functions
group = ScriptCAD.groups.group
assembly = ScriptCAD.groups.assembly
component = ScriptCAD.groups.component

-- Export material functions
material = ScriptCAD.materials.material

-- Export view functions
view = ScriptCAD.view.view

-- Export physics functions
electromagnetic = ScriptCAD.physics.electromagnetic
electrostatic = ScriptCAD.physics.electrostatic
magnetostatic = ScriptCAD.physics.magnetostatic
thermal = ScriptCAD.physics.thermal
thermal_transient = ScriptCAD.physics.thermal_transient
structural = ScriptCAD.physics.structural
multiphysics = ScriptCAD.physics.multiphysics
port = ScriptCAD.physics.port
current_source = ScriptCAD.physics.current_source
voltage_source = ScriptCAD.physics.voltage_source
heat_source = ScriptCAD.physics.heat_source
linspace = ScriptCAD.physics.linspace
logspace = ScriptCAD.physics.logspace

-- Export instrument functions
Probe = ScriptCAD.instruments.Probe
Oscilloscope = ScriptCAD.instruments.Oscilloscope
GaussMeter = ScriptCAD.instruments.GaussMeter
Thermometer = ScriptCAD.instruments.Thermometer
ForceSensor = ScriptCAD.instruments.ForceSensor
MagneticFieldPlane = ScriptCAD.instruments.MagneticFieldPlane
ElectricFieldPlane = ScriptCAD.instruments.ElectricFieldPlane
TemperaturePlane = ScriptCAD.instruments.TemperaturePlane
Streamlines = ScriptCAD.instruments.Streamlines
Isosurface = ScriptCAD.instruments.Isosurface
SParams = ScriptCAD.instruments.SParams

-- Export file functions
export_stl = ScriptCAD.export.export_stl
export_step = ScriptCAD.export.export_step
export_gltf = ScriptCAD.export.export_gltf
export_obj = ScriptCAD.export.export_obj

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
