# ScriptCAD Lua API Reference

Complete reference for the ScriptCAD Lua standard library.

## Table of Contents

1. [Primitives](#primitives)
2. [Transforms](#transforms)
3. [CSG Operations](#csg-operations)
4. [Materials](#materials)
5. [Groups](#groups)
6. [Physics](#physics)
7. [Instruments](#instruments)
8. [View](#view)
9. [Export](#export)

---

## Primitives

All primitives return a Shape object with chainable methods.

### box(w, d, h)

Creates a rectangular box centered at origin.

```lua
local b = box(50, 30, 20)  -- width, depth, height
local cube = box(50)       -- cube shorthand (all sides equal)
```

**Parameters:**
- `w` (number): Width (X dimension)
- `d` (number, optional): Depth (Y dimension), defaults to `w`
- `h` (number, optional): Height (Z dimension), defaults to `w`

### sphere(r)

Creates a sphere centered at origin.

```lua
local s = sphere(25)  -- radius 25
```

### cylinder(r, h)

Creates a cylinder along the Z axis, centered at origin.

```lua
local c = cylinder(10, 50)  -- radius 10, height 50
```

### cone(r, h)

Creates a cone with base radius `r` and height `h`.

```lua
local c = cone(15, 30)  -- base radius, height
```

### torus(major_r, minor_r)

Creates a torus (donut shape).

```lua
local t = torus(20, 5)  -- major radius, minor radius (tube)
```

### capsule(r, h)

Creates a capsule (cylinder with spherical caps).

```lua
local c = capsule(5, 30)  -- radius, total height
```

### helix(params)

Creates a helical coil.

```lua
local coil = helix({
  inner_radius = 10,
  outer_radius = 20,   -- for spiral, omit for solenoid
  turns = 5,
  pitch = 2,           -- vertical spacing per turn
  wire_diameter = 1,
  style = "spiral"     -- or "solenoid"
})
```

### plane(nx, ny, nz, d)

Creates an infinite plane.

```lua
local p = plane(0, 0, 1, 0)  -- XY plane at z=0
```

---

## Shape Methods

All shapes support these chainable methods:

### :at(x, y, z)

Position the shape.

```lua
local b = box(10):at(50, 0, 25)
```

### :rotate(rx, ry, rz)

Rotate the shape (Euler angles in degrees).

```lua
local c = cylinder(5, 20):rotate(90, 0, 0)  -- rotate to lie along Y
```

### :scale(sx, sy, sz)

Scale the shape.

```lua
local s = sphere(10):scale(1, 1, 2)  -- ellipsoid
local s2 = sphere(10):scale(2)       -- uniform scale
```

### :material(mat)

Assign a material.

```lua
local b = box(10):material(material("copper"))
```

### :color(r, g, b, a)

Set display color (overrides material color).

```lua
local b = box(10):color(1, 0, 0, 1)  -- red
```

### :name(n)

Name the shape for selection/visibility control.

```lua
local b = box(10):name("base_plate")
```

---

## Transforms

Standalone transform functions (alternative to methods).

### translate(shape, x, y, z)

```lua
local moved = translate(box(10), 50, 0, 0)
```

### rotate(shape, rx, ry, rz)

```lua
local rotated = rotate(cylinder(5, 20), 90, 0, 0)
```

### scale(shape, sx, sy, sz)

```lua
local scaled = scale(sphere(10), 2, 2, 1)
```

### mirror(shape, plane)

Mirror across a plane.

```lua
local mirrored = mirror(box(10):at(20, 0, 0), "YZ")  -- mirror across YZ
-- Planes: "XY", "XZ", "YZ"
```

### linear_pattern(shape, count, dx, dy, dz)

Create a linear array of shapes.

```lua
local holes = linear_pattern(cylinder(3, 10), 5, 15, 0, 0)
```

### circular_pattern(shape, count, radius, axis)

Create a circular array of shapes.

```lua
local bolts = circular_pattern(cylinder(3, 10):at(20, 0, 0), 6, 20, "Z")
```

---

## CSG Operations

### union(...)

Combine shapes (additive).

```lua
local combined = union(box(20), sphere(15))
local combined2 = union({shape1, shape2, shape3})  -- table form
```

### difference(base, ...)

Subtract shapes from base.

```lua
local with_hole = difference(box(30), cylinder(5, 40))
```

### intersect(...)

Keep only overlapping volume.

```lua
local lens = intersect(sphere(20):at(-5, 0, 0), sphere(20):at(5, 0, 0))
```

### smooth_union(k, ...)

Union with smooth blending.

```lua
local blob = smooth_union(3, sphere(10), sphere(10):at(15, 0, 0))
-- k is the blend radius
```

### shell(shape, thickness)

Create a hollow shell.

```lua
local hollow = shell(sphere(20), 2)  -- 2mm wall thickness
```

---

## Materials

### material(name, properties)

Create or retrieve a material.

```lua
-- From database
local copper = material("copper")
local steel = material("steel")

-- Custom properties
local custom_pcb = material("FR4", {
  permittivity = 4.4,
  loss_tangent = 0.02
})

-- Fully custom
local my_material = material("custom_alloy", {
  conductivity = 1e6,
  permeability = 100,
  density = 7500,
  color = {0.6, 0.6, 0.7, 1.0}
})
```

**Built-in Materials:**

| Name | Description |
|------|-------------|
| `copper` | Standard copper |
| `aluminum` | Aluminum alloy |
| `steel` | Carbon steel |
| `stainless_steel` | 304 stainless |
| `gold` | Pure gold |
| `fr4` | PCB substrate |
| `ptfe` | Teflon |
| `glass` | Borosilicate |
| `ferrite` | MnZn ferrite |
| `neodymium` | N52 magnet |
| `pla` | 3D print PLA |
| `abs` | ABS plastic |
| `air` | Air at STP |
| `vacuum` | Vacuum |
| `water` | Water at 20°C |

---

## Groups

### group(name, children)

Create a named group.

```lua
local motor = group("motor", {
  stator,
  rotor,
  windings
})
```

### assembly(name, children, metadata)

Top-level assembly with metadata.

```lua
local device = assembly("my_device", {
  base,
  mechanism,
  cover
}, {
  author = "John",
  version = "1.0"
})
```

### component(name, children)

Reusable component with instancing.

```lua
local bolt = component("M3_bolt", { head, shaft })

-- Create instances
local bolt1 = bolt:instance():at(10, 10, 0)
local bolt2 = bolt:instance():at(10, -10, 0)
```

---

## Physics

### electromagnetic(config)

Set up electromagnetic study.

```lua
local em = electromagnetic({
  type = "frequency_domain",  -- or "time_domain", "eigenfrequency"
  frequencies = linspace(1e9, 10e9, 100),
  ports = { port(pad1, pad2, { impedance = 50 }) },
  solver = "direct",
  formulation = "full_wave"
})
  :domain(assembly)
  :boundary("outer", { type = "radiation" })
  :mesh({ max_element_size = 2, min_element_size = 0.1 })
```

### magnetostatic(config)

```lua
local mag = magnetostatic({
  nonlinear = true  -- for saturation effects
})
```

### thermal(config)

```lua
local heat = thermal({
  type = "steady_state",  -- or "transient"
  ambient = 293.15        -- Kelvin
})
```

### structural(config)

```lua
local stress = structural({
  type = "static",  -- or "eigenfrequency", "transient"
  large_deformation = false
})
```

### Utility Functions

```lua
linspace(start, stop, count)   -- Linear spacing
logspace(start, stop, count)   -- Logarithmic spacing
port(pos, neg, { impedance })  -- S-parameter port
current_source(pos, neg, I)    -- Current excitation
voltage_source(pos, neg, V)    -- Voltage excitation
heat_source(domain, P)         -- Heat source
```

---

## Instruments

Virtual measurement instruments for visualization.

### Probe(name, config)

Point measurement.

```lua
Probe("E_at_gap", {
  type = "E_field",       -- E_field, H_field, voltage, current
  position = {0, 0, 5},
  component = "magnitude" -- x, y, z, or magnitude
})
```

### GaussMeter(position, config)

Magnetic field measurement.

```lua
GaussMeter({0, 0, 0}, {
  range = "mT",       -- T, mT, uT, G
  component = "z"
})
```

### Oscilloscope(position, config)

Time-domain measurement.

```lua
Oscilloscope({0, 0, 0}, {
  range = 5,          -- ±5V
  timebase = 0.001,   -- 1ms/div
  channels = 2
})
```

### MagneticFieldPlane(plane, offset, config)

2D field visualization.

```lua
MagneticFieldPlane("XZ", 0, {
  quantity = "H",         -- H or B
  style = "arrows",       -- arrows, streamlines, colormap
  scale = "log",
  resolution = 30,
  color_map = "viridis"
})
```

### ElectricFieldPlane(plane, offset, config)

```lua
ElectricFieldPlane("XY", 5, {
  quantity = "E",
  style = "colormap",
  color_map = "plasma"
})
```

### Streamlines(config)

3D field streamlines.

```lua
Streamlines({
  field = "H",
  seeds = "grid",
  count = 100,
  length = 50
})
```

### SParams(study, config)

S-parameter output.

```lua
SParams(em_study, {
  plot = {"S11_dB", "S11_phase", "S11_smith"},
  export = "results/sparams.s1p"
})
```

---

## View

### view(config)

Configure the 3D view.

```lua
view({
  -- Camera
  camera = "isometric",   -- preset or custom:
  -- camera = { position = {100,100,100}, target = {0,0,0}, fov = 45 },
  distance = 150,

  -- Clipping
  clip = {
    plane = "XZ",
    offset = 0,
    show_caps = true
  },

  -- Visibility
  show = {"part1", "part2"},
  hide = {"debug_lines"},

  -- Transparency overrides
  transparency = {
    substrate = 0.3,
    cover = 0.5
  },

  -- Display settings
  theme = "dark",
  grid = { show = true, size = 100, divisions = 10 },
  axes = { show = true, size = 20 },

  -- Render quality
  render = {
    quality = "high",
    shadows = true,
    ambient_occlusion = true
  }
})
```

**Camera Presets:**
- `isometric`
- `front`, `back`
- `left`, `right`
- `top`, `bottom`

---

## Export

### export_stl(filename, object, config)

Export to STL (3D printing).

```lua
export_stl("output.stl", assembly, {
  binary = true,      -- binary or ASCII
  quality = "high"    -- affects tessellation
})
```

### export_step(filename, object, config)

Export to STEP (CAD interchange).

```lua
export_step("output.step", assembly, {
  version = "AP214"  -- AP203, AP214, AP242
})
```

### export_gltf(filename, object, config)

Export to glTF (web/realtime).

```lua
export_gltf("output.glb", assembly, {
  binary = true,
  draco = false       -- compression
})
```

### export_obj(filename, object)

Export to OBJ (simple mesh).

```lua
export_obj("output.obj", assembly)
```

### export_vtk(filename, bounds, config)

Export field data for ParaView.

```lua
export_vtk("fields.vtk", { min = {-50,-50,-50}, max = {50,50,50} }, {
  resolution = {100, 100, 100},
  fields = {"E_field", "H_field"}
})
```
