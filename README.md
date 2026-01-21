# ScriptKitten

A script-first CAD system with multiphysics simulation capabilities for designing ultrasound and MRI components. OpenSCAD-inspired Lua scripting with real-time preview.


{ cat PROMPTx1.md; echo ""; } | cc

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
local ScriptCAD = require("stdlib")

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

ScriptCAD.register(assembly)
return ScriptCAD.serialize()
```

## Project Structure

```
stdlib/           # Lua standard library
renderer/         # WebGPU frontend
server/           # Rust backend
examples/         # Example scripts
```

## Examples

### Helmholtz Coil with Bridge Gap Resonator

`examples/multiphysics/helmholtz_coil.lua` - A Helmholtz coil pair with a bridge gap resonator for uniform magnetic field generation.

Includes a 3D printable scaffold (`helmholtz_scaffold.stl`) with:
- Tube that fits inside the coil inner radius
- Stoppers at coil ends and inside the gap
- Box support for the resonator tube with bridge cutout
- Axial hole through the coil axis

## License

MIT
