-- Mittens Standard Library: Materials
-- Physical material definitions for rendering and multiphysics simulation
--
-- Properties:
--   Visual:  color (RGBA), roughness, metallic
--   EM:      permittivity (ε_r), permeability (μ_r), conductivity (σ S/m), loss_tangent
--   Acoustic: speed_of_sound (m/s), impedance (Rayl), attenuation (Np/m/MHz)
--   Thermal:  thermal_conductivity (W/m·K), specific_heat (J/kg·K), density (kg/m³)

local Materials = {}

-- =============================================================================
-- Material Database
-- =============================================================================

Materials.database = {
  -- =========================================================================
  -- Conductors
  -- =========================================================================
  copper = {
    name = "Copper",
    color = {0.72, 0.45, 0.20, 1.0},
    metallic = 1.0,
    roughness = 0.3,
    -- EM properties
    permittivity = 1.0,
    permeability = 1.0,
    conductivity = 5.96e7,  -- S/m (effectively PEC at RF)
    is_pec = true,          -- Treat as perfect conductor in FDTD
    -- Acoustic
    speed_of_sound = 4660,  -- m/s
    impedance = 41.6e6,     -- Rayl
    -- Thermal
    density = 8960,
    thermal_conductivity = 401,
    specific_heat = 385,
  },

  aluminum = {
    name = "Aluminum",
    color = {0.77, 0.79, 0.82, 1.0},
    metallic = 1.0,
    roughness = 0.4,
    -- EM
    permittivity = 1.0,
    permeability = 1.0,
    conductivity = 3.77e7,
    is_pec = true,
    -- Acoustic
    speed_of_sound = 6420,
    impedance = 17.4e6,
    -- Thermal
    density = 2700,
    thermal_conductivity = 237,
    specific_heat = 897,
  },

  gold = {
    name = "Gold",
    color = {1.0, 0.84, 0.0, 1.0},
    metallic = 1.0,
    roughness = 0.2,
    -- EM
    permittivity = 1.0,
    permeability = 1.0,
    conductivity = 4.1e7,
    is_pec = true,
    -- Acoustic
    speed_of_sound = 3240,
    impedance = 62.5e6,
    -- Thermal
    density = 19300,
    thermal_conductivity = 318,
    specific_heat = 129,
  },

  steel = {
    name = "Steel",
    color = {0.5, 0.5, 0.55, 1.0},
    metallic = 1.0,
    roughness = 0.5,
    -- EM (stainless, non-magnetic)
    permittivity = 1.0,
    permeability = 1.0,
    conductivity = 1.45e6,
    is_pec = true,
    -- Acoustic
    speed_of_sound = 5790,
    impedance = 45.0e6,
    -- Thermal
    density = 7800,
    thermal_conductivity = 50,
    specific_heat = 500,
  },

  -- =========================================================================
  -- Dielectrics - PCB / Electronics
  -- =========================================================================
  fr4 = {
    name = "FR4 (PCB Substrate)",
    color = {0.1, 0.4, 0.1, 1.0},
    metallic = 0.0,
    roughness = 0.8,
    -- EM
    permittivity = 4.4,
    permeability = 1.0,
    conductivity = 0,
    loss_tangent = 0.02,    -- tanδ at ~1GHz
    -- Acoustic
    speed_of_sound = 2900,
    impedance = 5.2e6,
    -- Thermal
    density = 1800,
    thermal_conductivity = 0.3,
    specific_heat = 1100,
  },

  rogers_4350 = {
    name = "Rogers RO4350B",
    color = {0.8, 0.75, 0.6, 1.0},
    metallic = 0.0,
    roughness = 0.6,
    -- EM (high-frequency PCB)
    permittivity = 3.48,
    permeability = 1.0,
    conductivity = 0,
    loss_tangent = 0.0037,
    -- Thermal
    density = 1860,
    thermal_conductivity = 0.69,
  },

  -- =========================================================================
  -- Dielectrics - Plastics
  -- =========================================================================
  polycarbonate = {
    name = "Polycarbonate",
    color = {0.85, 0.85, 0.85, 0.7},
    metallic = 0.0,
    roughness = 0.4,
    -- EM
    permittivity = 2.9,
    permeability = 1.0,
    conductivity = 0,
    loss_tangent = 0.01,
    -- Acoustic
    speed_of_sound = 2300,
    impedance = 2.76e6,
    attenuation = 0.8,  -- Np/m/MHz
    -- Thermal
    density = 1200,
    thermal_conductivity = 0.2,
    specific_heat = 1200,
  },

  pla = {
    name = "PLA (3D Print)",
    color = {0.3, 0.3, 0.3, 1.0},
    metallic = 0.0,
    roughness = 0.7,
    -- EM
    permittivity = 3.0,
    permeability = 1.0,
    conductivity = 0,
    loss_tangent = 0.02,
    -- Acoustic
    speed_of_sound = 2100,
    impedance = 2.52e6,
    -- Thermal
    density = 1250,
    thermal_conductivity = 0.13,
    specific_heat = 1800,
  },

  abs = {
    name = "ABS (3D Print)",
    color = {0.9, 0.9, 0.85, 1.0},
    metallic = 0.0,
    roughness = 0.6,
    -- EM
    permittivity = 2.8,
    permeability = 1.0,
    conductivity = 0,
    loss_tangent = 0.005,
    -- Acoustic
    speed_of_sound = 2100,
    impedance = 2.2e6,
    -- Thermal
    density = 1050,
    thermal_conductivity = 0.17,
    specific_heat = 1400,
  },

  ptfe = {
    name = "PTFE (Teflon)",
    color = {0.95, 0.95, 0.95, 1.0},
    metallic = 0.0,
    roughness = 0.3,
    -- EM (excellent low-loss dielectric)
    permittivity = 2.1,
    permeability = 1.0,
    conductivity = 0,
    loss_tangent = 0.0002,
    -- Acoustic
    speed_of_sound = 1350,
    impedance = 2.97e6,
    -- Thermal
    density = 2200,
    thermal_conductivity = 0.25,
    specific_heat = 1000,
  },

  -- =========================================================================
  -- Dielectrics - Glass / Ceramics
  -- =========================================================================
  glass = {
    name = "Borosilicate Glass",
    color = {0.8, 0.85, 0.9, 0.6},
    metallic = 0.0,
    roughness = 0.1,
    -- EM
    permittivity = 4.6,
    permeability = 1.0,
    conductivity = 0,
    loss_tangent = 0.004,
    -- Acoustic
    speed_of_sound = 5640,
    impedance = 12.6e6,
    -- Thermal
    density = 2230,
    thermal_conductivity = 1.2,
    specific_heat = 753,
  },

  alumina = {
    name = "Alumina (Al₂O₃)",
    color = {0.95, 0.95, 0.9, 1.0},
    metallic = 0.0,
    roughness = 0.5,
    -- EM
    permittivity = 9.8,
    permeability = 1.0,
    conductivity = 0,
    loss_tangent = 0.0001,
    -- Acoustic
    speed_of_sound = 10520,
    impedance = 41.0e6,
    -- Thermal
    density = 3900,
    thermal_conductivity = 30,
    specific_heat = 880,
  },

  pzt = {
    name = "PZT Ceramic",
    color = {0.3, 0.3, 0.35, 1.0},
    metallic = 0.0,
    roughness = 0.6,
    -- EM
    permittivity = 1700,
    permeability = 1.0,
    conductivity = 0,
    loss_tangent = 0.02,
    -- Acoustic
    speed_of_sound = 4350,
    impedance = 33.0e6,
    -- Piezoelectric
    piezo_d33 = 374e-12,
    piezo_d31 = -171e-12,
    -- Thermal
    density = 7600,
  },

  -- =========================================================================
  -- Fluids / Gases
  -- =========================================================================
  air = {
    name = "Air",
    color = {0.9, 0.95, 1.0, 0.02},
    metallic = 0.0,
    -- EM
    permittivity = 1.0006,  -- ~1.0
    permeability = 1.0,
    conductivity = 0,
    -- Acoustic
    speed_of_sound = 343,
    impedance = 413,
    -- Thermal
    density = 1.225,
    thermal_conductivity = 0.026,
    specific_heat = 1005,
  },

  water = {
    name = "Water",
    color = {0.2, 0.4, 0.8, 0.3},
    metallic = 0.0,
    -- EM
    permittivity = 80,      -- High at low freq, ~1.8 at optical
    permeability = 1.0,
    conductivity = 0.05,    -- Deionized
    loss_tangent = 0.04,
    -- Acoustic
    speed_of_sound = 1480,
    impedance = 1.48e6,
    attenuation = 0.002,
    -- Thermal
    density = 1000,
    thermal_conductivity = 0.6,
    specific_heat = 4186,
  },

  saline = {
    name = "Saline (0.9% NaCl)",
    color = {0.25, 0.45, 0.85, 0.3},
    metallic = 0.0,
    -- EM
    permittivity = 76,
    permeability = 1.0,
    conductivity = 1.5,     -- S/m
    -- Acoustic
    speed_of_sound = 1507,
    impedance = 1.55e6,
    -- Thermal
    density = 1025,
    thermal_conductivity = 0.6,
    specific_heat = 3900,
  },

  -- =========================================================================
  -- Magnetic Materials
  -- =========================================================================
  neodymium = {
    name = "Neodymium (NdFeB N52)",
    color = {0.6, 0.6, 0.65, 1.0},
    metallic = 0.8,
    roughness = 0.5,
    -- EM
    permittivity = 1.0,
    permeability = 1.05,
    conductivity = 6.25e5,
    -- Magnetic
    remanence = 1.45,       -- T
    coercivity = 900e3,     -- A/m
    max_energy = 420e3,     -- J/m³
    -- Thermal
    density = 7500,
  },

  ferrite = {
    name = "Ferrite (MnZn)",
    color = {0.2, 0.2, 0.2, 1.0},
    metallic = 0.0,
    roughness = 0.7,
    -- EM (frequency dependent - values at ~1MHz)
    permittivity = 12,
    permeability = 2000,    -- Initial permeability
    conductivity = 0.1,
    -- Thermal
    density = 4800,
  },

  -- =========================================================================
  -- Special / Virtual Materials
  -- =========================================================================
  pec = {
    name = "Perfect Electric Conductor",
    color = {0.8, 0.8, 0.0, 1.0},
    metallic = 1.0,
    -- EM
    permittivity = 1.0,
    permeability = 1.0,
    conductivity = math.huge,
    is_pec = true,
  },

  pmc = {
    name = "Perfect Magnetic Conductor",
    color = {0.0, 0.8, 0.8, 1.0},
    metallic = 1.0,
    -- EM (virtual - for boundary conditions)
    permittivity = 1.0,
    permeability = math.huge,
    conductivity = 0,
    is_pmc = true,
  },
}

-- Aliases
Materials.database.metal = Materials.database.pec
Materials.database.cu = Materials.database.copper
Materials.database.al = Materials.database.aluminum
Materials.database.teflon = Materials.database.ptfe
Materials.database.glass_borosilicate = Materials.database.glass


-- =============================================================================
-- Material API
-- =============================================================================

--- Create or retrieve a material
-- @param name Material name (from database) or custom name
-- @param properties Optional table of properties to override/set
-- @return Material object
function Materials.material(name, properties)
  local mat = {}

  -- Look up in database (case-insensitive)
  local db_mat = Materials.database[string.lower(name)]
  if db_mat then
    for k, v in pairs(db_mat) do
      mat[k] = v
    end
  else
    mat.name = name
    -- Defaults for unknown materials
    mat.color = {0.5, 0.5, 0.5, 1.0}
    mat.permittivity = 1.0
    mat.permeability = 1.0
    mat.conductivity = 0
  end

  -- Override with user properties
  if properties then
    for k, v in pairs(properties) do
      mat[k] = v
    end
  end

  mat._type = "material"
  return mat
end

--- Check if material is a conductor (for FDTD meshing)
function Materials.is_conductor(mat)
  if mat.is_pec then return true end
  if mat.conductivity and mat.conductivity > 1e6 then return true end
  return false
end

--- Check if material is a dielectric
function Materials.is_dielectric(mat)
  return not Materials.is_conductor(mat)
end

--- Get MEEP material expression for a material
-- @param mat Material object
-- @param freq_hz Optional frequency for dispersive materials
-- @return String of Python code for MEEP
function Materials.to_meep(mat, freq_hz)
  if not mat then return "mp.air" end

  -- Perfect conductor
  if mat.is_pec or (mat.conductivity and mat.conductivity > 1e6) then
    return "mp.metal"
  end

  -- Air / vacuum
  local eps = mat.permittivity or 1.0
  local mu = mat.permeability or 1.0
  local sigma = mat.conductivity or 0
  local tan_d = mat.loss_tangent

  if eps == 1.0 and mu == 1.0 and sigma == 0 and not tan_d then
    return "mp.air"
  end

  -- Build Medium constructor
  local args = {}
  if eps ~= 1.0 then
    table.insert(args, string.format("epsilon=%.6g", eps))
  end
  if mu ~= 1.0 then
    table.insert(args, string.format("mu=%.6g", mu))
  end
  if sigma > 0 then
    -- MEEP D_conductivity is in units of 2πf
    table.insert(args, string.format("D_conductivity=%.6e", sigma))
  end

  if #args == 0 then
    return "mp.air"
  end

  return "mp.Medium(" .. table.concat(args, ", ") .. ")"
end

--- Get material ID for voxel grid (0=air, 1=pec, 2+=dielectrics)
-- @param mat Material object
-- @return Integer material ID
function Materials.to_voxel_id(mat)
  if not mat then return 0 end  -- air

  if Materials.is_conductor(mat) then
    return 1  -- PEC
  end

  -- Dielectric - hash based on permittivity
  local eps = mat.permittivity or 1.0
  if eps < 1.5 then return 0 end  -- ~air
  if eps < 3.5 then return 2 end  -- low-k plastic
  if eps < 5.0 then return 3 end  -- FR4/glass
  if eps < 15 then return 4 end   -- ceramic
  return 5                         -- high-k
end

-- Global shortcut
function material(name, properties)
  return Materials.material(name, properties)
end

return Materials
