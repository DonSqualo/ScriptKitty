-- ScriptCAD Standard Library: Export
-- File export functions for various CAD formats

local Export = {}

-- Export queue
Export._queue = {}

--- Export to STL format (3D printing)
-- @param filename Output filename
-- @param object Object or group to export
-- @param config {binary, quality}
function Export.export_stl(filename, object, config)
  config = config or {}
  table.insert(Export._queue, {
    format = "stl",
    filename = filename,
    object = object,
    binary = config.binary ~= false,  -- default true
    quality = config.quality or "high",  -- low, medium, high
    tolerance = config.tolerance or 0.01,  -- mm
  })
end

--- Export to STEP format (CAD interchange)
-- @param filename Output filename
-- @param object Object or group to export
-- @param config {version, units}
function Export.export_step(filename, object, config)
  config = config or {}
  table.insert(Export._queue, {
    format = "step",
    filename = filename,
    object = object,
    version = config.version or "AP214",  -- AP203, AP214, AP242
    units = config.units or "mm",
  })
end

--- Export to IGES format (legacy CAD)
-- @param filename Output filename
-- @param object Object or group to export
function Export.export_iges(filename, object, config)
  config = config or {}
  table.insert(Export._queue, {
    format = "iges",
    filename = filename,
    object = object,
    units = config.units or "mm",
  })
end

--- Export to OBJ format (3D graphics)
-- @param filename Output filename
-- @param object Object or group to export
-- @param config {mtl, normals}
function Export.export_obj(filename, object, config)
  config = config or {}
  table.insert(Export._queue, {
    format = "obj",
    filename = filename,
    object = object,
    include_mtl = config.mtl ~= false,
    include_normals = config.normals ~= false,
  })
end

--- Export to glTF format (web/realtime 3D)
-- @param filename Output filename
-- @param object Object or group to export
-- @param config {binary, draco}
function Export.export_gltf(filename, object, config)
  config = config or {}
  table.insert(Export._queue, {
    format = config.binary and "glb" or "gltf",
    filename = filename,
    object = object,
    draco_compression = config.draco or false,
    include_materials = config.materials ~= false,
  })
end

--- Export to 3MF format (modern 3D printing)
-- @param filename Output filename
-- @param object Object or group to export
-- @param config {units, color}
function Export.export_3mf(filename, object, config)
  config = config or {}
  table.insert(Export._queue, {
    format = "3mf",
    filename = filename,
    object = object,
    units = config.units or "mm",
    include_colors = config.color ~= false,
  })
end

--- Export SDF field data to VTK format (ParaView compatible)
-- @param filename Output filename
-- @param bounds {min, max} bounding box
-- @param config {resolution, fields}
function Export.export_vtk(filename, bounds, config)
  config = config or {}
  table.insert(Export._queue, {
    format = "vtk",
    filename = filename,
    bounds = bounds,
    resolution = config.resolution or {50, 50, 50},
    fields = config.fields or {"sdf"},  -- sdf, E_field, H_field, temperature
    binary = config.binary or true,
  })
end

--- Export simulation results
-- @param filename Output filename
-- @param study Study object
-- @param config {format, fields}
function Export.export_results(filename, study, config)
  config = config or {}
  table.insert(Export._queue, {
    format = config.format or "csv",  -- csv, json, hdf5
    filename = filename,
    study = study,
    fields = config.fields or "all",
  })
end

--- Export S-parameters to Touchstone format
-- @param filename Output filename
-- @param study EM study object
-- @param config {format}
function Export.export_touchstone(filename, study, config)
  config = config or {}
  table.insert(Export._queue, {
    format = "touchstone",
    filename = filename,
    study = study,
    touchstone_format = config.format or "s2p",  -- s1p, s2p, snp
    impedance = config.impedance or 50,
  })
end

--- Get export queue
-- @return Table of pending exports
function Export.get_queue()
  return Export._queue
end

--- Clear export queue
function Export.clear()
  Export._queue = {}
end

--- Serialize export queue for processing
function Export.serialize()
  local result = {}
  for i, exp in ipairs(Export._queue) do
    local entry = {}
    for k, v in pairs(exp) do
      if k == "object" or k == "study" then
        -- Serialize nested objects
        if v and v.serialize then
          entry[k] = v:serialize()
        else
          entry[k] = v
        end
      else
        entry[k] = v
      end
    end
    result[i] = entry
  end
  return result
end

-- Global shortcuts
export_stl = Export.export_stl
export_step = Export.export_step
export_iges = Export.export_iges
export_obj = Export.export_obj
export_gltf = Export.export_gltf
export_3mf = Export.export_3mf
export_vtk = Export.export_vtk
export_results = Export.export_results
export_touchstone = Export.export_touchstone

return Export
