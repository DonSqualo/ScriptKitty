//! STL Export - Binary STL file generation for 3D printing
//! Units: millimeters, manifold meshes

use crate::geometry::{self, MeshData};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use tracing::info;

/// Write binary STL file from mesh data
pub fn write_stl(mesh: &MeshData, path: &Path) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    // 80-byte header (padded with zeros)
    let mut header = [0u8; 80];
    let text = b"ScriptCAD STL - units: mm";
    header[..text.len()].copy_from_slice(text);
    writer.write_all(&header)?;

    // Triangle count
    let num_triangles = (mesh.indices.len() / 3) as u32;
    writer.write_all(&num_triangles.to_le_bytes())?;

    // Write each triangle
    for tri in 0..(mesh.indices.len() / 3) {
        let i0 = mesh.indices[tri * 3] as usize;
        let i1 = mesh.indices[tri * 3 + 1] as usize;
        let i2 = mesh.indices[tri * 3 + 2] as usize;

        // Get vertices
        let v0 = [
            mesh.positions[i0 * 3],
            mesh.positions[i0 * 3 + 1],
            mesh.positions[i0 * 3 + 2],
        ];
        let v1 = [
            mesh.positions[i1 * 3],
            mesh.positions[i1 * 3 + 1],
            mesh.positions[i1 * 3 + 2],
        ];
        let v2 = [
            mesh.positions[i2 * 3],
            mesh.positions[i2 * 3 + 1],
            mesh.positions[i2 * 3 + 2],
        ];

        // Compute face normal from vertices (right-hand rule)
        let edge1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let edge2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
        let normal = cross(edge1, edge2);
        let normal = normalize(normal);

        // Write normal
        for n in normal {
            writer.write_all(&n.to_le_bytes())?;
        }

        // Write vertices
        for v in [v0, v1, v2] {
            for coord in v {
                writer.write_all(&coord.to_le_bytes())?;
            }
        }

        // Attribute byte count (unused)
        writer.write_all(&0u16.to_le_bytes())?;
    }

    writer.flush()?;
    info!("Exported STL: {} triangles to {:?}", num_triangles, path);
    Ok(())
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len > 1e-10 {
        [v[0] / len, v[1] / len, v[2] / len]
    } else {
        [0.0, 0.0, 1.0]
    }
}

/// Process exports directly from the result table
pub fn process_exports_from_table(lua: &mlua::Lua, table: &mlua::Table, base_dir: &Path) {
    let exports = match table.get::<_, mlua::Table>("exports") {
        Ok(e) => e,
        Err(_) => return,
    };

    for pair in exports.pairs::<i32, mlua::Table>() {
        if let Ok((_, exp)) = pair {
            let format: String = exp.get("format").unwrap_or_default();
            let filename: String = exp.get("filename").unwrap_or_default();

            if format != "stl" || filename.is_empty() {
                continue;
            }

            let object: mlua::Table = match exp.get("object") {
                Ok(o) => o,
                Err(_) => continue,
            };

            let path = base_dir.join(&filename);
            match geometry::generate_mesh_from_object(lua, &object) {
                Ok(mesh) => {
                    if let Err(e) = write_stl(&mesh, &path) {
                        tracing::error!("STL export failed for {}: {}", filename, e);
                    }
                }
                Err(e) => {
                    tracing::error!("Mesh generation failed for {}: {}", filename, e);
                }
            }
        }
    }
}
