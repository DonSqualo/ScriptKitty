# Implementation Plan

ScriptKitty v0.0.12 - Prioritized implementation backlog (2026-01-22).

# Very High Priority

### NanoVNA Full 3D EMF Simulation
Current `nanovna.rs` uses Wheeler formula approximation (lumped parameter model) with no connection to actual geometry. It produces plausible-looking S11 curves but no resonance peaks from actual Loop Gap Resonator coupling.

**Current Implementation (What Exists):**
- Wheeler formula + Nagaoka correction for inductance: `L = μ₀·N²·A·K / length`
- Pure series RLC impedance: `Z = R + jωL` (DC resistance only)
- S11 conversion: `(Z - Z₀)/(Z + Z₀)` where Z₀ = 50Ω

**Hardcoded Approximations (What's Wrong):**
- Assumes uniform wire diameter/spacing (no actual geometry)
- Frequency-independent resistance (ignores skin effect >1 MHz)
- No parasitic capacitance between turns
- No mutual inductance M between drive coil and resonator gap
- No self-resonance frequency (SRF) calculation
- No sample environment coupling (biological tissue ε_r ≈ 80)

**Missing Connection:** The Biot-Savart computation for magnetic field (working in `field.rs`) could be repurposed to derive L and M from actual geometry, but currently NanoVNA reads from separate `NanoVNA` Lua global with hardcoded parameters.

**Implementation Roadmap:**
1. **Geometry-Aware Inductance**: Extract coil coordinates, compute self-inductance from field energy `L = 2W_m/I²`
2. **Mutual Inductance**: Compute M between drive coil and resonator gap via B-field integration
3. **Frequency Effects**: Add skin effect `R_ac = R_dc·√(1 + (ω/ω_skin)²)`, parasitic capacitance
4. **Coupled Impedance**: `Z_in = jωL_drive + (ωM)²/(R_load + jωL_load)` for S11 with sharp Q peaks
5. **Validation**: Compare simulation against real NanoVNA measurements

Files: `server/src/nanovna.rs`, `server/src/main.rs`, `server/src/field.rs`

# High Priority

(No current items)

# Medium Priority

### Circuit Simulation (Beyond SVG)
Current `circuit.rs` generates SVG diagrams only; no SPICE-like simulation.
- Could add: AC analysis, impedance at frequency, power transfer calculation
- Lower priority: NanoVNA covers frequency sweep use case for matching networks

Files: `server/src/circuit.rs`

# Low Priority

### Probe Volume Statistics
`Instruments/init.lua:71` has `statistics` config parameter but unclear what's computed.
- No documentation of min/max/mean/std statistics for volume probes
- Volume probe API exists but backend behavior undocumented

# ===

## Recently Fixed (2026-01-22)

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
