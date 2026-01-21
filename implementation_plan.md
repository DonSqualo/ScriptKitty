# Implementation List

Current state of ScriptKitty (v0.0.2, 2026-01-21).

## Completed Features

### Export
- [x] **STL export** (`server/src/export.rs`) - Binary STL with computed normals
- [x] **3MF export** (`server/src/export.rs`) - ZIP archive with colors

### Primitives
- [x] **box** (`stdlib/primitives.lua`, `server/src/geometry.rs`) - Corner at origin
- [x] **cylinder** (`stdlib/primitives.lua`, `server/src/geometry.rs`) - Base on XY, extends +Z
- [x] **sphere** (`stdlib/primitives.lua`, `server/src/geometry.rs`) - Centered at origin
- [x] **torus** (`stdlib/primitives.lua`) - Centered at origin, hole along Z

### CSG
- [x] **union** (`stdlib/csg.lua`, `server/src/geometry.rs`)
- [x] **difference** (`stdlib/csg.lua`, `server/src/geometry.rs`)
- [x] **intersect** (`stdlib/csg.lua`, `server/src/geometry.rs`)

### Physics
- [x] **Magnetostatic (Helmholtz)** (`server/src/field.rs`) - Biot-Savart for coil pairs with XZ/XY/YZ planes
- [x] **Acoustic (Rayleigh-Sommerfeld)** (`server/src/acoustic.rs`) - Pressure field with coverslip reflection

### Circuits
- [x] **SignalGenerator** (`stdlib/circuits.lua`, `server/src/circuit.rs`) - RF source with frequency
- [x] **Amplifier** (`stdlib/circuits.lua`, `server/src/circuit.rs`) - Power amplifier with gain
- [x] **MatchingNetwork** (`stdlib/circuits.lua`, `server/src/circuit.rs`) - L-network from impedance
- [x] **TransducerLoad** (`stdlib/circuits.lua`, `server/src/circuit.rs`) - Piezo load with ground
- [x] **Circuit** (`stdlib/circuits.lua`, `server/src/circuit.rs`) - SVG schematic generation

### Instruments
- [x] **MagneticFieldPlane** (`stdlib/instruments/init.lua`) - XZ/XY/YZ colormap and 3D arrows
- [x] **AcousticPressurePlane** (`stdlib/instruments/init.lua`) - Pressure field visualization
- [x] **Hydrophone** (`stdlib/instruments/init.lua`) - Acoustic point probe
- [x] **GaussMeter** (`stdlib/instruments/init.lua`) - Magnetic point probe
- [x] **Probe** (`stdlib/instruments/init.lua`) - Line/volume probe definition

### Renderer
- [x] **Mesh rendering** - X-ray Fresnel shader
- [x] **Flat shading** - dFdx/dFdy normals
- [x] **Colormap plane** - XZ/XY/YZ planes with jet/viridis/plasma colormaps
- [x] **Arrow field** - 3D vector visualization
- [x] **1D graph** - Bz vs Z plot

### Tests (36 total)
- [x] **export.rs** - STL/3MF export, cross product, normalize
- [x] **field.rs** - Biot-Savart, Helmholtz uniformity, colormap, binary format
- [x] **acoustic.rs** - Rayleigh integral, impedance, reflection, field generation
- [x] **circuit.rs** - Component drawing, SVG structure, L/C formulas, wire routing

## Deleted (specs preserved)

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

### Helmholtz Field
- Pattern-matches `config` global table with specific keys
- Requires: coil_mean_radius, gap
- Optionals: wire_diameter, windings, layers, packing_factor, current

## File Structure

```
stdlib/
├── init.lua          - Main entry, global exports
├── primitives.lua    - box(), cylinder(), sphere(), torus()
├── csg.lua           - union(), difference(), intersect()
├── groups.lua        - group(), assembly(), component()
├── transforms.lua    - translate(), rotate(), scale(), patterns
├── materials.lua     - material() with copper, air database
├── physics.lua       - magnetostatic(), acoustic(), current_source()
├── circuits.lua      - SignalGenerator(), Amplifier(), MatchingNetwork(), TransducerLoad(), Circuit()
├── instruments/
│   └── init.lua      - Probe(), GaussMeter(), MagneticFieldPlane(), AcousticPressurePlane(), Hydrophone()
├── view.lua          - view(), camera config
└── export.lua        - export_stl(), export_3mf()

server/src/
├── main.rs           - Axum server, file watcher, Lua runner
├── geometry.rs       - Manifold CSG, mesh generation
├── export.rs         - STL/3MF writers
├── field.rs          - Helmholtz Biot-Savart
├── acoustic.rs       - Rayleigh-Sommerfeld pressure field
└── circuit.rs        - SVG circuit diagram generation

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
