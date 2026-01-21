# Implementation Status

Actual backend implementation status vs Lua API declarations (post-cleanup 2026-01-21).

## Export Formats

| Format | Lua API | Rust Backend | Status |
|--------|---------|--------------|--------|
| STL | `export_stl()` | `write_stl()` in export.rs | **Complete** |
| 3MF | `export_3mf()` | `write_3mf()` in export.rs | **Complete** |

## Primitives

| Primitive | Lua API | Rust Backend | Status |
|-----------|---------|--------------|--------|
| box | `box()` | geometry.rs | **Complete** |
| cylinder | `cylinder()` | geometry.rs | **Complete** |
| sphere | `sphere()` | geometry.rs | **Complete** |
| torus | `torus()` | geometry.rs | **Complete** |
| ring | `ring()` | geometry.rs | **Complete** |

## CSG Operations

| Operation | Lua API | Rust Backend | Status |
|-----------|---------|--------------|--------|
| union | `union()` | geometry.rs (manifold3d) | **Complete** |
| difference | `difference()` | geometry.rs (manifold3d) | **Complete** |
| intersect | `intersect()` | geometry.rs (manifold3d) | **Complete** |

## Physics Simulations

| Type | Lua API | Rust Backend | Tests | Status |
|------|---------|--------------|-------|--------|
| Helmholtz magnetic field | `magnetostatic()` | field.rs (Biot-Savart) | 7 | **Pattern-matched** |
| Acoustic pressure field | `acoustic()` | acoustic.rs (Rayleigh-Sommerfeld) | 8 | **Pattern-matched** |
| NanoVNA S11 sweep | `NanoVNA = {}` | nanovna.rs | 7 | **Pattern-matched** |

**Pattern-matched**: Backend recognizes specific keywords (e.g., "helmholtz", "coil_mean_radius", "Acoustic", "NanoVNA") and runs hardcoded computation.

## Instruments/Visualizations

| Instrument | Lua API | Backend | Renderer | Status |
|------------|---------|---------|----------|--------|
| MagneticFieldPlane | Yes | field.rs | XZ/XY/YZ planes | **Complete** |
| AcousticPressurePlane | Yes | acoustic.rs | XZ/XY/YZ planes | **Complete** |
| 1D line plot | Implicit | field.rs | Canvas graph | **Complete** |
| Probe | Yes | Serializes only | - | **API only** |
| GaussMeter | Yes | field.rs | Measurement | **Complete** |
| Hydrophone | Yes | acoustic.rs | Measurement | **Complete** |

## Renderer Capabilities

| Feature | Lua API | Renderer | Status |
|---------|---------|----------|--------|
| Mesh rendering | Implicit | Three.js + custom shader | **Complete** |
| Flat shading | `flat_shading` | dFdx/dFdy normals | **Complete** |
| XZ/XY/YZ planes | `plane = "XZ"/"XY"/"YZ"` | create_field_plane() | **Complete** |
| Jet colormap | `color_map = "jet"` | jet_colormap() | **Complete** |
| Viridis colormap | `color_map = "viridis"` | viridis_colormap() | **Complete** |
| Plasma colormap | `color_map = "plasma"` | plasma_colormap() | **Complete** |

## WebSocket Message Types

| Header | Purpose | Status |
|--------|---------|--------|
| `VIEW` | Render config (flat_shading) | **Complete** |
| `FIELD` | Field visualization data | **Complete** |
| `CIRCUIT` | SVG circuit diagram | **Complete** |
| `MEASURE` | Point measurement data | **Complete** |
| `NANOVNA` | Frequency sweep data | **Complete** |
| (none) | Mesh geometry data | **Complete** |

## Key Files Reference

| Component | File | Notes |
|-----------|------|-------|
| STL writer | server/src/export.rs | Binary STL with normals |
| 3MF writer | server/src/export.rs | ZIP archive with colors |
| Helmholtz field | server/src/field.rs | Biot-Savart computation |
| Acoustic field | server/src/acoustic.rs | Rayleigh-Sommerfeld diffraction |
| NanoVNA sweep | server/src/nanovna.rs | S11 frequency sweep simulation |
| CSG operations | server/src/geometry.rs | manifold3d bindings |
| Renderer colormap | renderer/src/main.ts | Jet/Viridis/Plasma colormaps |
| Renderer arrows | renderer/src/main.ts | 3D vector field |
