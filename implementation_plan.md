# Implementation Plan

ScriptKitty v0.0.5 - Prioritized implementation backlog (2026-01-21).

## Medium Priority

### Circuit Simulation (Beyond SVG)
Current `circuit.rs` generates SVG diagrams only; no SPICE-like simulation.
- Could add: AC analysis, impedance at frequency, power transfer calculation
- Lower priority - NanoVNA covers frequency sweep use case

### Component/Instance Backend Support
Lua stdlib has assembly(), component(), and instance() but backend doesn't distinguish these types.
- assembly and component serialize as type="group" (not their actual types)
- Backend has no code path for type="instance"
- Instance expansion and component reuse optimization not implemented
- Files: `stdlib/groups.lua`, `server/src/geometry.rs`
- Impact: Instancing feature non-functional, all instances get full geometry copies

## Low Priority

### Mesh Validation
- Non-manifold geometry detection
- Zero-volume solid warnings
- Files: `server/src/geometry.rs`

### View State Serialization
Camera, visibility, and clipping NOT sent to renderer (per `specs/stdlib/view.md:86-89`).
- Only `flat_shading` and `circular_segments` are serialized
- If needed: extend VIEW WebSocket message to include camera position/target

## Recently Fixed

### Torus Primitive Rewrite (2026-01-21)
Replaced `Polygons.revolve()` with parametric mesh generation.
- Uses parametric equations: x = (R + r·cos(v))·cos(u), y = (R + r·cos(v))·sin(u), z = r·sin(v)
- Generates vertices and normals for u_segments × v_segments grid
- Creates Manifold via FFI to MeshGL
- File: `server/src/geometry.rs:226-282`

### Circuit → NanoVNA Impedance Integration (2026-01-21)
MatchingNetwork can now auto-populate impedance from NanoVNA computation.
- Added `nanovna::compute_impedance_at_frequency()` function
- MatchingNetwork accepts `use_nanovna: true` config option
- Falls back to static impedance values if NanoVNA config not found
- Files: `server/src/nanovna.rs`, `server/src/main.rs`

### Probe Backend Computation (2026-01-21)
Implemented line probe B-field sampling backend.
- Added `LineMeasurement` struct with binary serialization (LNPROBE header)
- `try_compute_probe_measurements()` samples B-field along line
- Uses same Biot-Savart computation as GaussMeter
- Files: `server/src/field.rs`, `server/src/main.rs`

### Window Z-Ordering (2026-01-21)
Implemented click-to-focus z-ordering for draggable windows.
- Added base z-index: 50 to .tui-window class
- Tracks topZIndex counter, increments on window click
- Files: `renderer/index.html`

### Magnetic Field Pattern Matching (2026-01-21)
Backend now recognizes `Coil` global (project convention) in addition to `config` global.
- `main.rs:try_compute_helmholtz_field` reads from `Coil.mean_radius`, `Coil.gap`, etc.
- `main.rs:try_compute_gaussmeter_measurements` updated similarly
- Also reads `Wire` global for wire diameter and packing factor

### NanoVNA Renderer Support (2026-01-21)
Added NanoVNA S11 display to renderer:
- `renderer/index.html`: Added nanovna-window with canvas
- `renderer/src/main.ts`: Added parse_nanovna_data() and draw_nanovna()
- `server/src/main.rs`: Added current_nanovna state caching for new WebSocket clients

### Ring Primitive Fix (2026-01-21)
Fixed `ring` primitive failing with `InvalidConstruction` error.
- Problem: `Polygons.revolve()` was failing with the rectangle cross-section
- Solution: Reimplemented ring as difference of two cylinders in `geometry.rs`
- Ring now created correctly via `outer_cylinder.difference(&inner_cylinder)`

### Probe Line Parsing and Renderer Support (2026-01-21)
Fixed Probe line measurement to use Lua array format.
- Problem: Backend expected `line.start`/`line.stop` keys but Lua API uses array `line[1]`/`line[2]`
- Solution: Changed `line_table.get("start")` to `line_table.get(1)` in main.rs
- Added renderer support for MEASURE and LNPROBE WebSocket messages
- Files: `server/src/main.rs`, `renderer/src/main.ts`

## Completed (Reference)

### Export
- STL binary export (5 tests passing)
- 3MF with per-vertex colors (5 tests passing)

### Geometry
- box, cylinder, sphere, torus, ring primitives
- CSG: union, difference, intersect via manifold3d
- group, assembly, component with instances
- Transform chain: at, rotate, scale, centered

### Physics
- Helmholtz magnetic field (Biot-Savart, 7 tests)
- Acoustic pressure field (Rayleigh-Sommerfeld, 8 tests)
- Standing wave reflection modeling (mirror source)
- NanoVNA S11 frequency sweep (7 tests)

### Instruments
- GaussMeter backend computation for point B-field measurement
- Hydrophone backend computation for point pressure measurement
- MagneticFieldPlane with XY/YZ/XZ plane support
- AcousticPressurePlane with XY/YZ/XZ plane and colormap support
- Probe line measurement for B-field sampling along arbitrary lines

### Materials
- Comprehensive acoustic properties database
- PZT ceramic, polycarbonate, PTFE, aluminum, neodymium
- Speed of sound, impedance, attenuation coefficients

### Visualization
- XZ/XY/YZ colormap planes with jet/viridis/plasma colormaps
- 3D arrow field for magnetic vectors
- 1D line plot (canvas graph)
- Circuit diagram SVG (18 tests)

### Renderer
- Three.js mesh rendering
- Flat shading with dFdx/dFdy normals
- WebSocket binary protocol (VIEW, FIELD, CIRCUIT, MEASURE, NANOVNA headers)
