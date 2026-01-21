-- ScriptCAD Standard Library: Materials
-- Physical material definitions for simulation and rendering

local Materials = {}

-- Material database with physical properties
-- Properties are used for rendering (color) and physics (conductivity, acoustic, etc.)
-- Acoustic: speed_of_sound (m/s), impedance (Rayl), attenuation (Np/m at 1MHz)
Materials.database = {
  copper = {
    name = "Copper",
    color = {0.72, 0.45, 0.20, 1.0},
    conductivity = 5.96e7,
    relative_permeability = 1.0,
    speed_of_sound = 4660,
    impedance = 41.6e6,
  },

  air = {
    name = "Air",
    color = {0.9, 0.95, 1.0, 0.05},
    conductivity = 0,
    relative_permeability = 1.0,
    speed_of_sound = 343,
    impedance = 413,
  },

  water = {
    name = "Water",
    color = {0.2, 0.4, 0.8, 0.3},
    conductivity = 0.05,
    relative_permeability = 1.0,
    speed_of_sound = 1480,
    impedance = 1.48e6,
    attenuation = 0.002,
  },

  glass_borosilicate = {
    name = "Borosilicate Glass",
    color = {0.8, 0.85, 0.9, 0.6},
    conductivity = 0,
    relative_permeability = 1.0,
    relative_permittivity = 4.6,
    speed_of_sound = 5640,
    impedance = 12.6e6,
  },

  pzt = {
    name = "PZT Ceramic (Lead Zirconate Titanate)",
    color = {0.3, 0.3, 0.35, 1.0},
    conductivity = 0,
    relative_permeability = 1.0,
    relative_permittivity = 1700,
    speed_of_sound = 4350,
    impedance = 33.0e6,
    piezo_d33 = 374e-12,
    piezo_d31 = -171e-12,
  },

  polycarbonate = {
    name = "Polycarbonate",
    color = {0.85, 0.85, 0.85, 0.7},
    conductivity = 0,
    relative_permeability = 1.0,
    relative_permittivity = 2.9,
    speed_of_sound = 2300,
    impedance = 2.76e6,
    attenuation = 0.8,
  },

  pla = {
    name = "PLA (Polylactic Acid)",
    color = {0.3, 0.3, 0.3, 1.0},
    conductivity = 0,
    relative_permeability = 1.0,
    relative_permittivity = 3.0,
    speed_of_sound = 2100,
    impedance = 2.52e6,
  },

  ptfe = {
    name = "PTFE (Teflon)",
    color = {0.95, 0.95, 0.95, 1.0},
    conductivity = 0,
    relative_permeability = 1.0,
    relative_permittivity = 2.1,
    speed_of_sound = 1350,
    impedance = 2.97e6,
  },

  aluminum = {
    name = "Aluminum",
    color = {0.77, 0.79, 0.82, 1.0},
    conductivity = 3.77e7,
    relative_permeability = 1.0,
    speed_of_sound = 6420,
    impedance = 17.4e6,
  },

  neodymium = {
    name = "Neodymium (NdFeB)",
    color = {0.6, 0.6, 0.65, 1.0},
    conductivity = 6.25e5,
    relative_permeability = 1.05,
    remanence = 1.2,
    coercivity = 900e3,
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
