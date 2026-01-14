//! Mesh generation with procedural primitives and CSG fallback
//!
//! Uses procedural generation for known shapes (produces clean manifolds),
//! with csgrs as fallback for arbitrary CSG operations.

use anyhow::{anyhow, Result};
use csgrs::mesh::Mesh as CsgMesh;
use csgrs::traits::CSG;
use mlua::{Lua, Value};
use std::f32::consts::PI;

/// Mesh data for WebSocket transfer
pub struct MeshData {
    pub positions: Vec<f32>,
    pub normals: Vec<f32>,
    pub indices: Vec<u32>,
}

impl MeshData {
    fn new() -> Self {
        MeshData {
            positions: Vec::new(),
            normals: Vec::new(),
            indices: Vec::new(),
        }
    }

    fn add_vertex(&mut self, pos: [f32; 3], normal: [f32; 3]) -> u32 {
        let idx = (self.positions.len() / 3) as u32;
        self.positions.extend_from_slice(&pos);
        self.normals.extend_from_slice(&normal);
        idx
    }

    fn add_triangle(&mut self, a: u32, b: u32, c: u32) {
        self.indices.extend_from_slice(&[a, b, c]);
    }

    fn merge(&mut self, other: &MeshData) {
        let offset = (self.positions.len() / 3) as u32;
        self.positions.extend_from_slice(&other.positions);
        self.normals.extend_from_slice(&other.normals);
        for &idx in &other.indices {
            self.indices.push(idx + offset);
        }
    }

    pub fn to_binary(&self) -> Vec<u8> {
        let num_vertices = (self.positions.len() / 3) as u32;
        let num_indices = self.indices.len() as u32;

        let mut data = Vec::new();
        data.extend_from_slice(&num_vertices.to_le_bytes());
        data.extend_from_slice(&num_indices.to_le_bytes());

        for &p in &self.positions {
            data.extend_from_slice(&p.to_le_bytes());
        }
        for &n in &self.normals {
            data.extend_from_slice(&n.to_le_bytes());
        }
        for &i in &self.indices {
            data.extend_from_slice(&i.to_le_bytes());
        }

        data
    }
}

const SEGMENTS: usize = 64;

// ============================================================================
// Procedural mesh generators (produce clean manifolds)
// ============================================================================

fn procedural_cylinder(radius: f32, height: f32) -> MeshData {
    let mut mesh = MeshData::new();

    // Side wall (base at z=0, top at z=height)
    let mut bottom_ring = Vec::new();
    let mut top_ring = Vec::new();

    for i in 0..SEGMENTS {
        let angle = (i as f32 / SEGMENTS as f32) * 2.0 * PI;
        let x = radius * angle.cos();
        let y = radius * angle.sin();
        let nx = angle.cos();
        let ny = angle.sin();

        bottom_ring.push(mesh.add_vertex([x, y, 0.0], [nx, ny, 0.0]));
        top_ring.push(mesh.add_vertex([x, y, height], [nx, ny, 0.0]));
    }

    for i in 0..SEGMENTS {
        let next = (i + 1) % SEGMENTS;
        mesh.add_triangle(bottom_ring[i], bottom_ring[next], top_ring[next]);
        mesh.add_triangle(bottom_ring[i], top_ring[next], top_ring[i]);
    }

    // Bottom cap (z=0)
    let bottom_center = mesh.add_vertex([0.0, 0.0, 0.0], [0.0, 0.0, -1.0]);
    let mut bottom_cap = Vec::new();
    for i in 0..SEGMENTS {
        let angle = (i as f32 / SEGMENTS as f32) * 2.0 * PI;
        bottom_cap.push(mesh.add_vertex(
            [radius * angle.cos(), radius * angle.sin(), 0.0],
            [0.0, 0.0, -1.0],
        ));
    }
    for i in 0..SEGMENTS {
        mesh.add_triangle(bottom_center, bottom_cap[(i + 1) % SEGMENTS], bottom_cap[i]);
    }

    // Top cap (z=height)
    let top_center = mesh.add_vertex([0.0, 0.0, height], [0.0, 0.0, 1.0]);
    let mut top_cap = Vec::new();
    for i in 0..SEGMENTS {
        let angle = (i as f32 / SEGMENTS as f32) * 2.0 * PI;
        top_cap.push(mesh.add_vertex(
            [radius * angle.cos(), radius * angle.sin(), height],
            [0.0, 0.0, 1.0],
        ));
    }
    for i in 0..SEGMENTS {
        mesh.add_triangle(top_center, top_cap[i], top_cap[(i + 1) % SEGMENTS]);
    }

    mesh
}

fn procedural_tube(outer_r: f32, inner_r: f32, height: f32) -> MeshData {
    let mut mesh = MeshData::new();

    // Outer wall (normals pointing out, base at z=0)
    let mut outer_bottom = Vec::new();
    let mut outer_top = Vec::new();

    for i in 0..SEGMENTS {
        let angle = (i as f32 / SEGMENTS as f32) * 2.0 * PI;
        let x = outer_r * angle.cos();
        let y = outer_r * angle.sin();
        let nx = angle.cos();
        let ny = angle.sin();

        outer_bottom.push(mesh.add_vertex([x, y, 0.0], [nx, ny, 0.0]));
        outer_top.push(mesh.add_vertex([x, y, height], [nx, ny, 0.0]));
    }

    for i in 0..SEGMENTS {
        let next = (i + 1) % SEGMENTS;
        mesh.add_triangle(outer_bottom[i], outer_bottom[next], outer_top[next]);
        mesh.add_triangle(outer_bottom[i], outer_top[next], outer_top[i]);
    }

    // Inner wall (normals pointing in)
    let mut inner_bottom = Vec::new();
    let mut inner_top = Vec::new();

    for i in 0..SEGMENTS {
        let angle = (i as f32 / SEGMENTS as f32) * 2.0 * PI;
        let x = inner_r * angle.cos();
        let y = inner_r * angle.sin();
        let nx = -angle.cos();
        let ny = -angle.sin();

        inner_bottom.push(mesh.add_vertex([x, y, 0.0], [nx, ny, 0.0]));
        inner_top.push(mesh.add_vertex([x, y, height], [nx, ny, 0.0]));
    }

    for i in 0..SEGMENTS {
        let next = (i + 1) % SEGMENTS;
        mesh.add_triangle(inner_bottom[i], inner_top[next], inner_bottom[next]);
        mesh.add_triangle(inner_bottom[i], inner_top[i], inner_top[next]);
    }

    // Bottom annulus (z=0, normal pointing down)
    let mut ob = Vec::new();
    let mut ib = Vec::new();
    for i in 0..SEGMENTS {
        let angle = (i as f32 / SEGMENTS as f32) * 2.0 * PI;
        ob.push(mesh.add_vertex(
            [outer_r * angle.cos(), outer_r * angle.sin(), 0.0],
            [0.0, 0.0, -1.0],
        ));
        ib.push(mesh.add_vertex(
            [inner_r * angle.cos(), inner_r * angle.sin(), 0.0],
            [0.0, 0.0, -1.0],
        ));
    }
    for i in 0..SEGMENTS {
        let next = (i + 1) % SEGMENTS;
        mesh.add_triangle(ob[i], ib[next], ib[i]);
        mesh.add_triangle(ob[i], ob[next], ib[next]);
    }

    // Top annulus (z=height, normal pointing up)
    let mut ot = Vec::new();
    let mut it = Vec::new();
    for i in 0..SEGMENTS {
        let angle = (i as f32 / SEGMENTS as f32) * 2.0 * PI;
        ot.push(mesh.add_vertex(
            [outer_r * angle.cos(), outer_r * angle.sin(), height],
            [0.0, 0.0, 1.0],
        ));
        it.push(mesh.add_vertex(
            [inner_r * angle.cos(), inner_r * angle.sin(), height],
            [0.0, 0.0, 1.0],
        ));
    }
    for i in 0..SEGMENTS {
        let next = (i + 1) % SEGMENTS;
        mesh.add_triangle(ot[i], it[i], it[next]);
        mesh.add_triangle(ot[i], it[next], ot[next]);
    }

    mesh
}

fn procedural_box(w: f32, d: f32, h: f32) -> MeshData {
    let mut mesh = MeshData::new();
    let hw = w / 2.0;
    let hd = d / 2.0;
    let hh = h / 2.0;

    let faces: [([f32; 3], [[f32; 3]; 4]); 6] = [
        ([1.0, 0.0, 0.0], [[hw, -hd, -hh], [hw, hd, -hh], [hw, hd, hh], [hw, -hd, hh]]),
        ([-1.0, 0.0, 0.0], [[-hw, hd, -hh], [-hw, -hd, -hh], [-hw, -hd, hh], [-hw, hd, hh]]),
        ([0.0, 1.0, 0.0], [[hw, hd, -hh], [-hw, hd, -hh], [-hw, hd, hh], [hw, hd, hh]]),
        ([0.0, -1.0, 0.0], [[-hw, -hd, -hh], [hw, -hd, -hh], [hw, -hd, hh], [-hw, -hd, hh]]),
        ([0.0, 0.0, 1.0], [[-hw, -hd, hh], [hw, -hd, hh], [hw, hd, hh], [-hw, hd, hh]]),
        ([0.0, 0.0, -1.0], [[hw, -hd, -hh], [-hw, -hd, -hh], [-hw, hd, -hh], [hw, hd, -hh]]),
    ];

    for (normal, verts) in faces {
        let v0 = mesh.add_vertex(verts[0], normal);
        let v1 = mesh.add_vertex(verts[1], normal);
        let v2 = mesh.add_vertex(verts[2], normal);
        let v3 = mesh.add_vertex(verts[3], normal);
        mesh.add_triangle(v0, v1, v2);
        mesh.add_triangle(v0, v2, v3);
    }

    mesh
}

fn procedural_sphere(radius: f32) -> MeshData {
    let mut mesh = MeshData::new();
    let stacks = 32usize;
    let slices = 64usize;

    let mut verts: Vec<Vec<u32>> = Vec::new();

    for i in 0..=stacks {
        let phi = PI * (i as f32 / stacks as f32);
        let z = radius * phi.cos();
        let r = radius * phi.sin();

        let mut ring = Vec::new();
        for j in 0..=slices {
            let theta = 2.0 * PI * (j as f32 / slices as f32);
            let x = r * theta.cos();
            let y = r * theta.sin();
            let nx = phi.sin() * theta.cos();
            let ny = phi.sin() * theta.sin();
            let nz = phi.cos();
            ring.push(mesh.add_vertex([x, y, z], [nx, ny, nz]));
        }
        verts.push(ring);
    }

    for i in 0..stacks {
        for j in 0..slices {
            let (v0, v1, v2, v3) = (verts[i][j], verts[i][j + 1], verts[i + 1][j + 1], verts[i + 1][j]);
            if i != 0 {
                mesh.add_triangle(v0, v1, v2);
            }
            if i != stacks - 1 {
                mesh.add_triangle(v0, v2, v3);
            }
        }
    }

    mesh
}

// ============================================================================
// Transform application
// ============================================================================

fn apply_transform_to_mesh(mesh: &mut MeshData, position: [f32; 3], rotation: [f32; 3], scale: [f32; 3]) {
    // Apply scale
    if scale != [1.0, 1.0, 1.0] {
        for i in 0..mesh.positions.len() / 3 {
            mesh.positions[i * 3] *= scale[0];
            mesh.positions[i * 3 + 1] *= scale[1];
            mesh.positions[i * 3 + 2] *= scale[2];
        }
    }

    // Apply rotation (Euler XYZ in degrees)
    if rotation != [0.0, 0.0, 0.0] {
        let rx = rotation[0].to_radians();
        let ry = rotation[1].to_radians();
        let rz = rotation[2].to_radians();

        let (sx, cx) = (rx.sin(), rx.cos());
        let (sy, cy) = (ry.sin(), ry.cos());
        let (sz, cz) = (rz.sin(), rz.cos());

        // Combined rotation matrix (ZYX order)
        let m00 = cy * cz;
        let m01 = sx * sy * cz - cx * sz;
        let m02 = cx * sy * cz + sx * sz;
        let m10 = cy * sz;
        let m11 = sx * sy * sz + cx * cz;
        let m12 = cx * sy * sz - sx * cz;
        let m20 = -sy;
        let m21 = sx * cy;
        let m22 = cx * cy;

        for i in 0..mesh.positions.len() / 3 {
            let x = mesh.positions[i * 3];
            let y = mesh.positions[i * 3 + 1];
            let z = mesh.positions[i * 3 + 2];

            mesh.positions[i * 3] = m00 * x + m01 * y + m02 * z;
            mesh.positions[i * 3 + 1] = m10 * x + m11 * y + m12 * z;
            mesh.positions[i * 3 + 2] = m20 * x + m21 * y + m22 * z;

            // Rotate normals too
            let nx = mesh.normals[i * 3];
            let ny = mesh.normals[i * 3 + 1];
            let nz = mesh.normals[i * 3 + 2];

            mesh.normals[i * 3] = m00 * nx + m01 * ny + m02 * nz;
            mesh.normals[i * 3 + 1] = m10 * nx + m11 * ny + m12 * nz;
            mesh.normals[i * 3 + 2] = m20 * nx + m21 * ny + m22 * nz;
        }
    }

    // Apply translation
    if position != [0.0, 0.0, 0.0] {
        for i in 0..mesh.positions.len() / 3 {
            mesh.positions[i * 3] += position[0];
            mesh.positions[i * 3 + 1] += position[1];
            mesh.positions[i * 3 + 2] += position[2];
        }
    }
}

// ============================================================================
// Lua scene parsing
// ============================================================================

fn get_transform(table: &mlua::Table) -> ([f32; 3], [f32; 3], [f32; 3]) {
    let mut position = [0.0f32, 0.0, 0.0];
    let mut rotation = [0.0f32, 0.0, 0.0];
    let mut scale = [1.0f32, 1.0, 1.0];

    if let Ok(transform) = table.get::<_, mlua::Table>("transform") {
        if let Ok(pos) = transform.get::<_, mlua::Table>("position") {
            position[0] = pos.get::<_, f32>(1).unwrap_or(0.0);
            position[1] = pos.get::<_, f32>(2).unwrap_or(0.0);
            position[2] = pos.get::<_, f32>(3).unwrap_or(0.0);
        }
        if let Ok(rot) = transform.get::<_, mlua::Table>("rotation") {
            rotation[0] = rot.get::<_, f32>(1).unwrap_or(0.0);
            rotation[1] = rot.get::<_, f32>(2).unwrap_or(0.0);
            rotation[2] = rot.get::<_, f32>(3).unwrap_or(0.0);
        }
        if let Ok(sc) = transform.get::<_, mlua::Table>("scale") {
            scale[0] = sc.get::<_, f32>(1).unwrap_or(1.0);
            scale[1] = sc.get::<_, f32>(2).unwrap_or(1.0);
            scale[2] = sc.get::<_, f32>(3).unwrap_or(1.0);
        }
    }

    (position, rotation, scale)
}

/// Check if this is a cylinder-cylinder difference (tube) and handle it specially
fn try_build_tube(table: &mlua::Table) -> Option<MeshData> {
    let obj_type: String = table.get("type").ok()?;
    if obj_type != "csg" {
        return None;
    }

    let operation: String = table.get("operation").ok()?;
    if operation != "difference" {
        return None;
    }

    let children: mlua::Table = table.get("children").ok()?;
    if children.len().ok()? != 2 {
        return None;
    }

    let child1: mlua::Table = children.get(1).ok()?;
    let child2: mlua::Table = children.get(2).ok()?;

    let type1: String = child1.get("type").ok()?;
    let type2: String = child2.get("type").ok()?;

    if type1 != "cylinder" || type2 != "cylinder" {
        return None;
    }

    let params1: mlua::Table = child1.get("params").ok()?;
    let params2: mlua::Table = child2.get("params").ok()?;

    let outer_r: f32 = params1.get("r").ok()?;
    let outer_h: f32 = params1.get("h").ok()?;
    let inner_r: f32 = params2.get("r").ok()?;

    // Use outer height for the tube
    let mut mesh = procedural_tube(outer_r, inner_r, outer_h);

    // Apply transform from the CSG node
    let (position, rotation, scale) = get_transform(table);
    apply_transform_to_mesh(&mut mesh, position, rotation, scale);

    Some(mesh)
}

/// Build mesh from a serialized object
fn build_object(table: &mlua::Table) -> Result<MeshData> {
    let obj_type: String = table.get("type")?;

    // Try special case: cylinder-cylinder difference = tube
    if let Some(tube) = try_build_tube(table) {
        return Ok(tube);
    }

    if obj_type == "csg" {
        // Generic CSG - use csgrs (may have artifacts)
        let operation: String = table.get("operation")?;
        let children: mlua::Table = table.get("children")?;

        let first_child: mlua::Table = children.get(1)?;
        let mut result = build_csg_mesh(&first_child)?;

        for i in 2..=children.len()? {
            let child: mlua::Table = children.get(i)?;
            let child_mesh = build_csg_mesh(&child)?;

            result = match operation.as_str() {
                "union" => result.union(&child_mesh),
                "difference" => result.difference(&child_mesh),
                "intersect" => result.intersection(&child_mesh),
                _ => return Err(anyhow!("Unknown CSG operation: {}", operation)),
            };
        }

        let mut mesh = csg_mesh_to_data(&result);
        let (position, rotation, scale) = get_transform(table);
        apply_transform_to_mesh(&mut mesh, position, rotation, scale);
        Ok(mesh)
    } else if obj_type == "group" {
        // Group - merge all children
        let children: mlua::Table = table.get("children")?;
        let mut combined = MeshData::new();

        for pair in children.pairs::<i64, mlua::Table>() {
            let (_, child) = pair?;
            let child_mesh = build_object(&child)?;
            combined.merge(&child_mesh);
        }

        let (position, rotation, scale) = get_transform(table);
        apply_transform_to_mesh(&mut combined, position, rotation, scale);
        Ok(combined)
    } else {
        // Primitive - use procedural generation
        let params: mlua::Table = table.get("params")?;
        let mut mesh = build_primitive(&obj_type, &params)?;
        let (position, rotation, scale) = get_transform(table);
        apply_transform_to_mesh(&mut mesh, position, rotation, scale);
        Ok(mesh)
    }
}

fn build_primitive(obj_type: &str, params: &mlua::Table) -> Result<MeshData> {
    match obj_type {
        "cylinder" => {
            let r: f32 = params.get("r")?;
            let h: f32 = params.get("h")?;
            Ok(procedural_cylinder(r, h))
        }
        "box" => {
            let w: f32 = params.get("w")?;
            let d: f32 = params.get::<_, f32>("d").unwrap_or(w);
            let h: f32 = params.get("h")?;
            Ok(procedural_box(w, d, h))
        }
        "sphere" => {
            let r: f32 = params.get("r")?;
            Ok(procedural_sphere(r))
        }
        "cube" => {
            let size: f32 = params.get("size").unwrap_or(1.0);
            Ok(procedural_box(size, size, size))
        }
        _ => Err(anyhow!("Unknown primitive type: {}", obj_type)),
    }
}

// ============================================================================
// CSG mesh conversion (fallback using csgrs)
// ============================================================================

fn build_csg_mesh(table: &mlua::Table) -> Result<CsgMesh<()>> {
    let obj_type: String = table.get("type")?;

    match obj_type.as_str() {
        "cylinder" => {
            let params: mlua::Table = table.get("params")?;
            let r: f64 = params.get("r")?;
            let h: f64 = params.get("h")?;
            Ok(CsgMesh::cylinder(r, h, SEGMENTS, None))
        }
        "box" => {
            let params: mlua::Table = table.get("params")?;
            let w: f64 = params.get("w")?;
            let d: f64 = params.get::<_, f64>("d").unwrap_or(w);
            let h: f64 = params.get("h")?;
            Ok(CsgMesh::cuboid(w, d, h, None))
        }
        "sphere" => {
            let params: mlua::Table = table.get("params")?;
            let r: f64 = params.get("r")?;
            Ok(CsgMesh::sphere(r, SEGMENTS, SEGMENTS / 2, None))
        }
        _ => Err(anyhow!("Unsupported CSG primitive: {}", obj_type)),
    }
}

fn csg_mesh_to_data(mesh: &CsgMesh<()>) -> MeshData {
    let triangulated = mesh.triangulate();
    let mut data = MeshData::new();
    let mut vertex_map: std::collections::HashMap<String, u32> = std::collections::HashMap::new();

    for poly in &triangulated.polygons {
        let verts = &poly.vertices;
        if verts.len() != 3 {
            continue;
        }

        let mut tri_indices = [0u32; 3];

        for (i, v) in verts.iter().enumerate() {
            let pos = &v.pos;
            let normal = &v.normal;

            let key = format!(
                "{:.6},{:.6},{:.6},{:.6},{:.6},{:.6}",
                pos.x, pos.y, pos.z, normal.x, normal.y, normal.z
            );

            let idx = if let Some(&existing) = vertex_map.get(&key) {
                existing
            } else {
                let idx = (data.positions.len() / 3) as u32;
                data.positions.push(pos.x as f32);
                data.positions.push(pos.y as f32);
                data.positions.push(pos.z as f32);
                data.normals.push(normal.x as f32);
                data.normals.push(normal.y as f32);
                data.normals.push(normal.z as f32);
                vertex_map.insert(key, idx);
                idx
            };

            tri_indices[i] = idx;
        }

        data.indices.push(tri_indices[0]);
        data.indices.push(tri_indices[1]);
        data.indices.push(tri_indices[2]);
    }

    data
}

/// Generate mesh from Lua scene
pub fn generate_mesh_from_lua(_lua: &Lua, value: &Value) -> Result<MeshData> {
    let table = value.as_table().ok_or_else(|| anyhow!("Expected table"))?;
    let objects: mlua::Table = table.get("objects")?;

    let mut combined = MeshData::new();

    for pair in objects.pairs::<i64, mlua::Table>() {
        let (_, obj) = pair?;
        let mesh = build_object(&obj)?;
        combined.merge(&mesh);
    }

    Ok(combined)
}
