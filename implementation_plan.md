# Implementation Plan

Mittens v0.0.14 - Prioritized implementation backlog (2026-01-28).

# Critical Priority

### MEEP Integration (Server-Side)

**Goal:** Compute resonant frequency of bridged loop-gap resonator via full-wave FDTD simulation, entirely in Rust.

**Spec:** `specs/meep/SPEC.md`

**Key requirements:**
- No Python — all MEEP via Rust FFI bindings
- Pipeline: Lua → Manifold3D mesh → MEEP simulation → WebSocket results
- Outputs: S11 (replaces Wheeler approximation), 2D B-field plane, oscilloscope widget

**Phases:**
1. [ ] MEEP FFI binding (`crates/meep-sys/`)
2. [ ] Geometry integration (mesh → MEEP)
3. [ ] Resonance detection (harminv)
4. [ ] Field visualization (colormap + oscilloscope)
5. [ ] Screenshot test validation

**Success criteria:** Screenshot shows full Helmholtz system + NanoVNA with MEEP-computed S11 + 2D B-field + oscilloscope at 16 ± 0.5 mT.

**Obsoletes:** `examples/meep/` (Python scripts) — delete after Rust implementation complete.

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
