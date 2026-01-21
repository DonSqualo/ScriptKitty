# colormap_plane

Per-project visualization primitive. Generate this when field visualization is needed.

## Function Signature

```lua
colormap_plane(config)
```

## Config Parameters

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| plane | string | "XZ" | Plane orientation: "XY", "XZ", or "YZ" |
| bounds | table | x:-10..10, z:0..10 | Plane extent in world coordinates |
| offset | number | 0 | Distance from origin along plane normal |
| resolution | number | 50 | Grid resolution (points per axis) |
| colormap_data | table | nil | 2D array of scalar values |
| color_map | string | "jet" | Colormap: "jet", "viridis", "plasma", etc. |
| opacity | number | 0.9 | Transparency (0-1) |

## Usage

```lua
colormap_plane({
  plane = "XZ",
  bounds = {x_min = -50, x_max = 50, z_min = 0, z_max = 100},
  colormap_data = field_values,
  color_map = "jet"
})
```

## Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `at` | `(x, y, z)` | Position the plane |
| `opacity` | `(a)` | Set transparency |
| `serialize` | `()` | Convert to JSON |

## Internal Structure

```lua
viz = {
  _type = "visualization",
  _viz_type = "colormap_plane",
  _plane = "XZ",
  _bounds = {x_min, x_max, z_min, z_max},
  _offset = 0,
  _resolution = 50,
  _colormap_data = {{...}, {...}},
  _color_map = "jet",
  _opacity = 0.9,
  _position = nil,
}
```

## Serialization

```lua
{
  type = "colormap_plane",
  plane = "XZ",
  bounds = {...},
  offset = 0,
  resolution = 50,
  colormap_data = {{...}},
  color_map = "jet",
  opacity = 0.9,
  position = {x, y, z},
}
```

## Implementation Notes

- Bounds format depends on plane: XZ uses `{x_min, x_max, z_min, z_max}`, XY uses `{x_min, x_max, y_min, y_max}`, etc.
- colormap_data is row-major: `data[row][col]` where row varies along second axis of plane
- Renderer interpolates colors between grid points
- Used with MagneticFieldPlane, TemperaturePlane, AcousticPressurePlane instruments
