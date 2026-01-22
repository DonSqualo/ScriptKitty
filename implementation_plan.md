# Implementation Plan

ScriptKitty v0.0.12 - Prioritized implementation backlog (2026-01-22).

# Very High Priority

### NanoVNA Coupled Resonator Simulation
Current `nanovna.rs` uses Wheeler formula with frequency-dependent corrections. Still no connection to actual geometry for inductance computation.

**Now Implemented:**
- Wheeler formula + Nagaoka correction for inductance
- Skin effect: `R_ac = R_dc·√(1 + (ω/ω_skin)²)` for frequency-dependent resistance
- Parasitic capacitance between turns (Medhurst approximation)
- Self-resonant frequency (SRF) calculation: `f_SRF = 1/(2π√(LC_parasitic))`
- S11 conversion with frequency-dependent impedance

**Still Missing:**
- No mutual inductance M between drive coil and resonator gap
- No geometry-aware inductance (reads hardcoded params, not actual mesh)
- No sample environment coupling (biological tissue ε_r ≈ 80)

**Remaining Roadmap:**
1. **Geometry-Aware Inductance**: Extract coil coordinates from mesh, compute L from field energy `L = 2W_m/I²`
2. **Mutual Inductance**: Compute M between drive coil and resonator gap via B-field integration
3. **Coupled Impedance**: `Z_in = jωL_drive + (ωM)²/(R_load + jωL_load)` for S11 with sharp Q peaks

Files: `server/src/nanovna.rs`, `server/src/main.rs`

# High Priority

(No current items)

# Medium Priority

### Circuit Simulation (Beyond SVG)
Current `circuit.rs` generates SVG diagrams only; no SPICE-like simulation.
- Could add: AC analysis, impedance at frequency, power transfer calculation
- Lower priority: NanoVNA covers frequency sweep use case for matching networks

Files: `server/src/circuit.rs`

# Low Priority

# ===

## Recently Fixed (2026-01-22)

### NanoVNA Frequency-Dependent Effects
Added realistic RF behavior to coil impedance model.
- Skin effect resistance: `R_ac = R_dc·√(1 + (ω/ω_skin)²)`
- Parasitic capacitance via Medhurst approximation
- Self-resonant frequency calculation
- File: `server/src/nanovna.rs`

### Non-Z-Aligned Coil Support
Field computation now supports arbitrarily oriented coils.
- Biot-Savart integration works for any coil axis orientation
- File: `server/src/field.rs`

### Group Bounds Recalculation on Remove
Fixed bounds not updating when children removed from groups.
- `remove()` now triggers bounds recalculation
- File: `stdlib/groups.lua`

### Probe Volume Statistics
Implemented statistics computation for line probes when `statistics` parameter is specified.
- `server/src/field.rs`: Added `ProbeStatistics` struct with min/max/mean/std fields
- `server/src/field.rs`: Added `statistics: Option<ProbeStatistics>` field to `LineMeasurement`
- `server/src/field.rs`: Updated `to_binary()` to serialize statistics (1-byte flag + 4x f32 if present)
- `server/src/main.rs`: `try_compute_probe_measurements()` now checks for `statistics` config and computes values

### Mesh Validation
Added mesh validation functions to detect common geometry issues.
- Checks for NaN/Inf in positions and normals
- Validates indices are within bounds
- Detects degenerate triangles (zero area)
- Warns about near-zero or extremely large mesh extents
- Added 4 unit tests for validation
- File: `server/src/geometry.rs`

### Export Placeholder Normals
Fixed test cube data in export.rs to have computed corner normals.
- Replaced `vec![0.0; 24]` with proper corner normals (averaged from adjacent faces)
- File: `server/src/export.rs`

### Component/Instance Backend Support
Full assembly/component/instance hierarchy now working end-to-end.
- `stdlib/groups.lua:serialize()` correctly returns `type="assembly"` and `type="component"`
- `geometry.rs:build_manifold_object()` handles "assembly", "component", and "instance" types
- `build_mesh_recursive()` also updated to handle all three types
- Instance resolution via component lookup implemented
- Files: `stdlib/groups.lua`, `server/src/geometry.rs`

### Acoustic 3D/1D Visualization
Acoustic field now has feature parity with magnetic field visualization.
- Added 3D arrow field: 10×10×10 grid with pressure gradient vectors
- Added 1D line profile: 101 points along Z axis at r=0
- File: `server/src/acoustic.rs`

### Transforms.lua Field Incompatibility
Transform functions now compatible with primitive shape creation.
- `translate()`, `rotate()`, `scale()` now use `._ops` pattern instead of `._transform`
- Properly chains operations with primitives created via `primitives.lua`
- Files: `stdlib/transforms.lua`

### Non-uniform Scale Normals
Fixed incorrect normal transformation for non-uniform scaling.
- `geometry.rs:apply_mesh_transforms()` now uses inverse scale factors for normals
- Normals properly re-normalized after transformation
- File: `server/src/geometry.rs`

# ===

## Recently Fixed (2026-01-21)

### Torus Primitive Rewrite
Replaced `Polygons.revolve()` with parametric mesh generation.
- Uses parametric equations: x = (R + r·cos(v))·cos(u), y = (R + r·cos(v))·sin(u), z = r·sin(v)
- Generates vertices and normals for u_segments × v_segments grid
- Creates Manifold via FFI to MeshGL
- File: `server/src/geometry.rs:226-282`

### Circuit → NanoVNA Impedance Integration
MatchingNetwork can now auto-populate impedance from NanoVNA computation.
- Added `nanovna::compute_impedance_at_frequency()` function
- MatchingNetwork accepts `use_nanovna: true` config option
- Falls back to static impedance values if NanoVNA config not found
- Files: `server/src/nanovna.rs`, `server/src/main.rs`

### Probe Backend Computation
Implemented line probe B-field sampling backend.
- Added `LineMeasurement` struct with binary serialization (LNPROBE header)
- `try_compute_probe_measurements()` samples B-field along line
- Uses same Biot-Savart computation as GaussMeter
- Files: `server/src/field.rs`, `server/src/main.rs`

### Window Z-Ordering
Implemented click-to-focus z-ordering for draggable windows.
- Added base z-index: 50 to .tui-window class
- Tracks topZIndex counter, increments on window click
- Files: `renderer/index.html`

### Magnetic Field Pattern Matching
Backend now recognizes `Coil` global (project convention) in addition to `config` global.
- `main.rs:try_compute_helmholtz_field` reads from `Coil.mean_radius`, `Coil.gap`, etc.
- Also reads `Wire` global for wire diameter and packing factor

### NanoVNA Renderer Support
Added NanoVNA S11 display to renderer:
- `renderer/index.html`: Added nanovna-window with canvas
- `renderer/src/main.ts`: Added parse_nanovna_data() and draw_nanovna()
- `server/src/main.rs`: Added current_nanovna state caching for new WebSocket clients

### Ring Primitive Fix
Fixed `ring` primitive failing with `InvalidConstruction` error.
- Reimplemented ring as difference of two cylinders in `geometry.rs`
- Ring now created correctly via `outer_cylinder.difference(&inner_cylinder)`

### Probe Line Parsing and Renderer Support
Fixed Probe line measurement to use Lua array format.
- Changed `line_table.get("start")` to `line_table.get(1)` in main.rs
- Added renderer support for MEASURE and LNPROBE WebSocket messages

# ===

## Completed (Reference)

### Export
- STL binary export (5 tests)
- 3MF with per-vertex colors (5 tests)

### Geometry
- box, cylinder, sphere, torus, ring primitives
- CSG: union, difference, intersect via manifold3d
- group containers with recursive children
- Transform chain: at, rotate, scale, centered
- assembly/component/instance hierarchy with backend support
- Mesh validation (4 tests)

### Physics
- Helmholtz magnetic field (Biot-Savart, 7 tests)
- Acoustic pressure field (Rayleigh-Sommerfeld, 7 tests)
- Standing wave reflection modeling (mirror source)
- NanoVNA S11 frequency sweep (7 tests) - Wheeler approximation only (see Very High Priority)

### Instruments
- GaussMeter backend computation for point B-field measurement
- Hydrophone backend computation for point pressure measurement
- MagneticFieldPlane with XY/YZ/XZ plane support + 3D arrows + 1D line
- AcousticPressurePlane with XY/YZ/XZ plane support + 3D arrows + 1D line
- Probe line measurement for B-field sampling along arbitrary lines

### Materials
- Comprehensive acoustic properties database (12 materials)
- Copper, air, water, borosilicate glass, PZT, polycarbonate, PLA, PTFE, aluminum, neodymium
- Speed of sound, impedance, attenuation coefficients

### Visualization
- XZ/XY/YZ colormap planes with jet/viridis/plasma colormaps
- 3D arrow field for magnetic and acoustic vectors
- 1D line plot (canvas graph) for magnetic and acoustic fields
- Circuit diagram SVG (15 tests)
- NanoVNA S11 frequency response graph

### Renderer
- Three.js mesh rendering with X-ray Fresnel material
- Flat shading with dFdx/dFdy normals
- WebSocket binary protocol: VIEW, FIELD, CIRCUIT, MEASURE, LNPROBE, NANOVNA headers
- Draggable TUI windows with z-ordering
- Retro scanline/CRT effects
