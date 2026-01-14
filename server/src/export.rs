//! STL Export - Binary STL file generation for 3D printing
//! Units: millimeters, manifold meshes

use crate::geometry::MeshData;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use tracing::info;

/// Write binary STL file from mesh data
pub fn write_stl(mesh: &MeshData, path: &Path) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    // 80-byte header
    let header = format!("ScriptCAD STL - units: mm{}", " ".repeat(80 - 27));
    writer.write_all(&header.as_bytes()[..80])?;

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

/// Process export queue from Lua scene
pub fn process_exports(exports: &[ExportRequest], mesh: &MeshData, base_dir: &Path) {
    for export in exports {
        if export.format == "stl" {
            let path = base_dir.join(&export.filename);
            if let Err(e) = write_stl(mesh, &path) {
                tracing::error!("STL export failed: {}", e);
            }
        }
    }
}

#[derive(Debug)]
pub struct ExportRequest {
    pub format: String,
    pub filename: String,
}

pub fn parse_exports(table: &mlua::Table) -> Vec<ExportRequest> {
    let mut requests = Vec::new();

    if let Ok(exports) = table.get::<_, mlua::Table>("exports") {
        for pair in exports.pairs::<i32, mlua::Table>() {
            if let Ok((_, exp)) = pair {
                let format: String = exp.get("format").unwrap_or_default();
                let filename: String = exp.get("filename").unwrap_or_default();
                if !format.is_empty() && !filename.is_empty() {
                    requests.push(ExportRequest { format, filename });
                }
            }
        }
    }

    requests
}
