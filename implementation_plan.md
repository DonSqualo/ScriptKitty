# Implementation Plan

Mittens v0.0.15 - Prioritized implementation backlog (2026-01-28).

# Critical Priority

### FDTD Electromagnetic Simulation (Server-Side)

**Goal:** Compute resonant frequency of bridged loop-gap resonator via full-wave FDTD simulation, entirely in Rust.

**Spec:** `specs/meep/SPEC.md`

**Key requirements:**
- No Python — pure Rust FDTD implementation (no MEEP dependency)
- Pipeline: Lua → Manifold3D mesh → voxelization → FDTD simulation → WebSocket results
- Outputs: S11 (replaces Wheeler approximation), 2D B-field plane, oscilloscope widget

**Phases:**
1. [x] Core FDTD solver (`server/src/fdtd.rs`) - 4 tests
   - 3D Yee grid with staggered E/H fields
   - Leapfrog time-stepping with CFL-stable dt
   - Material properties (permittivity, permeability, conductivity)
   - Gaussian pulse and CW sources
   - Field monitors with time-series recording
   - Resonance detection via FFT
   - S11 computation from incident/reflected waves
2. [x] Voxelization (`server/src/voxel.rs`) - 1 test
   - Ray-casting point-in-mesh test
   - Mesh to voxel grid conversion
3. [ ] PML absorbing boundaries (required for open-domain simulations)
4. [ ] Geometry integration (voxel grid → FDTD material assignment)
5. [ ] Lua API: `FdtdStudy({ geometry = {...}, freq_center = 450e6, ... })`
6. [ ] WebSocket protocol for FDTD results (FDTD\0 header)
7. [ ] Renderer: oscilloscope widget for time-domain field display
8. [ ] Integration test with loop-gap resonator

**Success criteria:** Screenshot shows full Helmholtz system + NanoVNA with FDTD-computed S11 + 2D B-field + oscilloscope at 16 ± 0.5 mT.

**Note:** Original spec called for MEEP FFI bindings, but MEEP is a complex C++ library with no Rust bindings available. Pure Rust FDTD achieves the same goal with zero external dependencies and full integration.

---

# High Priority

(None currently)

---

# Medium Priority

### Geometry-Aware Inductance
Extract coil coordinates from mesh, compute L from field energy `L = 2W_m/I²` instead of Wheeler formula.

**Files:** `server/src/nanovna.rs`

---

# Low Priority

### Sample Environment Coupling
Model biological tissue effects (ε_r ≈ 80) on resonator Q.

---

# Recently Fixed (2026-01-28)

### Pure Rust FDTD Solver
Added complete FDTD electromagnetic solver in pure Rust.
- File: `server/src/fdtd.rs`
- 3D Yee algorithm with leapfrog time-stepping
- CFL-stable automatic time step calculation
- Material properties: eps_r, mu_r, sigma_e, sigma_m
- Sources: Gaussian pulse (broadband), continuous wave
- Monitors: field probes with time-series recording
- Resonance detection via FFT peak finding
- S11 computation from frequency-domain division
- 2D field slice extraction for visualization
- 4 tests passing

### Voxel Point-in-Mesh Fix
Fixed edge case in ray-casting point-in-mesh test where points on triangle edges were double-counted.
- File: `server/src/voxel.rs`

### Whisper Audio Transcription
Added chunked whisper transcription for voice notes.
- Script: `~/.local/bin/whisper-chunked`
- Handles long audio via ffmpeg segmentation

---

# Recently Fixed (2026-01-22)

### NanoVNA Coupled Resonator
Added mutual inductance and coupled impedance for Loop Gap Resonator modeling.
- Neumann formula for coaxial loop mutual inductance
- Coupled impedance calculation with reflected impedance
- Sharp Q peaks visible in S11 sweep when resonator coupled
- File: `server/src/nanovna.rs`

### Circuit AC Analysis
Added impedance chain analysis beyond SVG diagram generation.
- `CircuitAnalysis` struct with S11, power transfer, voltage gain
- `analyze_circuit_ac()` function computes circuit response at frequency
- L-network impedance transformation for matching networks
- File: `server/src/circuit.rs`

### NanoVNA Frequency-Dependent Effects
Added realistic RF behavior to coil impedance model.
- Skin effect resistance: `R_ac = R_dc·√(1 + (ω/ω_skin)²)`
- Parasitic capacitance via Medhurst approximation
- Self-resonant frequency calculation
- File: `server/src/nanovna.rs`

---

# Completed (Reference)

### Export
- STL binary export (5 tests)
- 3MF with per-vertex colors (5 tests)

### Geometry
- box, cylinder, sphere, torus, ring primitives
- CSG: union, difference, intersect via manifold3d
- group containers with recursive children
- Transform chain: at, rotate, scale, centered
- assembly/component/instance hierarchy with backend support
- Mesh validation (7 tests)

### Physics
- Helmholtz magnetic field (Biot-Savart, 10 tests)
- Acoustic pressure field (Rayleigh-Sommerfeld, 7 tests)
- Standing wave reflection modeling (mirror source)
- NanoVNA S11 frequency sweep (12 tests) - Wheeler + skin effect + parasitic capacitance + mutual inductance
- FDTD electromagnetic solver (4 tests)

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
- Circuit diagram SVG + AC analysis (21 tests)
- NanoVNA S11 frequency response graph

### Renderer
- Three.js mesh rendering with X-ray Fresnel material
- Flat shading with dFdx/dFdy normals
- WebSocket binary protocol: VIEW, FIELD, CIRCUIT, MEASURE, LNPROBE, NANOVNA headers
- Draggable TUI windows with z-ordering
- Retro scanline/CRT effects
