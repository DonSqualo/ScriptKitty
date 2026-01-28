# MEEP Integration Spec

Full-wave FDTD electromagnetic simulation via MEEP, integrated into the Mittens pipeline.

**Goal:** Compute resonant frequency of a bridged loop-gap resonator and visualize EM fields server-side.

---

## Overview

```
Lua Script (.lua)
      │
      ▼
┌──────────────────────────────────────────────────────────────────────┐
│                         Rust Server                                   │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ │
│  │ Lua Runner  │─▶│ Manifold3D  │─▶│ MEEP        │─▶│ WebSocket   │ │
│  │ (mlua)      │  │ (mesh gen)  │  │ (EM sim)    │  │ (results)   │ │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘ │
└──────────────────────────────────────────────────────────────────────┘
      │
      ▼
┌──────────────────────────────────────────────────────────────────────┐
│                         Renderer                                      │
│  • Full 3D geometry (existing)                                       │
│  • NanoVNA S11 plot (MEEP-computed)                                  │
│  • 2D B-field colormap (XY plane)                                    │
│  • Oscilloscope (Gauss probe at 0,0,0)                              │
└──────────────────────────────────────────────────────────────────────┘
```

**Key constraint:** All MEEP computation happens server-side in Rust. No Python subprocess. No external scripts.

---

## Rust MEEP Binding

### Crate: `meep-sys` (FFI)

MEEP is a C++ library with a C API. Bind via `bindgen` or manual FFI.

Required MEEP functions:
```rust
// Geometry
meep_geometry_add_block(center, size, material)
meep_geometry_add_cylinder(center, radius, height, axis, material)

// Materials
meep_material_lorentzian(epsilon, frequencies, sigmas)
meep_material_metal(conductivity)

// Simulation
meep_simulation_new(cell_size, resolution, boundary_layers)
meep_simulation_add_source(source_type, center, size, frequency, width)
meep_simulation_run_until(time)

// Field output
meep_get_field_at_point(component, point) -> f64
meep_get_eigenmode_frequency(band) -> f64
meep_get_flux(monitor) -> f64
```

### Alternative: `libmeep` Rust crate

If direct FFI is painful, consider wrapping MEEP's Python bindings via PyO3 as a fallback. But prefer pure FFI for performance.

---

## Geometry Pipeline

### 1. Lua defines geometry (existing)

```lua
-- Loop gap resonator
resonator = LoopGapResonator({
    outer_radius = 15,  -- mm
    inner_radius = 12,
    height = 20,
    gap_width = 2,
    gap_angle = 0,      -- degrees
    material = "copper"
})

-- Coupling coil
coupling_coil = Coil({
    mean_radius = 18,
    turns = 1,
    wire_diameter = 0.8,
    position = {0, 0, 25}
})

-- Helmholtz coils (existing)
helmholtz = HelmholtzCoils({
    mean_radius = 50,
    gap = 42.5,
    windings = 120,
    current = 2.0
})
```

### 2. Manifold3D generates mesh (existing)

Server already converts Lua geometry to triangle meshes via Manifold3D.

### 3. MEEP ingests mesh

New: `server/src/meep.rs`

```rust
pub struct MeepSimulation {
    sim: *mut meep_simulation,
    cell_size: [f64; 3],
    resolution: f64,
}

impl MeepSimulation {
    /// Convert Manifold mesh to MEEP geometry
    pub fn add_mesh(&mut self, mesh: &TriangleMesh, material: Material) {
        // Use meep's `material_function` or voxelize the mesh
        // For simple geometries, approximate with primitives
    }
    
    /// Add source (Gaussian pulse for broadband)
    pub fn add_source(&mut self, center: [f64; 3], frequency: f64, bandwidth: f64) {
        // meep_simulation_add_source(...)
    }
    
    /// Run simulation and extract eigenfrequencies
    pub fn compute_resonance(&mut self) -> Vec<f64> {
        // Run harminv (built into MEEP) for resonance detection
    }
    
    /// Get S11 at frequency
    pub fn compute_s11(&mut self, freq: f64) -> Complex<f64> {
        // Flux monitors for incident/reflected power
    }
    
    /// Sample E/H field at point over time
    pub fn probe_field(&mut self, point: [f64; 3], duration: f64) -> Vec<f64> {
        // meep_get_field_at_point at each timestep
    }
}
```

---

## Lua API Extension

```lua
-- Trigger MEEP simulation
MeepStudy({
    geometry = { resonator, coupling_coil },  -- objects to simulate
    frequency_center = 450e6,                 -- Hz
    frequency_width = 100e6,                  -- Hz (for broadband pulse)
    resolution = 10,                          -- pixels per wavelength
    boundary = "pml",                         -- absorbing boundaries
    
    -- Outputs
    compute_resonance = true,                 -- find eigenfrequencies
    compute_s11 = true,                       -- S-parameter sweep
    field_probe = {0, 0, 0},                  -- point to monitor
    field_plane = "xy",                       -- 2D slice for visualization
})
```

---

## Renderer Outputs

### 1. NanoVNA S11 Plot (existing widget, new data source)

Currently `nanovna.rs` uses Wheeler formula approximations. 

**Change:** When `MeepStudy` is present, use MEEP-computed S11 instead.

```rust
// In main.rs
if has_meep_study {
    let s11_data = meep_sim.compute_s11_sweep(f_start, f_end, n_points);
    broadcast_nanovna(s11_data);
} else {
    // Fallback to Wheeler formula
    let s11_data = nanovna::compute_wheeler_s11(...);
    broadcast_nanovna(s11_data);
}
```

### 2. 2D Magnetic Field Colormap (existing widget, new data)

Existing `MagneticFieldPlane` shows static B-field from Helmholtz coils.

**Add:** RF B-field from MEEP at resonant frequency.

```lua
MagneticFieldPlane({
    plane = "xy",
    z = 0,
    size = {100, 100},
    source = "meep",  -- NEW: use MEEP-computed field instead of Biot-Savart
    frequency = 450e6 -- at this frequency
})
```

### 3. Oscilloscope (new widget)

Displays time-domain field strength at a point (Gauss probe).

**Binary protocol:** `OSCOPE\0\0` (8 bytes header)

```rust
struct OscopeData {
    sample_rate: f32,    // Hz
    n_samples: u32,
    samples: Vec<f32>,   // field magnitude over time
    
    // Derived quantities
    dc_component: f32,   // mT (static field from Helmholtz)
    ac_amplitude: f32,   // mT (RF field amplitude)
    ac_frequency: f32,   // Hz (detected oscillation frequency)
}
```

**Renderer:** Canvas-based waveform display, similar to existing line plots.

**Expected values:**
- DC: ~16 mT (from Helmholtz coils)
- AC: ~0.5 mT peak-to-peak at ~450-460 MHz

---

## Success Criteria (Screenshot Test)

Final screenshot shows:

1. ✓ Full Helmholtz coil geometry (existing)
2. ✓ Loop gap resonator visible (existing primitives)
3. ✓ Coupling coil visible (existing primitives)
4. ✓ NanoVNA widget with S11 dip at resonant frequency
5. ✓ 2D B-field colormap on XY plane showing RF field pattern
6. ✓ Oscilloscope showing 16 ± 0.5 mT with RF modulation

Ralph validates by comparing against reference screenshot.

---

## Implementation Phases

### Phase 1: MEEP FFI Binding

1. Add `meep-sys` crate with bindgen
2. Implement basic simulation lifecycle (new/add_block/run/destroy)
3. Test with simple dielectric cube

**Files:** `crates/meep-sys/`, `server/Cargo.toml`

### Phase 2: Geometry Integration

1. Convert Manifold mesh → MEEP geometry (voxelization or primitive approximation)
2. Add material database (copper conductivity, air permittivity)
3. Add `lorentzian_susceptibility` for dispersive materials

**Files:** `server/src/meep.rs`

### Phase 3: Resonance Detection

1. Add Gaussian pulse source
2. Implement `harminv` integration for eigenfrequency extraction
3. Wire to NanoVNA display

**Files:** `server/src/meep.rs`, `server/src/main.rs`

### Phase 4: Field Visualization

1. Add field plane extraction from MEEP
2. Add time-domain probe for oscilloscope
3. Implement oscilloscope renderer widget

**Files:** `server/src/meep.rs`, `renderer/src/main.ts`

### Phase 5: Integration & Testing

1. Full pipeline test with `helmholtz_coil.lua`
2. Screenshot comparison test
3. Performance optimization (resolution vs accuracy tradeoff)

---

## Dependencies

### System
```bash
# Ubuntu/Debian
sudo apt install libmeep-dev libharminv-dev libhdf5-dev

# Or build from source
git clone https://github.com/NanoComp/meep
cd meep && ./configure && make && sudo make install
```

### Cargo
```toml
[dependencies]
meep-sys = { path = "../crates/meep-sys" }  # Local FFI bindings

[build-dependencies]
bindgen = "0.69"
```

---

## Notes

- MEEP uses normalized units internally. Conversion: `f_Hz = f_meep × c / a` where `a` is the length unit.
- For RF simulations at ~450 MHz (λ ≈ 0.67m), the cell needs to be several wavelengths. Use subpixel smoothing for accuracy.
- PML (perfectly matched layers) absorb outgoing waves — essential for resonator simulations.
- MEEP's `harminv` function extracts resonant frequencies from time-domain data automatically.

---

## References

- [MEEP Documentation](https://meep.readthedocs.io/)
- [MEEP C++ API](https://github.com/NanoComp/meep/blob/master/src/meep.hpp)
- Oskooi et al., "MEEP: A flexible free-software package for electromagnetic simulations by the FDTD method" (2010)
