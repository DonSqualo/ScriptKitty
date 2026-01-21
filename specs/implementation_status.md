# Implementation Status

Actual backend implementation status vs Lua API declarations (post-cleanup 2026-01-21).

## Export Formats

| Format | Lua API | Rust Backend | Status |
|--------|---------|--------------|--------|
| STL | `export_stl()` | `write_stl()` in export.rs | **Complete** |
| 3MF | `export_3mf()` | `write_3mf()` in export.rs | **Complete** |

## CSG Operations

| Operation | Lua API | Rust Backend | Status |
|-----------|---------|--------------|--------|
| union | `union()` | geometry.rs (manifold3d) | **Complete** |
| difference | `difference()` | geometry.rs (manifold3d) | **Complete** |
| intersect | `intersect()` | geometry.rs (manifold3d) | **Complete** |

## Physics Simulations

| Type | Lua API | Rust Backend | Status |
|------|---------|--------------|--------|
| Helmholtz magnetic field | `magnetostatic()` | field.rs (Biot-Savart) | **Pattern-matched** |
| Acoustic (deleted) | - | - | See specs/acoustic.md |

**Pattern-matched**: Backend recognizes specific keywords (e.g., "helmholtz", "coil_mean_radius") and runs hardcoded computation.

## Instruments/Visualizations

| Instrument | Lua API | Backend | Renderer | Status |
|------------|---------|---------|----------|--------|
| MagneticFieldPlane (colormap) | Yes | field.rs | XZ plane only | **Partial** |
| MagneticFieldPlane (arrows) | Yes | field.rs | 3D arrows | **Complete** |
| 1D line plot | Implicit | field.rs | Canvas graph | **Complete** |
| Probe | Yes | Serializes only | - | **API only** |
| GaussMeter | Yes | Serializes only | - | **API only** |

## Renderer Capabilities

| Feature | Lua API | Renderer | Status |
|---------|---------|----------|--------|
| Mesh rendering | Implicit | Three.js + custom shader | **Complete** |
| Flat shading | `flat_shading` | dFdx/dFdy normals | **Complete** |
| XZ colormap plane | `plane = "XZ"` | create_field_plane() | **Complete** |
| Jet colormap | `color_map = "jet"` | value_to_color() | **Complete** |
| XY/YZ planes | Ignored | Hardcoded to XZ | **Not implemented** |
| Other colormaps | Ignored | Hardcoded to jet | **Not implemented** |

## WebSocket Message Types

| Header | Purpose | Status |
|--------|---------|--------|
| `VIEW` | Render config (flat_shading) | **Complete** |
| `FIELD` | Field visualization data | **Complete** |
| (none) | Mesh geometry data | **Complete** |

## Key Files Reference

| Component | File | Notes |
|-----------|------|-------|
| STL writer | server/src/export.rs | Binary STL with normals |
| 3MF writer | server/src/export.rs | ZIP archive with colors |
| Helmholtz field | server/src/field.rs | Biot-Savart computation |
| CSG operations | server/src/geometry.rs | manifold3d bindings |
| Renderer colormap | renderer/src/main.ts | Jet colormap only |
| Renderer arrows | renderer/src/main.ts | 3D vector field |
