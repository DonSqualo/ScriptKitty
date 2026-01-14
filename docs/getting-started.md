# Getting Started with ScriptCAD

This guide will walk you through setting up ScriptCAD and creating your first model.

## Installation

### Prerequisites

1. **Rust** (1.70 or newer)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Node.js** (18 or newer)
   ```bash
   # Using nvm (recommended)
   nvm install 18
   nvm use 18
   ```

3. **WebGPU-compatible Browser**
   - Chrome 113+
   - Edge 113+
   - Firefox Nightly (with flags)

### Setup

```bash
# Clone or create the project
cd scriptcad

# Install renderer dependencies
cd renderer
npm install

# Build server (first time takes a while due to Lua compilation)
cd ../server
cargo build --release
```

## Running ScriptCAD

You'll need two terminal windows:

**Terminal 1 - Renderer:**
```bash
cd renderer
npm run dev
```
This starts the Vite dev server on http://localhost:3000

**Terminal 2 - Server:**
```bash
cd server
cargo run --release -- \
  --watch ../examples \
  --file ../examples/multiphysics/bridge_gap_resonator.lua \
  --port 3001
```

Open http://localhost:3000 in Chrome or Edge.

## Your First Model

Create a new file `my_first_model.lua`:

```lua
-- Load the standard library
local ScriptCAD = require("stdlib")

-- Define materials
local steel = material("steel")
local copper = material("copper")

-- Create geometry
local base_plate = box(100, 100, 5)
  :at(0, 0, 2.5)
  :material(steel)
  :name("base_plate")

local pillar = cylinder(8, 40)
  :at(0, 0, 25)
  :material(copper)
  :name("pillar")

local top_sphere = sphere(15)
  :at(0, 0, 55)
  :material(copper)
  :name("top_sphere")

-- Group into assembly
local assembly = group("my_assembly", {
  base_plate,
  pillar,
  top_sphere
})

-- Register with scene
ScriptCAD.register(assembly)

-- Configure the view
view({
  camera = "isometric",
  distance = 150,
  grid = { show = true, size = 120 },
})

-- Export options
export_stl("my_model.stl", assembly)

-- Return scene data
return ScriptCAD.serialize()
```

Save the file and start the server pointing to it:
```bash
cargo run -- --watch . --file my_first_model.lua
```

## Workflow with Neovim

ScriptCAD is designed for a terminal-centric workflow:

```
┌─────────────────────────────────────────────────────────────┐
│                         tmux                                 │
├─────────────────────────────┬───────────────────────────────┤
│                             │                               │
│          Neovim             │        Browser/Kitty          │
│                             │                               │
│   my_model.lua              │     ┌─────────────────────┐   │
│   ─────────────             │     │                     │   │
│   local base = box(...)     │     │   3D Preview        │   │
│   :at(0, 0, 5)              │     │                     │   │
│   :material(steel)          │     │      [cube]         │   │
│                             │     │                     │   │
│                             │     └─────────────────────┘   │
│                             │                               │
├─────────────────────────────┴───────────────────────────────┤
│  server: Watching my_model.lua... Reloaded!                 │
└─────────────────────────────────────────────────────────────┘
```

### Recommended tmux Layout

```bash
# Create session
tmux new-session -s scriptcad

# Split for editor and preview
tmux split-window -h

# Left pane: editor
nvim my_model.lua

# Right pane: could be browser or status
# Use a terminal browser like w3m or just watch the external browser
```

### File Watching

The server automatically watches for `.lua` file changes:

1. Edit your file in Neovim
2. Save with `:w`
3. Server detects change → Re-executes Lua
4. WebSocket pushes update → Browser refreshes

Typical reload time: <100ms for simple scenes.

## Parametric Design

Use variables for flexible designs:

```lua
-- Parameters at the top
local config = {
  base_size = 100,
  hole_diameter = 10,
  hole_count = 4,
  hole_margin = 15,
}

-- Derived values
local hole_radius = config.hole_diameter / 2
local hole_offset = config.base_size / 2 - config.hole_margin

-- Create geometry using parameters
local base = box(config.base_size, config.base_size, 10)

-- Create holes using pattern
local hole = cylinder(hole_radius, 20):at(hole_offset, hole_offset, 5)
local holes = circular_pattern(hole, config.hole_count, hole_offset, "Z")

-- Boolean subtract
local result = difference(base, holes)
```

## Next Steps

- Read the [Lua API Reference](./lua-api.md)
- Explore [examples/](../examples/)
- Learn about [Architecture](./architecture.md)
- Set up [Neovim integration](./neovim-setup.md)
