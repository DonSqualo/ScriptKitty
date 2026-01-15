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
    pub colors: Vec<f32>,
    pub indices: Vec<u32>,
}

impl MeshData {
    fn new() -> Self {
        MeshData {
            positions: Vec::new(),
            normals: Vec::new(),
            colors: Vec::new(),
            indices: Vec::new(),
        }
    }

    fn add_vertex(&mut self, pos: [f32; 3], normal: [f32; 3]) -> u32 {
        let idx = (self.positions.len() / 3) as u32;
        self.positions.extend_from_slice(&pos);
        self.normals.extend_from_slice(&normal);
        self.colors.extend_from_slice(&[1.0, 1.0, 1.0]); // default white
        idx
    }

    fn add_triangle(&mut self, a: u32, b: u32, c: u32) {
        self.indices.extend_from_slice(&[a, b, c]);
    }

    fn set_color(&mut self, r: f32, g: f32, b: f32) {
        for i in 0..self.colors.len() / 3 {
            self.colors[i * 3] = r;
            self.colors[i * 3 + 1] = g;
            self.colors[i * 3 + 2] = b;
        }
    }

    fn merge(&mut self, other: &MeshData) {
        let offset = (self.positions.len() / 3) as u32;
        self.positions.extend_from_slice(&other.positions);
        self.normals.extend_from_slice(&other.normals);
        self.colors.extend_from_slice(&other.colors);
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
        for &c in &self.colors {
            data.extend_from_slice(&c.to_le_bytes());
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

    // Box with corner at origin, extends to (w, d, h)
    let faces: [([f32; 3], [[f32; 3]; 4]); 6] = [
        ([1.0, 0.0, 0.0], [[w, 0.0, 0.0], [w, d, 0.0], [w, d, h], [w, 0.0, h]]),
        ([-1.0, 0.0, 0.0], [[0.0, d, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, h], [0.0, d, h]]),
        ([0.0, 1.0, 0.0], [[w, d, 0.0], [0.0, d, 0.0], [0.0, d, h], [w, d, h]]),
        ([0.0, -1.0, 0.0], [[0.0, 0.0, 0.0], [w, 0.0, 0.0], [w, 0.0, h], [0.0, 0.0, h]]),
        ([0.0, 0.0, 1.0], [[0.0, 0.0, h], [w, 0.0, h], [w, d, h], [0.0, d, h]]),
        ([0.0, 0.0, -1.0], [[w, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, d, 0.0], [w, d, 0.0]]),
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

// Box with centered cylindrical hole (centered XY, base at z=0)
fn procedural_box_with_hole(w: f32, d: f32, h: f32, hole_r: f32) -> MeshData {
    let mut mesh = MeshData::new();
    let cx = w / 2.0;
    let cy = d / 2.0;

    // 4 outer side walls
    let faces: [([f32; 3], [[f32; 3]; 4]); 4] = [
        ([1.0, 0.0, 0.0], [[w, 0.0, 0.0], [w, d, 0.0], [w, d, h], [w, 0.0, h]]),
        ([-1.0, 0.0, 0.0], [[0.0, d, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, h], [0.0, d, h]]),
        ([0.0, 1.0, 0.0], [[w, d, 0.0], [0.0, d, 0.0], [0.0, d, h], [w, d, h]]),
        ([0.0, -1.0, 0.0], [[0.0, 0.0, 0.0], [w, 0.0, 0.0], [w, 0.0, h], [0.0, 0.0, h]]),
    ];
    for (normal, verts) in faces {
        let v0 = mesh.add_vertex(verts[0], normal);
        let v1 = mesh.add_vertex(verts[1], normal);
        let v2 = mesh.add_vertex(verts[2], normal);
        let v3 = mesh.add_vertex(verts[3], normal);
        mesh.add_triangle(v0, v1, v2);
        mesh.add_triangle(v0, v2, v3);
    }

    // Inner cylindrical surface (normals pointing inward)
    let mut inner_bottom = Vec::new();
    let mut inner_top = Vec::new();
    for i in 0..SEGMENTS {
        let angle = (i as f32 / SEGMENTS as f32) * 2.0 * PI;
        let x = cx + hole_r * angle.cos();
        let y = cy + hole_r * angle.sin();
        let nx = -angle.cos();
        let ny = -angle.sin();
        inner_bottom.push(mesh.add_vertex([x, y, 0.0], [nx, ny, 0.0]));
        inner_top.push(mesh.add_vertex([x, y, h], [nx, ny, 0.0]));
    }
    for i in 0..SEGMENTS {
        let next = (i + 1) % SEGMENTS;
        mesh.add_triangle(inner_bottom[i], inner_top[next], inner_bottom[next]);
        mesh.add_triangle(inner_bottom[i], inner_top[i], inner_top[next]);
    }

    // Top face with hole (z=h, triangulate from box corners to circle)
    let corners_top = [
        mesh.add_vertex([0.0, 0.0, h], [0.0, 0.0, 1.0]),
        mesh.add_vertex([w, 0.0, h], [0.0, 0.0, 1.0]),
        mesh.add_vertex([w, d, h], [0.0, 0.0, 1.0]),
        mesh.add_vertex([0.0, d, h], [0.0, 0.0, 1.0]),
    ];
    let mut circle_top = Vec::new();
    for i in 0..SEGMENTS {
        let angle = (i as f32 / SEGMENTS as f32) * 2.0 * PI;
        circle_top.push(mesh.add_vertex(
            [cx + hole_r * angle.cos(), cy + hole_r * angle.sin(), h],
            [0.0, 0.0, 1.0],
        ));
    }
    // Fan triangles from each corner to nearby circle segments
    for i in 0..SEGMENTS {
        let next = (i + 1) % SEGMENTS;
        let angle = (i as f32 + 0.5) / SEGMENTS as f32 * 2.0 * PI;
        let corner_idx = if angle < PI / 2.0 { 1 } else if angle < PI { 2 } else if angle < 3.0 * PI / 2.0 { 3 } else { 0 };
        mesh.add_triangle(corners_top[corner_idx], circle_top[i], circle_top[next]);
    }
    // Fill corner triangles
    mesh.add_triangle(corners_top[0], corners_top[1], circle_top[0]);
    mesh.add_triangle(corners_top[1], corners_top[2], circle_top[SEGMENTS / 4]);
    mesh.add_triangle(corners_top[2], corners_top[3], circle_top[SEGMENTS / 2]);
    mesh.add_triangle(corners_top[3], corners_top[0], circle_top[3 * SEGMENTS / 4]);

    // Bottom face with hole (z=0)
    let corners_bot = [
        mesh.add_vertex([0.0, 0.0, 0.0], [0.0, 0.0, -1.0]),
        mesh.add_vertex([w, 0.0, 0.0], [0.0, 0.0, -1.0]),
        mesh.add_vertex([w, d, 0.0], [0.0, 0.0, -1.0]),
        mesh.add_vertex([0.0, d, 0.0], [0.0, 0.0, -1.0]),
    ];
    let mut circle_bot = Vec::new();
    for i in 0..SEGMENTS {
        let angle = (i as f32 / SEGMENTS as f32) * 2.0 * PI;
        circle_bot.push(mesh.add_vertex(
            [cx + hole_r * angle.cos(), cy + hole_r * angle.sin(), 0.0],
            [0.0, 0.0, -1.0],
        ));
    }
    for i in 0..SEGMENTS {
        let next = (i + 1) % SEGMENTS;
        let angle = (i as f32 + 0.5) / SEGMENTS as f32 * 2.0 * PI;
        let corner_idx = if angle < PI / 2.0 { 1 } else if angle < PI { 2 } else if angle < 3.0 * PI / 2.0 { 3 } else { 0 };
        mesh.add_triangle(corners_bot[corner_idx], circle_bot[next], circle_bot[i]);
    }
    mesh.add_triangle(corners_bot[0], circle_bot[0], corners_bot[1]);
    mesh.add_triangle(corners_bot[1], circle_bot[SEGMENTS / 4], corners_bot[2]);
    mesh.add_triangle(corners_bot[2], circle_bot[SEGMENTS / 2], corners_bot[3]);
    mesh.add_triangle(corners_bot[3], circle_bot[3 * SEGMENTS / 4], corners_bot[0]);

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
// Transform application - operations applied in order
// ============================================================================

fn apply_translate(mesh: &mut MeshData, x: f32, y: f32, z: f32) {
    for i in 0..mesh.positions.len() / 3 {
        mesh.positions[i * 3] += x;
        mesh.positions[i * 3 + 1] += y;
        mesh.positions[i * 3 + 2] += z;
    }
}

fn apply_rotate(mesh: &mut MeshData, rx: f32, ry: f32, rz: f32) {
    let rx = rx.to_radians();
    let ry = ry.to_radians();
    let rz = rz.to_radians();

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

        let nx = mesh.normals[i * 3];
        let ny = mesh.normals[i * 3 + 1];
        let nz = mesh.normals[i * 3 + 2];

        mesh.normals[i * 3] = m00 * nx + m01 * ny + m02 * nz;
        mesh.normals[i * 3 + 1] = m10 * nx + m11 * ny + m12 * nz;
        mesh.normals[i * 3 + 2] = m20 * nx + m21 * ny + m22 * nz;
    }
}

fn apply_scale(mesh: &mut MeshData, sx: f32, sy: f32, sz: f32) {
    for i in 0..mesh.positions.len() / 3 {
        mesh.positions[i * 3] *= sx;
        mesh.positions[i * 3 + 1] *= sy;
        mesh.positions[i * 3 + 2] *= sz;
    }
}

fn apply_ops(mesh: &mut MeshData, table: &mlua::Table) {
    if let Ok(ops) = table.get::<_, mlua::Table>("ops") {
        for pair in ops.pairs::<i64, mlua::Table>() {
            if let Ok((_, op_table)) = pair {
                let op: String = op_table.get("op").unwrap_or_default();
                let x: f32 = op_table.get("x").unwrap_or(0.0);
                let y: f32 = op_table.get("y").unwrap_or(0.0);
                let z: f32 = op_table.get("z").unwrap_or(0.0);

                match op.as_str() {
                    "translate" => apply_translate(mesh, x, y, z),
                    "rotate" => apply_rotate(mesh, x, y, z),
                    "scale" => apply_scale(mesh, x, y, z),
                    _ => {}
                }
            }
        }
    }
}

// ============================================================================
// Lua scene parsing
// ============================================================================

fn apply_material_color(mesh: &mut MeshData, table: &mlua::Table) {
    if let Ok(material) = table.get::<_, mlua::Table>("material") {
        if let Ok(color) = material.get::<_, mlua::Table>("color") {
            let r: f32 = color.get(1).unwrap_or(1.0);
            let g: f32 = color.get(2).unwrap_or(1.0);
            let b: f32 = color.get(3).unwrap_or(1.0);
            mesh.set_color(r, g, b);
        }
    }
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
    let inner_h: f32 = params2.get("h").ok()?;

    // Only use tube optimization if inner cylinder goes all the way through
    // (i.e., it's actually a tube, not a cup/lid shape)
    if inner_h < outer_h {
        return None;
    }

    let mut mesh = procedural_tube(outer_r, inner_r, outer_h);

    // Apply ops and material from the CSG node
    apply_ops(&mut mesh, table);
    apply_material_color(&mut mesh, table);

    Some(mesh)
}

/// Check if this is a box-cylinder difference (box with hole) and handle it specially
fn try_build_box_with_hole(table: &mlua::Table) -> Option<MeshData> {
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

    if type1 != "box" || type2 != "cylinder" {
        return None;
    }

    let params1: mlua::Table = child1.get("params").ok()?;
    let params2: mlua::Table = child2.get("params").ok()?;

    let w: f32 = params1.get("w").ok()?;
    let d: f32 = params1.get::<_, f32>("d").unwrap_or(w);
    let h: f32 = params1.get("h").ok()?;
    let hole_r: f32 = params2.get("r").ok()?;
    let hole_h: f32 = params2.get("h").ok()?;

    // Only use if hole goes all the way through
    if hole_h < h {
        return None;
    }

    // Check if box is centered (has centerXY ops)
    let box_ops: mlua::Table = child1.get("ops").ok()?;
    let mut is_centered = false;
    for pair in box_ops.pairs::<i64, mlua::Table>() {
        if let Ok((_, op)) = pair {
            let op_name: String = op.get("op").unwrap_or_default();
            if op_name == "translate" {
                let tx: f32 = op.get("x").unwrap_or(0.0);
                let ty: f32 = op.get("y").unwrap_or(0.0);
                // Check if centered (translated by -w/2, -d/2)
                if (tx + w / 2.0).abs() < 0.01 && (ty + d / 2.0).abs() < 0.01 {
                    is_centered = true;
                }
            }
        }
    }

    if !is_centered {
        return None;
    }

    let mut mesh = procedural_box_with_hole(w, d, h, hole_r);

    // Shift back to centered position
    apply_translate(&mut mesh, -w / 2.0, -d / 2.0, 0.0);

    // Apply ops from the CSG node
    apply_ops(&mut mesh, table);
    apply_material_color(&mut mesh, table);

    Some(mesh)
}

/// Build mesh from a serialized object
fn build_object(table: &mlua::Table) -> Result<MeshData> {
    let obj_type: String = table.get("type")?;

    // Try special case: box-cylinder difference = box with hole
    if let Some(box_hole) = try_build_box_with_hole(table) {
        return Ok(box_hole);
    }

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
        apply_ops(&mut mesh, table);
        apply_material_color(&mut mesh, table);
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

        apply_ops(&mut combined, table);
        apply_material_color(&mut combined, table);
        Ok(combined)
    } else {
        // Primitive - use procedural generation
        let params: mlua::Table = table.get("params")?;
        let mut mesh = build_primitive(&obj_type, &params)?;
        apply_ops(&mut mesh, table);
        apply_material_color(&mut mesh, table);
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

    let mut mesh = match obj_type.as_str() {
        "cylinder" => {
            let params: mlua::Table = table.get("params")?;
            let r: f64 = params.get("r")?;
            let h: f64 = params.get("h")?;
            CsgMesh::cylinder(r, h, SEGMENTS, None)
        }
        "box" => {
            let params: mlua::Table = table.get("params")?;
            let w: f64 = params.get("w")?;
            let d: f64 = params.get::<_, f64>("d").unwrap_or(w);
            let h: f64 = params.get("h")?;
            CsgMesh::cuboid(w, d, h, None)
        }
        "sphere" => {
            let params: mlua::Table = table.get("params")?;
            let r: f64 = params.get("r")?;
            CsgMesh::sphere(r, SEGMENTS, SEGMENTS / 2, None)
        }
        _ => return Err(anyhow!("Unsupported CSG primitive: {}", obj_type)),
    };

    // Apply ops in order
    if let Ok(ops) = table.get::<_, mlua::Table>("ops") {
        for pair in ops.pairs::<i64, mlua::Table>() {
            if let Ok((_, op_table)) = pair {
                let op: String = op_table.get("op").unwrap_or_default();
                let x: f64 = op_table.get("x").unwrap_or(0.0);
                let y: f64 = op_table.get("y").unwrap_or(0.0);
                let z: f64 = op_table.get("z").unwrap_or(0.0);

                match op.as_str() {
                    "translate" => {
                        mesh = mesh.translate(x, y, z);
                    }
                    "rotate" => {
                        mesh = mesh.rotate(x, y, z);
                    }
                    "scale" => {
                        mesh = mesh.scale(x, y, z);
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(mesh)
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
                data.colors.push(1.0);
                data.colors.push(1.0);
                data.colors.push(1.0);
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

/// Generate mesh from Lua scene (all objects combined)
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

/// Generate mesh from a single serialized object
pub fn generate_mesh_from_object(_lua: &Lua, table: &mlua::Table) -> Result<MeshData> {
    build_object(table)
}
