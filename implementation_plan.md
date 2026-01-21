# Implementation List

Current state of ScriptKitty after cleanup (2026-01-21).

## Completed Features

### Export
- [x] **STL export** (`server/src/export.rs`) - Binary STL with computed normals
- [x] **3MF export** (`server/src/export.rs`) - ZIP archive with colors

### CSG
- [x] **union** (`stdlib/csg.lua`, `server/src/geometry.rs`)
- [x] **difference** (`stdlib/csg.lua`, `server/src/geometry.rs`)
- [x] **intersect** (`stdlib/csg.lua`, `server/src/geometry.rs`)

### Physics
- [x] **Magnetostatic (Helmholtz)** (`server/src/field.rs`) - Biot-Savart for coil pairs

### Instruments
- [x] **MagneticFieldPlane** - XZ colormap and 3D arrows
- [x] **GaussMeter** - Point probe definition
- [x] **Probe** - Line/volume probe definition

### Renderer
- [x] **Mesh rendering** - X-ray Fresnel shader
- [x] **Flat shading** - dFdx/dFdy normals
- [x] **Colormap plane** - XZ plane, jet colormap
- [x] **Arrow field** - 3D vector visualization
- [x] **1D graph** - Bz vs Z plot

## Deleted (specs preserved)

### Acoustic (`specs/stdlib/acoustic.md`)
- Rayleigh-Sommerfeld pressure field computation
- AcousticPressurePlane, AcousticEnergyPlane, AcousticIntensityPlane
- Hydrophone probe

### Circuits (`specs/stdlib/circuits.md`)
- SVG circuit diagram generation
- SignalGenerator, Amplifier, MatchingNetwork, TransducerLoad
- L-match calculation

### Physics (no specs - API was empty)
- electromagnetic()
- electrostatic()
- thermal() / thermal_transient()
- structural()
- multiphysics()

### Instruments (no specs - API was empty)
- Oscilloscope
- Thermometer
- ForceSensor
- Streamlines
- Isosurface
- SParams
- ElectricFieldPlane
- TemperaturePlane

### Materials (partial)
- Removed: fr4, steel, water, pzt, petg, rubber, glass, polycarbonate
- Kept: copper, air

### Export (no specs - never implemented)
- export_step
- export_gltf
- export_obj

## Known Limitations

### Renderer
- Only XZ plane supported (XY/YZ ignored)
- Only jet colormap (viridis/plasma ignored)

### Helmholtz Field
- Pattern-matches `config` global table with specific keys
- Requires: coil_mean_radius, gap
- Optionals: wire_diameter, windings, layers, packing_factor, current

## File Structure

```
stdlib/
├── init.lua          - Main entry, global exports
├── primitives.lua    - box(), cylinder()
├── csg.lua           - union(), difference(), intersect()
├── groups.lua        - group(), assembly(), component()
├── transforms.lua    - translate(), rotate(), scale(), patterns
├── materials.lua     - material() with copper, air database
├── physics.lua       - magnetostatic(), acoustic(), current_source()
├── instruments/
│   └── init.lua      - Probe(), GaussMeter(), MagneticFieldPlane()
├── view.lua          - view(), camera config
└── export.lua        - export_stl(), export_3mf()

server/src/
├── main.rs           - Axum server, file watcher, Lua runner
├── geometry.rs       - Manifold CSG, mesh generation
├── export.rs         - STL/3MF writers
└── field.rs          - Helmholtz Biot-Savart

renderer/src/
└── main.ts           - Three.js scene, WebSocket client

specs/
├── overview.md       - Architecture philosophy
├── server/
│   ├── architecture.md   - System diagram
│   ├── implementation_status.md - API vs backend matrix
│   └── garbage_collection.md - Post-project review
├── stdlib/
│   ├── primitives.md     - box(), cylinder() docs
│   ├── csg.md            - Boolean ops docs
│   ├── groups.md         - Hierarchy docs
│   ├── export.md         - File output docs
│   ├── view.md           - Camera/render docs
│   ├── gotchas.md        - Known pitfalls
│   ├── acoustic.md       - Deleted acoustic implementation
│   └── circuits.md       - Deleted circuit implementation
└── renderer/
    └── colormap_plane.md - Field visualization docs
```
