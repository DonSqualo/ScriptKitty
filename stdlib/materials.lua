-- ScriptCAD Standard Library: Materials
-- Physical material definitions for simulation and rendering

local Materials = {}

-- Material database with physical properties
-- Properties are used for rendering (color) and physics (conductivity, etc.)
Materials.database = {
  copper = {
    name = "Copper",
    color = {0.72, 0.45, 0.20, 1.0},
    conductivity = 5.96e7,
    relative_permeability = 1.0,
  },

  air = {
    name = "Air",
    color = {0.9, 0.95, 1.0, 0.05},
    conductivity = 0,
    relative_permeability = 1.0,
  },
}

--- Create or retrieve a material
-- @param name Material name (from database) or custom name
-- @param properties Optional table of properties to override/set
-- @return Material object
function Materials.material(name, properties)
  local mat = {}

  local db_mat = Materials.database[string.lower(name)]
  if db_mat then
    for k, v in pairs(db_mat) do
      mat[k] = v
    end
  else
    mat.name = name
  end

  if properties then
    for k, v in pairs(properties) do
      mat[k] = v
    end
  end

  mat._type = "material"

  return mat
end

function material(name, properties)
  return Materials.material(name, properties)
end

return Materials
