# export.lua

File export functions. Exports are QUEUED, not executed immediately.

## Export Queue

Exports are added to `Export._queue` when called. The queue is serialized and sent to Rust backend which processes them.

```lua
export_stl("part.stl", my_shape)
export_3mf("part.3mf", my_shape)
Export.get_queue()  -- returns queue
Export.clear()      -- empties queue
```

## Export Functions

### export_stl(filename, object, [circular_segments])

3D printing format.

| Param | Type | Default | Description |
|-------|------|---------|-------------|
| filename | string | required | Output path |
| object | shape/group | required | Object to export |
| circular_segments | number | 128 | Curve resolution |

### export_3mf(filename, object, [config])

Modern 3D printing format with color support.

| Config key | Type | Default | Description |
|------------|------|---------|-------------|
| units | string | "millimeter" | Unit system (3MF standard units) |
| color | boolean | true | Include colors |

## Global Shortcuts

- `export_stl`
- `export_3mf`

## Queue Entry Structure

```lua
{
  format = "stl" | "3mf",
  filename = "part.stl",
  object = <serialized>,
  circular_segments = 128,  -- stl only
  units = "millimeter",     -- 3mf only (3MF standard: millimeter, centimeter, meter, inch, foot, micron)
  include_colors = true,    -- 3mf only
}
```

## Backend Implementation

Location: `server/src/export.rs`

### STL Export

Binary STL format with computed face normals. 80-byte header + triangle data.

```rust
write_stl(mesh: &MeshData, path: &Path) -> io::Result<()>
```

### 3MF Export

ZIP archive containing XML model data. Uses `zip` crate with DEFLATE compression.

```rust
write_3mf(mesh: &MeshData, path: &Path, units: &str, include_colors: bool) -> io::Result<()>
```

Archive structure:
- `[Content_Types].xml` - MIME type declarations
- `_rels/.rels` - Root relationships
- `3D/3dmodel.model` - Mesh data with vertices, triangles, optional colors

Color support uses 3MF material extension (`m:colorgroup`) with per-vertex colors mapped to triangles via `pid`/`p1`/`p2`/`p3` attributes.
