# AGENT.md

Build and development notes for Claude agents.

## Build Commands

### Server (Rust)
```bash
cd server
cargo test        # Run tests (5 tests in export.rs)
cargo build       # Debug build
cargo build --release  # Release build
```

### Renderer (TypeScript)
```bash
cd renderer
npm install       # Install dependencies
npm run build     # Production build to dist/
npm run dev       # Development server
```

## Running the Application

1. Start the server with a Lua file:
```bash
cd server
cargo run --release -- ../project/helmholtz_coil.lua
```

2. Serve the renderer (in another terminal):
```bash
cd renderer
npm run dev
```

3. Open browser to http://localhost:5173

The server watches the Lua file for changes and pushes updates via WebSocket to port 3001.

## Project Structure

- `stdlib/` - Lua standard library loaded by scripts
- `server/src/` - Rust backend (geometry, field computation, export)
- `renderer/src/` - TypeScript frontend (Three.js)
- `project/` - Current project Lua files
- `specs/` - Feature specifications and learnings

## Key Patterns

### Helmholtz Field Computation
The server pattern-matches for `helmholtz` or `coil_mean_radius` in Lua content to trigger Biot-Savart computation. Config is read from the `config` global table.

### Binary Protocols
- Mesh data: `[num_vertices:u32][num_indices:u32][positions][normals][colors][indices]`
- Field data: Header `FIELD\0\0\0`, then slice dims, bounds, Bx, Bz, magnitude, arrows, line data
- View config: Header `VIEW\0\0\0\0`, then `flat_shading:u8`
- Circuit: Header `CIRCUIT\0`, then size, SVG data

## Test Files

- `server/src/export.rs` - 5 tests for STL/3MF export
