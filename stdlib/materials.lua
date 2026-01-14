-- ScriptCAD Standard Library: Materials
-- Physical material definitions for simulation and rendering

local Materials = {}

-- Material database with physical properties
Materials.database = {
  copper = {
    name = "Copper",
    color = {0.72, 0.45, 0.20, 1.0},
    conductivity = 5.96e7,
    thermal_conductivity = 401,
    density = 8960,
    permeability = 0.999994,
    permittivity = 1.0,
    youngs_modulus = 130e9,
    poissons_ratio = 0.34,
    yield_strength = 70e6,
  },

  fr4 = {
    name = "FR4 (PCB)",
    color = {0.1, 0.4, 0.1, 0.8},
    conductivity = 0,
    thermal_conductivity = 0.3,
    density = 1850,
    permeability = 1.0,
    permittivity = 4.4,
    loss_tangent = 0.02,
    youngs_modulus = 24e9,
    poissons_ratio = 0.12,
  },

  air = {
    name = "Air",
    color = {0.9, 0.95, 1.0, 0.05},
    conductivity = 0,
    thermal_conductivity = 0.026,
    density = 1.225,
    permeability = 1.0,
    permittivity = 1.0006,
  },

  steel = {
    name = "Steel",
    color = {0.15, 0.15, 0.17, 1.0},
    conductivity = 1.45e6,
    thermal_conductivity = 50,
    density = 7850,
    permeability = 100,
    permittivity = 1.0,
    youngs_modulus = 200e9,
    poissons_ratio = 0.3,
    yield_strength = 250e6,
  },

  water = {
    name = "Water",
    color = {0.2, 0.8, 0.8, 1.0},
    conductivity = 0.0005,
    thermal_conductivity = 0.6,
    density = 1000,
    permeability = 1.0,
    permittivity = 80,
    speed_of_sound = 1480,
    viscosity = 0.001,
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
