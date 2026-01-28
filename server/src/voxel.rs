//! Voxelizer for FDTD simulation
//!
//! Converts triangle meshes to 3D material grids for MEEP/FDTD.
//! Uses ray casting to determine inside/outside for watertight meshes.

use std::collections::HashMap;

/// Material properties for FDTD
#[derive(Debug, Clone)]
pub struct VoxelMaterial {
    pub id: u8,
    pub name: String,
    pub permittivity: f64,
    pub permeability: f64,
    pub conductivity: f64,
    pub is_pec: bool,
}

impl VoxelMaterial {
    pub fn air() -> Self {
        Self {
            id: 0,
            name: "air".to_string(),
            permittivity: 1.0,
            permeability: 1.0,
            conductivity: 0.0,
            is_pec: false,
        }
    }

    pub fn pec() -> Self {
        Self {
            id: 1,
            name: "pec".to_string(),
            permittivity: 1.0,
            permeability: 1.0,
            conductivity: f64::INFINITY,
            is_pec: true,
        }
    }

    /// Generate MEEP Python expression
    pub fn to_meep(&self) -> String {
        if self.is_pec || self.conductivity > 1e6 {
            "mp.metal".to_string()
        } else if self.permittivity == 1.0 && self.conductivity == 0.0 {
            "mp.air".to_string()
        } else {
            let mut args = vec![];
            if self.permittivity != 1.0 {
                args.push(format!("epsilon={:.6}", self.permittivity));
            }
            if self.permeability != 1.0 {
                args.push(format!("mu={:.6}", self.permeability));
            }
            if self.conductivity > 0.0 && self.conductivity < 1e6 {
                args.push(format!("D_conductivity={:.6e}", self.conductivity));
            }
            if args.is_empty() {
                "mp.air".to_string()
            } else {
                format!("mp.Medium({})", args.join(", "))
            }
        }
    }
}

/// A mesh object with material
#[derive(Debug, Clone)]
pub struct MeshObject {
    pub name: String,
    pub vertices: Vec<[f32; 3]>,
    pub triangles: Vec<[u32; 3]>,
    pub material: VoxelMaterial,
}

/// 3D voxel grid with material IDs
#[derive(Debug, Clone)]
pub struct VoxelGrid {
    /// Grid dimensions
    pub nx: usize,
    pub ny: usize,
    pub nz: usize,
    /// Physical bounds (min corner)
    pub origin: [f64; 3],
    /// Voxel size
    pub voxel_size: f64,
    /// Material ID for each voxel (flattened: z * ny * nx + y * nx + x)
    pub data: Vec<u8>,
    /// Material palette
    pub materials: Vec<VoxelMaterial>,
}

impl VoxelGrid {
    /// Create empty grid filled with air
    pub fn new(
        origin: [f64; 3],
        size: [f64; 3],
        voxel_size: f64,
    ) -> Self {
        let nx = (size[0] / voxel_size).ceil() as usize;
        let ny = (size[1] / voxel_size).ceil() as usize;
        let nz = (size[2] / voxel_size).ceil() as usize;

        let total = nx * ny * nz;

        Self {
            nx,
            ny,
            nz,
            origin,
            voxel_size,
            data: vec![0; total], // 0 = air
            materials: vec![VoxelMaterial::air()],
        }
    }

    /// Get voxel index from coordinates
    #[inline]
    pub fn index(&self, x: usize, y: usize, z: usize) -> usize {
        z * self.ny * self.nx + y * self.nx + x
    }

    /// Get voxel center position
    #[inline]
    pub fn voxel_center(&self, x: usize, y: usize, z: usize) -> [f64; 3] {
        [
            self.origin[0] + (x as f64 + 0.5) * self.voxel_size,
            self.origin[1] + (y as f64 + 0.5) * self.voxel_size,
            self.origin[2] + (z as f64 + 0.5) * self.voxel_size,
        ]
    }

    /// Set material at voxel
    pub fn set(&mut self, x: usize, y: usize, z: usize, material_id: u8) {
        let idx = self.index(x, y, z);
        if idx < self.data.len() {
            self.data[idx] = material_id;
        }
    }

    /// Get material at voxel
    pub fn get(&self, x: usize, y: usize, z: usize) -> u8 {
        let idx = self.index(x, y, z);
        if idx < self.data.len() {
            self.data[idx]
        } else {
            0
        }
    }

    /// Add a material to the palette, return its ID
    pub fn add_material(&mut self, mat: VoxelMaterial) -> u8 {
        // Check if already exists
        for (i, existing) in self.materials.iter().enumerate() {
            if existing.name == mat.name {
                return i as u8;
            }
        }
        let id = self.materials.len() as u8;
        let mut mat = mat;
        mat.id = id;
        self.materials.push(mat);
        id
    }

    /// Voxelize a mesh object into the grid
    pub fn voxelize_mesh(&mut self, mesh: &MeshObject) {
        let mat_id = self.add_material(mesh.material.clone());

        // Compute mesh bounding box
        let (mesh_min, mesh_max) = mesh_bounds(mesh);

        // Find voxel range that overlaps mesh bounds
        let x_start = ((mesh_min[0] - self.origin[0]) / self.voxel_size).floor().max(0.0) as usize;
        let y_start = ((mesh_min[1] - self.origin[1]) / self.voxel_size).floor().max(0.0) as usize;
        let z_start = ((mesh_min[2] - self.origin[2]) / self.voxel_size).floor().max(0.0) as usize;

        let x_end = ((mesh_max[0] - self.origin[0]) / self.voxel_size).ceil() as usize;
        let y_end = ((mesh_max[1] - self.origin[1]) / self.voxel_size).ceil() as usize;
        let z_end = ((mesh_max[2] - self.origin[2]) / self.voxel_size).ceil() as usize;

        let x_end = x_end.min(self.nx);
        let y_end = y_end.min(self.ny);
        let z_end = z_end.min(self.nz);

        // For each voxel in range, test if center is inside mesh
        for z in z_start..z_end {
            for y in y_start..y_end {
                for x in x_start..x_end {
                    let center = self.voxel_center(x, y, z);
                    if point_in_mesh(&center, mesh) {
                        self.set(x, y, z, mat_id);
                    }
                }
            }
        }
    }

    /// Export to MEEP Python script
    pub fn to_meep_script(&self, config: &MeepConfig) -> String {
        let mut script = String::with_capacity(self.data.len() * 2 + 4096);

        // Header
        script.push_str(r#"#!/usr/bin/env python3
"""
MEEP FDTD Simulation - Voxelized geometry from Mittens

Grid: {nx} x {ny} x {nz} voxels
Voxel size: {vs} mm
"""

import meep as mp
import numpy as np
import argparse
import os
from datetime import datetime

"#);
        script = script
            .replace("{nx}", &self.nx.to_string())
            .replace("{ny}", &self.ny.to_string())
            .replace("{nz}", &self.nz.to_string())
            .replace("{vs}", &format!("{:.4}", self.voxel_size));

        // Material definitions
        script.push_str("# =============================================================================\n");
        script.push_str("# Materials\n");
        script.push_str("# =============================================================================\n\n");
        script.push_str("MATERIALS = [\n");
        for mat in &self.materials {
            script.push_str(&format!("    {},  # {} (id={})\n", mat.to_meep(), mat.name, mat.id));
        }
        script.push_str("]\n\n");

        // Grid parameters
        script.push_str(&format!(r#"
# Grid parameters
ORIGIN = [{:.6}, {:.6}, {:.6}]
VOXEL_SIZE = {:.6}
NX, NY, NZ = {}, {}, {}

# Cell size (add PML)
PML_THICKNESS = {:.6}
CELL_X = NX * VOXEL_SIZE + 2 * PML_THICKNESS
CELL_Y = NY * VOXEL_SIZE + 2 * PML_THICKNESS
CELL_Z = NZ * VOXEL_SIZE + 2 * PML_THICKNESS

# Frequency
FCEN = {:.6e}
FWIDTH = {:.6e}
RESOLUTION = {:.1}

"#,
            self.origin[0], self.origin[1], self.origin[2],
            self.voxel_size,
            self.nx, self.ny, self.nz,
            config.pml_thickness,
            config.fcen,
            config.fwidth,
            config.resolution,
        ));

        // Material grid data (compressed RLE or raw)
        script.push_str("# =============================================================================\n");
        script.push_str("# Voxel Data (material IDs)\n");
        script.push_str("# =============================================================================\n\n");

        // Use numpy-compatible format
        script.push_str("# Decode voxel data\n");
        script.push_str("import base64, zlib\n\n");

        // Compress the voxel data
        let compressed = compress_voxels(&self.data);
        let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &compressed);

        script.push_str("VOXEL_DATA_B64 = \"\"\"\n");
        // Split into 76-char lines
        for chunk in b64.as_bytes().chunks(76) {
            script.push_str(std::str::from_utf8(chunk).unwrap());
            script.push_str("\n");
        }
        script.push_str("\"\"\"\n\n");

        script.push_str(r#"
def decode_voxels():
    """Decode compressed voxel data to 3D numpy array."""
    data = base64.b64decode(VOXEL_DATA_B64.strip())
    data = zlib.decompress(data)
    arr = np.frombuffer(data, dtype=np.uint8)
    return arr.reshape((NZ, NY, NX))


# =============================================================================
# Geometry using material_function
# =============================================================================

def make_material_function(voxels):
    """Create a material function from voxel data."""
    def mat_func(p):
        # Convert position to voxel indices
        x = int((p.x - ORIGIN[0]) / VOXEL_SIZE)
        y = int((p.y - ORIGIN[1]) / VOXEL_SIZE)
        z = int((p.z - ORIGIN[2]) / VOXEL_SIZE)

        # Bounds check
        if x < 0 or x >= NX or y < 0 or y >= NY or z < 0 or z >= NZ:
            return MATERIALS[0]  # air

        mat_id = voxels[z, y, x]
        return MATERIALS[mat_id]

    return mat_func


def build_geometry():
    """Build geometry using material function."""
    voxels = decode_voxels()

    # Create a block covering the voxel region with material function
    geometry = [
        mp.Block(
            center=mp.Vector3(
                ORIGIN[0] + NX * VOXEL_SIZE / 2,
                ORIGIN[1] + NY * VOXEL_SIZE / 2,
                ORIGIN[2] + NZ * VOXEL_SIZE / 2
            ),
            size=mp.Vector3(NX * VOXEL_SIZE, NY * VOXEL_SIZE, NZ * VOXEL_SIZE),
            material=mp.MaterialGrid(
                mp.Vector3(NX, NY, NZ),
                MATERIALS[0],  # default (air)
                MATERIALS[1] if len(MATERIALS) > 1 else MATERIALS[0],  # contrast
                grid_type="U_MEAN"
            )
        )
    ]

    # Note: For complex multi-material, use material_function instead:
    # material=mp.material_function(make_material_function(voxels))

    return geometry, voxels


# =============================================================================
# Sources
# =============================================================================

def build_sources():
    """Build excitation sources."""
    return [
        mp.Source(
            src=mp.GaussianSource(frequency=FCEN, fwidth=FWIDTH),
            component=mp.Ez,
            center=mp.Vector3(0, 0, 0),
            size=mp.Vector3(0, 0, 0)
        )
    ]


# =============================================================================
# Simulation
# =============================================================================

def run_simulation(output_dir="output", use_eigenmode=False):
    """Run FDTD or eigenmode simulation."""
    os.makedirs(output_dir, exist_ok=True)

    print("=" * 60)
    print("MEEP Voxelized FDTD Simulation")
    print("=" * 60)
    print(f"Grid: {NX} x {NY} x {NZ} = {NX*NY*NZ:,} voxels")
    print(f"Cell: {CELL_X:.2f} x {CELL_Y:.2f} x {CELL_Z:.2f} mm")
    print(f"Materials: {len(MATERIALS)}")
    print()

    geometry, voxels = build_geometry()
    sources = build_sources()

    sim = mp.Simulation(
        cell_size=mp.Vector3(CELL_X, CELL_Y, CELL_Z),
        geometry=geometry,
        sources=sources,
        boundary_layers=[mp.PML(thickness=PML_THICKNESS)],
        resolution=RESOLUTION,
        default_material=mp.air,
    )

    if use_eigenmode:
        # Find resonant modes
        print("Finding eigenmodes...")
        harminv_results = sim.run(
            mp.Harminv(mp.Ez, mp.Vector3(0, 0, 0), FCEN, FWIDTH),
            until_after_sources=200
        )
        print("\\nResonant frequencies found:")
        for mode in harminv_results:
            print(f"  f = {mode.freq:.6f}, Q = {mode.Q:.1f}")
    else:
        # Time-domain simulation
        field_data = {"t": [], "ez": []}

        def capture(sim):
            ez = sim.get_field_point(mp.Ez, mp.Vector3(0, 0, 0))
            field_data["t"].append(sim.meep_time())
            field_data["ez"].append(complex(ez).real)

        print("Running time-domain simulation...")
        sim.run(mp.at_every(1, capture), until_after_sources=100)

        np.savez(f"{output_dir}/results.npz",
                 t=field_data["t"],
                 ez=field_data["ez"],
                 voxels=voxels)
        print(f"Results saved to {output_dir}/results.npz")

    return sim


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--eigenmode", action="store_true", help="Find resonant modes")
    parser.add_argument("--output", default="output", help="Output directory")
    args = parser.parse_args()

    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    run_simulation(f"{args.output}/sim_{timestamp}", use_eigenmode=args.eigenmode)
"#);

        script
    }
}

/// MEEP configuration
#[derive(Debug, Clone)]
pub struct MeepConfig {
    pub resolution: f64,
    pub pml_thickness: f64,
    pub fcen: f64,
    pub fwidth: f64,
}

impl Default for MeepConfig {
    fn default() -> Self {
        Self {
            resolution: 10.0,
            pml_thickness: 1.0,
            fcen: 0.01,    // ~3 GHz at mm scale
            fwidth: 0.005,
        }
    }
}

// =============================================================================
// Ray casting for point-in-mesh test
// =============================================================================

fn mesh_bounds(mesh: &MeshObject) -> ([f64; 3], [f64; 3]) {
    let mut min = [f64::MAX; 3];
    let mut max = [f64::MIN; 3];

    for v in &mesh.vertices {
        for i in 0..3 {
            min[i] = min[i].min(v[i] as f64);
            max[i] = max[i].max(v[i] as f64);
        }
    }

    (min, max)
}

/// Test if a point is inside a watertight mesh using ray casting
fn point_in_mesh(point: &[f64; 3], mesh: &MeshObject) -> bool {
    // Cast ray in +X direction, count intersections
    let ray_origin = *point;
    let ray_dir = [1.0, 0.0, 0.0];

    let mut intersections = 0;

    for tri in &mesh.triangles {
        let v0 = &mesh.vertices[tri[0] as usize];
        let v1 = &mesh.vertices[tri[1] as usize];
        let v2 = &mesh.vertices[tri[2] as usize];

        if ray_triangle_intersect(&ray_origin, &ray_dir, v0, v1, v2) {
            intersections += 1;
        }
    }

    // Odd number of intersections = inside
    intersections % 2 == 1
}

/// Möller–Trumbore ray-triangle intersection
fn ray_triangle_intersect(
    ray_origin: &[f64; 3],
    ray_dir: &[f64; 3],
    v0: &[f32; 3],
    v1: &[f32; 3],
    v2: &[f32; 3],
) -> bool {
    const EPSILON: f64 = 1e-9;

    let v0 = [v0[0] as f64, v0[1] as f64, v0[2] as f64];
    let v1 = [v1[0] as f64, v1[1] as f64, v1[2] as f64];
    let v2 = [v2[0] as f64, v2[1] as f64, v2[2] as f64];

    let edge1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let edge2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];

    let h = cross(ray_dir, &edge2);
    let a = dot(&edge1, &h);

    if a.abs() < EPSILON {
        return false; // Ray parallel to triangle
    }

    let f = 1.0 / a;
    let s = [
        ray_origin[0] - v0[0],
        ray_origin[1] - v0[1],
        ray_origin[2] - v0[2],
    ];
    let u = f * dot(&s, &h);

    if u < 0.0 || u > 1.0 {
        return false;
    }

    let q = cross(&s, &edge1);
    let v = f * dot(ray_dir, &q);

    if v < 0.0 || u + v > 1.0 {
        return false;
    }

    let t = f * dot(&edge2, &q);

    t > EPSILON // Intersection in front of ray origin
}

#[inline]
fn dot(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn cross(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Compress voxel data using zlib
fn compress_voxels(data: &[u8]) -> Vec<u8> {
    use std::io::Write;
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(data).unwrap();
    encoder.finish().unwrap()
}

// =============================================================================
// Convert from Mittens mesh data
// =============================================================================

/// Convert Mittens MeshData to voxel grid
pub fn voxelize_scene(
    meshes: &[(crate::geometry::MeshData, VoxelMaterial)],
    voxel_size: f64,
    padding: f64,
) -> VoxelGrid {
    // Compute scene bounds
    let mut min = [f64::MAX; 3];
    let mut max = [f64::MIN; 3];

    for (mesh, _) in meshes {
        let n_verts = mesh.positions.len() / 3;
        for i in 0..n_verts {
            let x = mesh.positions[i * 3] as f64;
            let y = mesh.positions[i * 3 + 1] as f64;
            let z = mesh.positions[i * 3 + 2] as f64;
            min[0] = min[0].min(x);
            min[1] = min[1].min(y);
            min[2] = min[2].min(z);
            max[0] = max[0].max(x);
            max[1] = max[1].max(y);
            max[2] = max[2].max(z);
        }
    }

    // Add padding
    let origin = [min[0] - padding, min[1] - padding, min[2] - padding];
    let size = [
        max[0] - min[0] + 2.0 * padding,
        max[1] - min[1] + 2.0 * padding,
        max[2] - min[2] + 2.0 * padding,
    ];

    let mut grid = VoxelGrid::new(origin, size, voxel_size);

    // Convert and voxelize each mesh
    for (mesh_data, material) in meshes {
        let mesh_obj = mesh_data_to_object(mesh_data, material.clone());
        grid.voxelize_mesh(&mesh_obj);
    }

    grid
}

fn mesh_data_to_object(mesh: &crate::geometry::MeshData, material: VoxelMaterial) -> MeshObject {
    let n_verts = mesh.positions.len() / 3;
    let n_tris = mesh.indices.len() / 3;

    let mut vertices = Vec::with_capacity(n_verts);
    for i in 0..n_verts {
        vertices.push([
            mesh.positions[i * 3],
            mesh.positions[i * 3 + 1],
            mesh.positions[i * 3 + 2],
        ]);
    }

    let mut triangles = Vec::with_capacity(n_tris);
    for i in 0..n_tris {
        triangles.push([
            mesh.indices[i * 3],
            mesh.indices[i * 3 + 1],
            mesh.indices[i * 3 + 2],
        ]);
    }

    MeshObject {
        name: material.name.clone(),
        vertices,
        triangles,
        material,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Count intersections for debugging
    fn count_intersections(point: &[f64; 3], mesh: &MeshObject) -> i32 {
        let ray_origin = *point;
        let ray_dir = [1.0, 0.0, 0.0];
        let mut count = 0;

        for tri in &mesh.triangles {
            let v0 = &mesh.vertices[tri[0] as usize];
            let v1 = &mesh.vertices[tri[1] as usize];
            let v2 = &mesh.vertices[tri[2] as usize];

            if ray_triangle_intersect(&ray_origin, &ray_dir, v0, v1, v2) {
                count += 1;
            }
        }

        count
    }

    #[test]
    fn test_point_in_cube() {
        // Simple cube from (0,0,0) to (1,1,1)
        // Using Manifold-style winding: outward-facing normals, CCW from outside
        let mesh = MeshObject {
            name: "cube".to_string(),
            vertices: vec![
                [0.0, 0.0, 0.0], // 0
                [1.0, 0.0, 0.0], // 1
                [1.0, 1.0, 0.0], // 2
                [0.0, 1.0, 0.0], // 3
                [0.0, 0.0, 1.0], // 4
                [1.0, 0.0, 1.0], // 5
                [1.0, 1.0, 1.0], // 6
                [0.0, 1.0, 1.0], // 7
            ],
            triangles: vec![
                // Face at z=0 (front): normal (0,0,-1), CCW from outside = CW from inside
                [0, 2, 1], [0, 3, 2],
                // Face at z=1 (back): normal (0,0,+1)
                [4, 5, 6], [4, 6, 7],
                // Face at y=0 (bottom): normal (0,-1,0)
                [0, 1, 5], [0, 5, 4],
                // Face at y=1 (top): normal (0,+1,0)
                [3, 7, 6], [3, 6, 2],
                // Face at x=0 (left): normal (-1,0,0)
                [0, 4, 7], [0, 7, 3],
                // Face at x=1 (right): normal (+1,0,0)
                [1, 2, 6], [1, 6, 5],
            ],
            material: VoxelMaterial::pec(),
        };

        // Test a point off-diagonal to avoid edge ambiguity
        // Point at (0.5, 0.3, 0.7) is clearly inside and not on any triangle edge
        let inside_count = count_intersections(&[0.5, 0.3, 0.7], &mesh);
        assert_eq!(inside_count, 1, "Inside point should have 1 intersection (odd = inside), got {}", inside_count);
        assert!(point_in_mesh(&[0.5, 0.3, 0.7], &mesh), "Inside point should be inside");

        // Test outside +X
        let outside_count = count_intersections(&[2.0, 0.5, 0.5], &mesh);
        assert_eq!(outside_count, 0, "Outside +X should have 0 intersections, got {}", outside_count);
        assert!(!point_in_mesh(&[2.0, 0.5, 0.5], &mesh), "Outside +X should be outside");

        // Test outside -X (point at x=-1, ray goes +X, should hit both -X face and +X face)
        let outside_neg_count = count_intersections(&[-1.0, 0.3, 0.7], &mesh);
        assert_eq!(outside_neg_count, 2, "Outside -X should have 2 intersections (even = outside), got {}", outside_neg_count);
        assert!(!point_in_mesh(&[-1.0, 0.3, 0.7], &mesh), "Outside -X should be outside");
    }
}
