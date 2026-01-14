# ScriptCAD Architecture

Technical overview of ScriptCAD's design and implementation.

## System Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                              User                                        │
│                         (Neovim / Editor)                                │
└─────────────────────────────────┬───────────────────────────────────────┘
                                  │ .lua files
                                  ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         File System                                      │
│                    (watched by server)                                   │
└─────────────────────────────────┬───────────────────────────────────────┘
                                  │ notify events
                                  ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                      Rust Server (scriptcad-server)                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐    │
│  │ File        │  │ Lua         │  │ Scene       │  │ WebSocket   │    │
│  │ Watcher     │─▶│ Runner      │─▶│ Manager     │─▶│ Server      │    │
│  │ (notify)    │  │ (mlua)      │  │             │  │ (axum)      │    │
│  └─────────────┘  └─────────────┘  └─────────────┘  └──────┬──────┘    │
└─────────────────────────────────────────────────────────────┼───────────┘
                                                              │ JSON/WS
                                                              ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                      WebGPU Renderer (Browser)                           │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐    │
│  │ Scene       │  │ Camera      │  │ SDF         │  │ UI          │    │
│  │ Manager     │─▶│ Controller  │─▶│ Raymarcher  │─▶│ Overlay     │    │
│  │             │  │             │  │ (WGSL)      │  │             │    │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘    │
└─────────────────────────────────────────────────────────────────────────┘
```

## Component Details

### 1. Lua Standard Library (`stdlib/`)

The Lua stdlib provides the scripting API. Key design decisions:

**Signed Distance Functions (SDF)**

All geometry is represented as SDFs internally:
- Enables smooth CSG operations
- GPU-friendly for raymarching
- Infinite resolution (no mesh)

```lua
-- SDF for a sphere at origin with radius r
function sphere_sdf(p, r)
  return length(p) - r
end

-- SDF for a box with half-extents b
function box_sdf(p, b)
  local q = abs(p) - b
  return length(max(q, 0)) + min(max(q.x, q.y, q.z), 0)
end
```

**Method Chaining**

All shapes return objects with chainable methods using Lua metatables:

```lua
local Shape = {}
Shape.__index = Shape

function Shape:at(x, y, z)
  self.position = {x, y, z}
  return self  -- Return self for chaining
end
```

**Serialization**

Scene data is serialized to JSON for transmission:

```lua
function Shape:serialize()
  return {
    type = self.primitive_type,
    params = self.params,
    transform = self.transform,
    material = self.material
  }
end
```

### 2. Rust Server (`server/`)

Built with:
- **axum**: HTTP/WebSocket server
- **mlua**: Lua 5.4 interpreter
- **notify**: File system watching

**File Watching Pipeline:**

```
File Change → Debounce (200ms) → Read File → Execute Lua → Serialize → Broadcast
```

**Lua Execution:**

The server embeds a Lua 5.4 interpreter:

```rust
pub struct LuaRunner {
    lua: Lua,
}

impl LuaRunner {
    pub fn execute(&mut self, script: &str) -> Result<SceneData> {
        let result: Value = self.lua.load(script).eval()?;
        self.extract_scene_data(result)
    }
}
```

**WebSocket Protocol:**

Messages are JSON with a `type` field:

```json
// Server → Client
{
  "type": "scene_update",
  "payload": { "objects": [...], "view": {...} },
  "filename": "model.lua"
}

{
  "type": "compile_error",
  "payload": {
    "message": "attempt to call nil value 'undefined_func'",
    "line": 42
  }
}

// Client → Server
{
  "type": "load_file",
  "path": "/path/to/model.lua"
}

{
  "type": "update_view",
  "camera": { "position": [100, 100, 100] }
}
```

### 3. WebGPU Renderer (`renderer/`)

Built with:
- **WebGPU**: Modern GPU API
- **WGSL**: Shader language
- **TypeScript**: Application logic
- **Vite**: Build tool

**Rendering Pipeline:**

```
Scene JSON → Parse Objects → Generate WGSL → Compile Pipeline → Raymarch → Display
```

**SDF Raymarching Shader:**

```wgsl
fn ray_march(ro: vec3<f32>, rd: vec3<f32>) -> vec2<f32> {
  var t = 0.0;

  for (var i = 0; i < MAX_STEPS; i++) {
    let p = ro + rd * t;
    let d = scene_sdf(p);

    if (d.x < EPSILON) {
      return vec2(t, d.y);  // Hit: distance and material ID
    }

    t += d.x;

    if (t > MAX_DIST) {
      return vec2(-1.0, -1.0);  // Miss
    }
  }

  return vec2(-1.0, -1.0);
}
```

**Dynamic Scene Compilation:**

The scene SDF is generated at runtime from the JSON scene data:

```typescript
generateSDFCode(): string {
  let code = '';

  for (const obj of this.objects) {
    code += this.objectToSDF(obj);
  }

  code += `
fn scene_sdf(p: vec3<f32>) -> vec2<f32> {
  var d = vec2(1e10, 0.0);
  ${this.objects.map((_, i) => `d = op_union(d, sdf_${i}(p));`).join('\n')}
  return d;
}`;

  return code;
}
```

## Data Flow

### Scene Update Flow

```
1. User edits model.lua in Neovim
2. User saves file (:w)
3. notify detects file change
4. Server reads file content
5. Lua interpreter executes script
6. Script returns scene data table
7. Server serializes to JSON
8. WebSocket broadcasts to all clients
9. Renderer parses scene data
10. Renderer generates new WGSL code
11. Renderer recompiles pipeline (if needed)
12. New frame renders with updated scene
```

### AI Extension Flow (Future)

```
1. User writes: torus(50, 10)
2. Lua execution fails: "undefined function 'torus'"
3. Server captures error with context
4. Server sends error to AI service
5. AI generates implementation:
   function torus(major, minor)
     return Shape.new("torus", {major_r = major, minor_r = minor})
   end
6. Server injects into Lua environment
7. Server re-executes script
8. Success: scene renders with torus
9. Generated code saved to stdlib/primitives.lua
```

## Performance Considerations

### Lua Execution

- Scripts typically execute in <10ms
- Large arrays may benefit from LuaJIT (future)
- Complex CSG trees cached as compound SDFs

### WebSocket Updates

- Debounced to 200ms to batch rapid saves
- Binary protocol option for large scenes (future)
- Delta updates for incremental changes (future)

### GPU Rendering

- SDF raymarching: O(n) per pixel where n = objects
- Bounding volume hierarchy for large scenes (future)
- Level-of-detail for distant objects (future)
- Target: 60 FPS for <1000 objects

### Memory

- Lua interpreter: ~10MB base
- Scene data: typically <1MB
- GPU buffers: depends on resolution
- WebSocket: minimal overhead

## Extension Points

### Custom Primitives

Add to `stdlib/primitives.lua`:

```lua
function Primitives.custom_shape(params)
  local function sdf(x, y, z)
    -- Custom SDF implementation
    return distance
  end

  return Shape(sdf, bounds, {primitive = "custom_shape", params = params})
end
```

### Custom Materials

Add to `stdlib/materials.lua`:

```lua
Materials.database.my_alloy = {
  name = "My Alloy",
  conductivity = 1e6,
  permeability = 50,
  -- ...
}
```

### Custom Instruments

Add to `stdlib/instruments/`:

```lua
function Instruments.MyInstrument(position, config)
  return Instrument("my_instrument", position, {
    -- config
  })
end
```

### Server Extensions

The Rust server can be extended with:
- Additional file format parsers
- Physics solver integrations
- Export format handlers
- AI service connections

## Future Architecture

### Phase 2: Physics Integration

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│ Scene Data  │────▶│ Mesh Gen    │────▶│ FEM Solver  │
└─────────────┘     └─────────────┘     └─────────────┘
                                               │
                                               ▼
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│ Renderer    │◀────│ Field Data  │◀────│ Post-Proc   │
└─────────────┘     └─────────────┘     └─────────────┘
```

### Phase 3: Distributed Computation

```
┌─────────────────────────────────────────────────────┐
│                 Orchestrator                         │
└──────────┬───────────────┬───────────────┬──────────┘
           │               │               │
     ┌─────▼─────┐   ┌─────▼─────┐   ┌─────▼─────┐
     │ Worker 1  │   │ Worker 2  │   │ Worker N  │
     │ (EM Sim)  │   │ (Thermal) │   │ (Struct)  │
     └───────────┘   └───────────┘   └───────────┘
```
