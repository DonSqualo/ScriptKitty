# primitives.lua

Basic geometric shapes.

## Shape Object

All primitives return a Shape with these methods:

| Method | Signature | Description |
|--------|-----------|-------------|
| `at` | `(x, y, z)` | Translate shape |
| `rotate` | `(rx, ry, rz)` | Rotate in degrees |
| `scale` | `(sx, [sy], [sz])` | Scale (uniform if 1 arg) |
| `center` | `(cx, cy, cz)` | Center on axes where arg is true |
| `centerXY` | `()` | Center on X and Y only |
| `centered` | `()` | Center on all axes |
| `material` | `(mat)` | Set material name |
| `color` | `(r, g, b, [a])` | Set RGBA color (0-1) |
| `name` | `(n)` | Set object name |
| `eval` | `(x, y, z)` | Evaluate SDF at point |
| `serialize` | `()` | Convert to JSON-serializable table |

## Primitives

### box(w, [d], [h])

Creates a box with corner at origin, extending to +w, +d, +h.

| Param | Type | Default | Description |
|-------|------|---------|-------------|
| w | number | required | Width (X) |
| d | number | w | Depth (Y) |
| h | number | w | Height (Z) |

```lua
box(10)              -- 10x10x10 cube
box(20, 10, 5)       -- 20x10x5 box
box(10):centered()   -- Centered at origin
```

**Validation:** `box(10)` creates bounds `{min={0,0,0}, max={10,10,10}}`

### cylinder(r, h)

Creates a cylinder with base on XY plane at Z=0, extending to +h.

| Param | Type | Description |
|-------|------|-------------|
| r | number | Radius |
| h | number | Height |

```lua
cylinder(5, 20)              -- r=5, h=20
cylinder(5, 20):centered()   -- Center at origin
```

**Validation:** `cylinder(5, 20)` creates bounds `{min={-5,-5,0}, max={5,5,20}}`

## Internal Structure

```lua
shape = {
  _type = "shape",
  _sdf = function(x, y, z) ... end,
  _bounds = {min = {x, y, z}, max = {x, y, z}},
  _ops = {},      -- transforms added via method calls
  _material = nil,
  _color = nil,
  _name = nil,
  _metadata = {primitive = "box", params = {w=10, d=10, h=10}}
}
```
