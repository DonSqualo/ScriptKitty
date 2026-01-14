# ScriptCAD

A script-first CAD system with multiphysics simulation capabilities. Think OpenSCAD meets COMSOL, designed for terminal-centric workflows.

## Features

- **Script-Based Modeling**: Define geometry using Lua scripts
- **Real-Time Preview**: WebGPU-accelerated SDF raymarching
- **Live Reload**: Automatic refresh on file save
- **Neovim Integration**: Designed for terminal workflows
- **Multiphysics Ready**: Framework for electromagnetic, thermal, and structural simulation
- **AI-Extensible**: Undefined primitives can trigger Claude Code to implement them

## Quick Start

### Prerequisites

- Rust 1.70+ (for server)
- Node.js 18+ (for renderer dev)
- Chrome 113+ or Edge 113+ (WebGPU support)

### Running

```bash
# Terminal 1: Start the renderer (dev mode)
cd renderer
npm install
npm run dev

# Terminal 2: Start the server
cd server
cargo run -- --watch ../examples --file ../examples/multiphysics/bridge_gap_resonator.lua
```

Open http://localhost:3000 in Chrome/Edge.

### Basic Example

```lua
-- my_model.lua
local ScriptCAD = require("stdlib")

-- Create a simple assembly
local base = box(50, 50, 10)
  :at(0, 0, 5)
  :material(material("steel"))

local cylinder = cylinder(10, 30)
  :at(0, 0, 25)
  :material(material("aluminum"))

local assembly = group("my_part", { base, cylinder })

-- Configure view
view({
  camera = "isometric",
  distance = 100,
})

-- Export
export_stl("my_part.stl", assembly)

return ScriptCAD.serialize()
```

## Project Structure

```
scriptcad/
├── stdlib/           # Lua standard library
│   ├── primitives.lua
│   ├── transforms.lua
│   ├── materials.lua
│   ├── csg.lua
│   ├── groups.lua
│   ├── physics.lua
│   ├── instruments/
│   ├── view.lua
│   └── export.lua
├── renderer/         # WebGPU frontend
│   ├── src/
│   └── shaders/
├── server/           # Rust backend
│   └── src/
├── examples/
│   ├── basic/
│   └── multiphysics/
└── docs/
```

## Lua API Overview

### Primitives

```lua
box(width, depth, height)
cube(size)
sphere(radius)
cylinder(radius, height)
cone(radius, height)
torus(major_radius, minor_radius)
capsule(radius, height)
helix({ inner_radius, outer_radius, turns, pitch, wire_diameter, style })
```

### Transforms

```lua
shape:at(x, y, z)           -- Position
shape:rotate(rx, ry, rz)    -- Rotation (degrees)
shape:scale(sx, sy, sz)     -- Scale

translate(shape, x, y, z)
rotate(shape, rx, ry, rz)
mirror(shape, "XZ")
linear_pattern(shape, count, dx, dy, dz)
circular_pattern(shape, count, radius, axis)
```

### CSG Operations

```lua
union(shape1, shape2, ...)
difference(base, cutter1, cutter2, ...)
intersect(shape1, shape2, ...)
smooth_union(blend_radius, shape1, shape2, ...)
shell(shape, thickness)
```

### Materials

```lua
material("copper")                    -- Database lookup
material("custom", { permittivity = 4.4 })  -- Custom properties
```

Available materials: `copper`, `aluminum`, `steel`, `stainless_steel`, `gold`, `fr4`, `ptfe`, `glass`, `ferrite`, `neodymium`, `pla`, `abs`, `air`, `vacuum`, `water`

### Physics Studies

```lua
electromagnetic({ type = "frequency_domain", frequencies = linspace(1e9, 10e9, 100) })
electrostatic({})
magnetostatic({ nonlinear = true })
thermal({ type = "steady_state" })
structural({ type = "static" })
```

### Virtual Instruments

```lua
Probe("name", { type = "E_field", position = {0,0,0} })
GaussMeter({x,y,z}, { range = "mT" })
Oscilloscope({x,y,z}, { range = 5, timebase = 0.001 })
MagneticFieldPlane("XZ", offset, { style = "arrows" })
ElectricFieldPlane("XY", offset, { style = "colormap" })
```

### View Configuration

```lua
view({
  camera = "isometric",  -- or {position = {x,y,z}, target = {x,y,z}}
  distance = 100,
  clip = { plane = "XZ", offset = 0 },
  transparency = { substrate = 0.3 },
  theme = "dark",
  grid = { show = true, size = 100 },
})
```

### Export

```lua
export_stl("file.stl", object)
export_step("file.step", object)
export_gltf("file.glb", object, { binary = true })
export_obj("file.obj", object)
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `LMB` | Rotate camera |
| `RMB` | Pan camera |
| `Scroll` | Zoom |
| `R` | Reset view |
| `F` | Focus on selection |
| `G` | Toggle grid |

## Roadmap

- [ ] **Phase 1**: Core CAD primitives and CSG
- [ ] **Phase 2**: Neovim LSP integration
- [ ] **Phase 3**: Basic FEM solver (electromagnetic)
- [ ] **Phase 4**: Virtual instruments and visualization
- [ ] **Phase 5**: AI-assisted primitive generation
- [ ] **Phase 6**: Export to STL/STEP

## License

MIT
