//! Manifold-based CSG geometry backend
//! Uses manifold3d for guaranteed watertight manifold meshes

use anyhow::{anyhow, Result};
use manifold3d::manifold::Rotation;
use manifold3d::types::{NormalizedAngle, PositiveF64, Vec3};
use manifold3d::{Manifold, MeshGL};
use mlua::{Lua, Value};
use std::alloc::{alloc, Layout};
use std::os::raw::c_void;

use crate::geometry::MeshData;

// FFI to access tri_verts which isn't exposed in the high-level Rust API
extern "C" {
    fn manifold_meshgl_tri_verts(mem: *mut c_void, m: *mut std::ffi::c_void) -> *mut u32;
}

fn get_mesh_indices(mesh: &MeshGL, count: usize) -> Vec<u32> {
    if count == 0 {
        return vec![];
    }
    let layout = Layout::array::<u32>(count).unwrap();
    let array_ptr = unsafe { alloc(layout) } as *mut u32;

    // MeshGL stores its internal pointer at offset 0
    let mesh_ptr = unsafe { std::ptr::read(mesh as *const MeshGL as *const *mut c_void) };

    unsafe {
        manifold_meshgl_tri_verts(array_ptr as *mut c_void, mesh_ptr);
        Vec::from_raw_parts(array_ptr, count, count)
    }
}

fn manifold_to_mesh_data(manifold: &Manifold) -> MeshData {
    let mesh = manifold.as_mesh();
    let properties = mesh.vertex_properties();
    let num_props = mesh.properties_per_vertex_count() as usize;
    let num_verts = mesh.vertex_count() as usize;
    let num_tris = mesh.triangle_count() as usize;
    let index_count = num_tris * 3;

    let mut data = MeshData::new_empty();

    if num_verts == 0 || num_props < 3 {
        return data;
    }

    // Extract positions (first 3 properties per vertex)
    for i in 0..num_verts {
        let base = i * num_props;
        if base + 2 < properties.len() {
            data.positions.push(properties[base]);
            data.positions.push(properties[base + 1]);
            data.positions.push(properties[base + 2]);
        }
    }

    // Get actual triangle indices via FFI
    let indices = get_mesh_indices(&mesh, index_count);
    data.indices = indices;

    // Initialize normals
    data.normals = vec![0.0; num_verts * 3];
    let mut counts = vec![0u32; num_verts];

    // Compute normals per-face and average at vertices
    for tri in 0..num_tris {
        let base = tri * 3;
        if base + 2 >= data.indices.len() {
            continue;
        }

        let i0 = data.indices[base] as usize;
        let i1 = data.indices[base + 1] as usize;
        let i2 = data.indices[base + 2] as usize;

        if i0 >= num_verts || i1 >= num_verts || i2 >= num_verts {
            continue;
        }

        let v0 = [
            data.positions[i0 * 3],
            data.positions[i0 * 3 + 1],
            data.positions[i0 * 3 + 2],
        ];
        let v1 = [
            data.positions[i1 * 3],
            data.positions[i1 * 3 + 1],
            data.positions[i1 * 3 + 2],
        ];
        let v2 = [
            data.positions[i2 * 3],
            data.positions[i2 * 3 + 1],
            data.positions[i2 * 3 + 2],
        ];

        let edge1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let edge2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
        let normal = cross(edge1, edge2);

        for &idx in &[i0, i1, i2] {
            data.normals[idx * 3] += normal[0];
            data.normals[idx * 3 + 1] += normal[1];
            data.normals[idx * 3 + 2] += normal[2];
            counts[idx] += 1;
        }
    }

    // Normalize the normals
    for i in 0..num_verts {
        if counts[i] > 0 {
            let len = (data.normals[i * 3].powi(2)
                + data.normals[i * 3 + 1].powi(2)
                + data.normals[i * 3 + 2].powi(2))
            .sqrt();
            if len > 1e-10 {
                data.normals[i * 3] /= len;
                data.normals[i * 3 + 1] /= len;
                data.normals[i * 3 + 2] /= len;
            }
        }
    }

    // Default white color
    data.colors = vec![1.0; num_verts * 3];

    data
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn pos(v: f64) -> PositiveF64 {
    PositiveF64::new(v.abs().max(0.001)).unwrap()
}

fn build_manifold_primitive(obj_type: &str, params: &mlua::Table) -> Result<Manifold> {
    match obj_type {
        "cylinder" => {
            let r: f64 = params.get("r")?;
            let h: f64 = params.get("h")?;
            // origin_at_center = false: base at z=0, extends to z=h
            Ok(Manifold::new_cylinder(
                pos(h),
                pos(r),
                None::<PositiveF64>,
                None::<manifold3d::types::PositiveI32>,
                false,
            ))
        }
        "box" => {
            let w: f64 = params.get("w")?;
            let d: f64 = params.get::<_, f64>("d").unwrap_or(w);
            let h: f64 = params.get("h")?;
            // origin_at_center = false: corner at origin, extends to (w, d, h)
            Ok(Manifold::new_cuboid(pos(w), pos(d), pos(h), false))
        }
        "sphere" => {
            let r: f64 = params.get("r")?;
            // Spheres are always centered at origin
            Ok(Manifold::new_sphere(
                pos(r),
                None::<manifold3d::types::PositiveI32>,
            ))
        }
        "cube" => {
            let size: f64 = params.get("size").unwrap_or(1.0);
            Ok(Manifold::new_cuboid(pos(size), pos(size), pos(size), false))
        }
        _ => Err(anyhow!("Unknown primitive type: {}", obj_type)),
    }
}

fn apply_manifold_ops(manifold: Manifold, table: &mlua::Table) -> Result<Manifold> {
    let mut result = manifold;

    if let Ok(ops) = table.get::<_, mlua::Table>("ops") {
        for pair in ops.pairs::<i64, mlua::Table>() {
            if let Ok((_, op_table)) = pair {
                let op: String = op_table.get("op").unwrap_or_default();
                let x: f64 = op_table.get("x").unwrap_or(0.0);
                let y: f64 = op_table.get("y").unwrap_or(0.0);
                let z: f64 = op_table.get("z").unwrap_or(0.0);

                tracing::debug!("Applying op: {} ({}, {}, {})", op, x, y, z);

                result = match op.as_str() {
                    "translate" => result.translate(Vec3::new(x, y, z)),
                    "rotate" => {
                        let rotation = Rotation::new(
                            NormalizedAngle::from_degrees(x as f32),
                            NormalizedAngle::from_degrees(y as f32),
                            NormalizedAngle::from_degrees(z as f32),
                        );
                        result.rotate(rotation)
                    }
                    "scale" => result.scale(Vec3::new(x, y, z)),
                    _ => result,
                };
            }
        }
    }

    Ok(result)
}

fn build_manifold_object(table: &mlua::Table) -> Result<Manifold> {
    let obj_type: String = table.get("type")?;
    let name: String = table.get("name").unwrap_or_default();
    tracing::debug!("Building manifold object: type={}, name={}", obj_type, name);

    if obj_type == "csg" {
        let operation: String = table.get("operation")?;
        let children: mlua::Table = table.get("children")?;

        let first_child: mlua::Table = children.get(1)?;
        let mut result = build_manifold_object(&first_child)?;

        for i in 2..=children.len()? {
            let child: mlua::Table = children.get(i)?;
            let child_manifold = build_manifold_object(&child)?;
            result = match operation.as_str() {
                "union" => result.union(&child_manifold),
                "difference" => result.difference(&child_manifold),
                "intersect" => result.intersection(&child_manifold),
                _ => return Err(anyhow!("Unknown CSG operation: {}", operation)),
            };
        }

        apply_manifold_ops(result, table)
    } else if obj_type == "group" {
        let children: mlua::Table = table.get("children")?;
        let mut result: Option<Manifold> = None;

        for pair in children.pairs::<i64, mlua::Table>() {
            let (_, child) = pair?;
            let child_manifold = build_manifold_object(&child)?;
            result = Some(match result {
                Some(r) => r.union(&child_manifold),
                None => child_manifold,
            });
        }

        let manifold = result.ok_or_else(|| anyhow!("Empty group"))?;
        apply_manifold_ops(manifold, table)
    } else {
        let params: mlua::Table = table.get("params")?;
        let manifold = build_manifold_primitive(&obj_type, &params)?;
        apply_manifold_ops(manifold, table)
    }
}

fn apply_material_color(mesh: &mut MeshData, table: &mlua::Table) {
    if let Ok(material) = table.get::<_, mlua::Table>("material") {
        if let Ok(color) = material.get::<_, mlua::Table>("color") {
            let r: f32 = color.get(1).unwrap_or(1.0);
            let g: f32 = color.get(2).unwrap_or(1.0);
            let b: f32 = color.get(3).unwrap_or(1.0);
            for i in 0..mesh.colors.len() / 3 {
                mesh.colors[i * 3] = r;
                mesh.colors[i * 3 + 1] = g;
                mesh.colors[i * 3 + 2] = b;
            }
        }
    }
}

/// Build mesh from a serialized object using Manifold
pub fn build_object_manifold(table: &mlua::Table) -> Result<MeshData> {
    let manifold = build_manifold_object(table)?;
    let mut mesh = manifold_to_mesh_data(&manifold);
    apply_material_color(&mut mesh, table);
    Ok(mesh)
}

/// Generate mesh from Lua scene using Manifold backend
pub fn generate_mesh_from_lua_manifold(_lua: &Lua, value: &Value) -> Result<MeshData> {
    let table = value.as_table().ok_or_else(|| anyhow!("Expected table"))?;
    let objects: mlua::Table = table.get("objects")?;

    let mut combined: Option<Manifold> = None;

    for pair in objects.pairs::<i64, mlua::Table>() {
        let (_, obj) = pair?;
        let manifold = build_manifold_object(&obj)?;
        combined = Some(match combined {
            Some(c) => c.union(&manifold),
            None => manifold,
        });
    }

    let final_manifold = combined.ok_or_else(|| anyhow!("No objects in scene"))?;
    Ok(manifold_to_mesh_data(&final_manifold))
}

/// Generate mesh from a single serialized object using Manifold
pub fn generate_mesh_from_object_manifold(_lua: &Lua, table: &mlua::Table) -> Result<MeshData> {
    build_object_manifold(table)
}
