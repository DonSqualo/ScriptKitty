# Manifold Alternatives for Rust + Multiphysics Architecture

> Source: ChatGPT conversation (2025)
> Context: ScriptKitten/Mittens CAD pipeline for ultrasound + EMF simulation + 3D printing

## TL;DR

**Key insight:** Separate "authoring geometry" from "analysis geometry."
- Authoring: CSG/B-Rep booleans, previews, STL/3MF output
- Analysis: tetrahedral mesh + material regions + boundary tags for PDE solvers

**Recommendation:** Use Gmsh as the central adapter between geometry and simulation.

---

## The Problem

For multiphysics (ultrasound, EMF), the hard part is getting:
- Volumetric mesh with correct material labels
- Good element quality
- Not just watertight surface

If pipeline only produces triangle meshes → war to recover volumes/regions/boundaries for simulation.

---

## Options in Rust-land

### Option A: Gmsh as Bridge (Pragmatic)

Use any solid modeling method, then hand to Gmsh for:
- Tetrahedral volume mesh
- Physical groups (regions/boundaries) for materials and BCs

**Rust bindings:**
- `gmsh_sys` (low-level)
- `rgmsh` (higher-level)

**Advantage:** Escape hatch for robust meshing/tagging without writing a mesher.

### Option B: Real CAD Kernel for Booleans + Topology

For robust region identity and topology (faces/edges) mapping to boundary conditions:

| Crate | Notes |
|-------|-------|
| OpenCascade bindings | Heavy but very CAD |
| Truck | Pure Rust, still maturing |

Preserves "this face is boundary Γ" semantics better than mesh-only CSG.

### Option C: Simulation-First (SDF/Voxels)

For ultrasound (naturally grid-based / finite differences):
- Use SDFs / voxel grids for geometry and material maps
- Run grid-based solvers
- Extract printable surface at end

**Surface extraction:** `fast-surface-nets` for isosurface mesh extraction from sampled field.
- Extremely robust
- Resolution-limited

---

## Rust Solver Libraries

### Electromagnetics
- `rems`: Rust FDTD (time-domain Maxwell) simulator
- Aligned with grid/SDF lane if FDTD-friendly

### General FEM
- `fenris`: Finite element library (explicitly unstable/not production-ready)
- `gemlab`: Geometry/meshing utilities for FE analyses

### Reality Check
For serious ultrasound (elastic/acoustic wave PDEs), may need to couple Rust to established solver ecosystem. Keep geometry + meshing + IO in Rust.

---

## Recommended Architecture for ScriptKitten

```
Lua Script
    ↓
CSG Scene Graph (authoring)
    ↓
┌─────────────────────────────────────┐
│                                     │
▼                                     ▼
Manifold (FFI) or Truck          Gmsh (rgmsh)
    ↓                                 ↓
Triangle mesh → 3MF              Volume mesh + tags
(3D printing)                    (simulation)
                                      ↓
                              ┌───────┴───────┐
                              ▼               ▼
                           rems           Wave solver
                          (FDTD)          (grid or FEM)
                         for EMF         for ultrasound
```

### Component Choices

| Component | Choice |
|-----------|--------|
| Authoring/scripting | Lua → CSG scene graph |
| Booleans for printing | Manifold (FFI) or Truck → triangle mesh → 3MF |
| Meshing for simulation | Gmsh (physical groups, region tags) |
| EMF solver | `rems` (FDTD, native Rust) |
| Ultrasound solver | Grid-based (voxel/material grids) OR FEM (high-quality tetra mesh) |

---

## Key Takeaway

**Gmsh as the central adapter** is least painful way to serve both simulation and 3D printing without betting on single "perfect" Rust manifold/boolean crate.
