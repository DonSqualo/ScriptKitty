# Implementation Plan

ScriptKitty v0.0.4 - Prioritized implementation backlog (2026-01-21).

## High Priority

### Signal Generator → Amplifier → Coupling Coil Circuit
Connect circuit diagram components to coupling coil geometry.
- Circuit components exist: SignalGenerator, Amplifier, MatchingNetwork, TransducerLoad (`server/src/circuit.rs`)
- Need: Wire coupling coil impedance to MatchingNetwork calculation
- Location: `stdlib/circuits.lua`, `server/src/circuit.rs`

## Medium Priority

### Draggable Windows for Multiphysics
Add movable UI windows for field plane controls.
- Basic window framework exists in renderer
- Need: z-ordering

### Circuit Simulation (Beyond Visualization)
Current `circuit.rs` generates SVG diagrams only; no SPICE-like simulation.
- Could add: AC analysis, impedance at frequency, power transfer calculation
- Lower priority than NanoVNA which covers frequency sweep use case

## Low Priority

### Mesh Validation
- Non-manifold geometry detection
- Zero-volume solid warnings

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
