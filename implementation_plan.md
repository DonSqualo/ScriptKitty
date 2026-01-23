# Implementation Plan

ScriptKitty v0.0.15 - Electron-MRI Project (2026-01-23)

## Task Tracker

| # | Task | Status | Depends On |
|---|------|--------|------------|
| 1 | SLMG resonator geometry (16 wedge segments) | completed | — |
| 2 | Double coupling loop geometry | completed | 1 |
| 3 | Multi-gap resonator RF physics (NanoVNA) | completed | 1 |
| 4 | B1 field homogeneity visualization | pending | 1 |
| 5 | 19-tube phantom geometry | completed | — |
| 6 | Loaded Q computation | pending | 3, 5 |
| 7 | Modulation coils | pending | 1 |
| 8 | Housing and shield | pending | 1 |
| 9 | EPR image simulation (final deliverable) | pending | 4, 5, 6 |

**Next**: Tasks 4 and 6 are unblocked.

## Project Goal

Replicate the Petryakov et al. 2007 "Single loop multi-gap resonator for whole body EPR imaging" device in simulation. Final deliverable: B1 field homogeneity visualization of a phantom inside the resonator, matching Figure 4 from the paper.

**Paper Reference**: Petryakov et al., J. Magn. Reson. 188 (2007) 68-73

## Very High Priority

### 1. SLMG Resonator Geometry
Parametric 16-gap loop-gap resonator matching paper dimensions.

**Dimensions from paper:**
- Inner diameter: 42mm
- Outer diameter: 88mm
- Length: 48mm
- Number of gaps: 16 (parametric, min 8 required for 1.2 GHz)
- Gap thickness: 1.68mm polystyrene plates
- Segment material: Rexolite (silver-plated inner face + 13mm on sides)

**Geometry breakdown:**
- 16 wedge-shaped segments arranged radially
- Each segment is a trapezoidal prism
- Gaps between segments filled with polystyrene dielectric
- PVC reinforcing cylinder (inner shell, diameter 42mm)
- Conductive outer shield

**Lua API needed:**
```lua
Resonator = {
  inner_diameter = 42,
  outer_diameter = 88,
  length = 48,
  num_gaps = 16,
  gap_thickness = 1.68,
}
```

Files: `project/Electron-MRI.lua`, `server/src/geometry.rs`

### 2. Coupling Loop Geometry
Double coupling loop with laterally displaced λ/4 feeding lines.

**From paper:**
- Parallel double loop design (20mm diameter each for L-band)
- λ/4 feeding lines (~63mm at 1.2 GHz)
- Attached to polystyrene spacer ring
- Coupling capacitor in series

Files: `project/Electron-MRI.lua`

### 3. Multi-Gap Resonator RF Physics
Extend NanoVNA simulation for multi-gap loop-gap resonator.
Simulate physics using complete 3D + time full EMF wave calculations, do not rely on given formulas, instead use them to write tests.

**Physics from paper:**
- ω = 1/√(L_sum × C_sum)
- L_sum = L × N (N gaps increases frequency by √N)
- C_sum = C/N where C = ε₀ × S_c / d
- Q = iωL/R (Q drops with increased gaps due to lower inductance)

**Parameters to compute:**
- Resonant frequency from geometry
- Gap capacitance from plate area and dielectric
- Total inductance from loop geometry
- Q factor (empty and loaded)

Files: `server/src/nanovna.rs`

### 4. B1 Field Homogeneity Visualization
Show RF magnetic field distribution inside resonator volume.

**From paper (Figure 4):**
- XY and XZ slices through resonator center
- Uniform intensity confirms good B1 homogeneity
- 10mm scale bar

**Implementation:**
- Compute B1 field at grid points inside resonator
- Use MagneticFieldPlane with XY and XZ views
- Normalize intensity for homogeneity assessment

Files: `server/src/field.rs`, `project/Electron-MRI.lua`

## High Priority

### 5. Sample Phantom
19-tube phantom for field homogeneity testing (Figure 4 from paper).

**From paper:**
- 19 polystyrene tubes, 4mm diameter
- Arranged in circular pattern
- Filled to 11mm height with 1mM TAM solution
- Total volume ~11cc

Files: `project/Electron-MRI.lua`

### 6. Loaded Q Computation
Compute Q factor with lossy sample inside resonator.

**From paper Table 1:**
- Empty: f0 = 1.22 GHz
- With 11cc saline: f0 = 1.216 GHz, Q = 72
- With 20cc saline: Q = 57

Files: `server/src/nanovna.rs`

## Medium Priority

### 7. Modulation Coils
Form-wound coils in cylinder slots for field modulation.

Files: `project/Electron-MRI.lua`

### 8. Housing and Shield
PVC case with parallel slots for sample access, silver-plated lids.

Files: `project/Electron-MRI.lua`

## Low Priority

### 9. EPR Image Simulation
Simulated EPR image of phantom/sample (final deliverable).

## ===

## Completed (Reference)

### Electron-MRI Geometry (v0.0.14)
- SLMG resonator: 16 wedge segments arranged radially
- Wedge primitive (stdlib + geometry.rs) for radial resonator segments
- Double coupling loop geometry with λ/4 feeding lines
- 19-tube phantom geometry for field homogeneity testing
- Multi-gap resonator RF physics (NanoVNA extension with gap capacitance, resonant frequency, Q factor)

### NanoVNA Coupled Resonator (v0.0.13)
- Wheeler formula + Nagaoka correction for inductance
- Skin effect resistance
- Parasitic capacitance via Medhurst approximation
- Mutual inductance M via Neumann formula
- Coupled impedance calculation
- S11 conversion with frequency-dependent impedance

### Circuit AC Analysis (v0.0.13)
- `CircuitAnalysis` struct with S11, power transfer, voltage gain
- `analyze_circuit_ac()` function
- L-network impedance transformation

### Mesh Validation (v0.0.13)
- Degenerate triangle removal
- NaN/Inf validation
- Index bounds checking

### Physics Simulations (v0.0.12)
- Helmholtz magnetic field (Biot-Savart, 10 tests)
- Acoustic pressure field (Rayleigh-Sommerfeld, 7 tests)
- NanoVNA S11 frequency sweep (12 tests)

### Instruments (v0.0.12)
- GaussMeter, Hydrophone, Probe line
- MagneticFieldPlane, AcousticPressurePlane
- 3D arrows, 1D line profiles
