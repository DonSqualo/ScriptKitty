# Mittens 🐱

*"My CAD's breath smells like cat food."* — Ralph Wiggum, probably

[![Mittens](https://static.wikia.nocookie.net/simpsons/images/a/ad/Snookums.jpg/revision/latest/smart/width/386/height/259?cb=20191119154645)](https://simpsons.fandom.com/wiki/Mittens)

A script-first CAD system with multiphysics simulation capabilities for designing ultrasound and MRI components. OpenSCAD-inspired Lua scripting with real-time preview.

Named after Ralph Wiggum's cat, because every good CAD needs a feline mascot, and this one runs on `ralph.sh`.

## Running

```bash
# Terminal 1: Renderer
cd renderer
npm install
npm run dev

# Terminal 2: Server
cd server
cargo run -- --watch ../examples --file ../examples/multiphysics/pure_acoustics.lua
```

Open http://localhost:3000 in Chrome/Edge (WebGPU required).

## Example

```lua
local Mittens = require("stdlib")

Coverslip = {
  diameter = 30,
  thickness = 0.17,
}

Coverslip.model = cylinder(Coverslip.diameter / 2, Coverslip.thickness)

Oring = {
  outer_diameter = 28,
  inner_diameter = 24,
  thickness = 1.5,
}

Oring.model = difference(
      cylinder(Oring.outer_diameter / 2, Oring.thickness),
      cylinder(Oring.inner_diameter / 2, Oring.thickness)
    )
    :at(0, 0, Coverslip.thickness)

local assembly = group("assembly", { Coverslip.model, Oring.model })

Mittens.register(assembly)
return Mittens.serialize()
```

## Project Structure

```
stdlib/           # Lua standard library
renderer/         # WebGPU frontend (Three.js)
server/           # Rust backend (Manifold CSG)
examples/         # Example scripts
```

## Features

- **Lua scripting** — OpenSCAD-inspired syntax with full Lua power
- **Real-time preview** — Hot reload on file changes
- **Multiphysics simulation**
  - Magnetic field (Biot-Savart, Helmholtz coils)
  - Acoustic pressure (Rayleigh-Sommerfeld)
  - NanoVNA S11 frequency sweep
  - Circuit analysis
- **Export** — STL, 3MF

## Examples

### Helmholtz Coil with Bridge Gap Resonator

`examples/multiphysics/bridge_gap_resonator.lua` — Helmholtz coil pair with a bridge gap resonator for uniform magnetic field generation.

## License

MIT
