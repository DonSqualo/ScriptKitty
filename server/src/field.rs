//! Magnetic field computation using Biot-Savart law
//!
//! Computes B-field from circular current loops (coils)

use std::f64::consts::PI;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlaneType {
    XZ = 0,  // Plane at Y=offset
    XY = 1,  // Plane at Z=offset
    YZ = 2,  // Plane at X=offset
}

const MU0: f64 = 4.0 * PI * 1e-7; // Permeability of free space (H/m)

/// A circular current loop
#[derive(Clone)]
pub struct CurrentLoop {
    pub center: [f64; 3],    // Center position (mm)
    pub radius: f64,         // Radius (mm)
    #[allow(dead_code)]
    pub normal: [f64; 3],    // Normal direction (unit vector) - for future non-Z-aligned coils
    pub ampere_turns: f64,   // Current × turns (A·turns)
}

/// Magnetic field data for visualization
pub struct FieldData {
    // 2D slice data
    pub plane_type: PlaneType,
    pub slice_width: usize,
    pub slice_height: usize,
    pub slice_bounds: [f64; 4],  // Bounds in mm: [axis1_min, axis1_max, axis2_min, axis2_max]
    pub slice_offset: f64,       // Offset along normal axis (mm)
    pub slice_bx: Vec<f32>,      // B component along first axis (T)
    pub slice_bz: Vec<f32>,      // B component along second axis (T)
    pub slice_magnitude: Vec<f32>, // |B| (T)

    // 3D arrow field
    pub arrows_positions: Vec<f32>,  // x, y, z positions
    pub arrows_vectors: Vec<f32>,    // Bx, By, Bz vectors (normalized for display)
    pub arrows_magnitudes: Vec<f32>, // |B| for coloring

    // 1D line data (along Z axis)
    pub line_z: Vec<f32>,        // Z positions (mm)
    pub line_bz: Vec<f32>,       // Bz values (T)
}

impl FieldData {
    pub fn to_binary(&self) -> Vec<u8> {
        let mut data = Vec::new();

        // Header: type marker
        data.extend_from_slice(b"FIELD\0\0\0");

        // 2D slice dimensions
        data.extend_from_slice(&(self.slice_width as u32).to_le_bytes());
        data.extend_from_slice(&(self.slice_height as u32).to_le_bytes());

        // 2D slice bounds
        for &b in &self.slice_bounds {
            data.extend_from_slice(&(b as f32).to_le_bytes());
        }

        // Plane type (u8) and offset (f32)
        data.push(self.plane_type as u8);
        data.extend_from_slice(&(self.slice_offset as f32).to_le_bytes());

        // 2D slice data
        for &v in &self.slice_bx {
            data.extend_from_slice(&v.to_le_bytes());
        }
        for &v in &self.slice_bz {
            data.extend_from_slice(&v.to_le_bytes());
        }
        for &v in &self.slice_magnitude {
            data.extend_from_slice(&v.to_le_bytes());
        }

        // 3D arrows count
        let num_arrows = self.arrows_positions.len() / 3;
        data.extend_from_slice(&(num_arrows as u32).to_le_bytes());

        // 3D arrow data
        for &v in &self.arrows_positions {
            data.extend_from_slice(&v.to_le_bytes());
        }
        for &v in &self.arrows_vectors {
            data.extend_from_slice(&v.to_le_bytes());
        }
        for &v in &self.arrows_magnitudes {
            data.extend_from_slice(&v.to_le_bytes());
        }

        // 1D line count
        data.extend_from_slice(&(self.line_z.len() as u32).to_le_bytes());

        // 1D line data
        for &v in &self.line_z {
            data.extend_from_slice(&v.to_le_bytes());
        }
        for &v in &self.line_bz {
            data.extend_from_slice(&v.to_le_bytes());
        }

        data
    }
}

/// Compute B-field from a single current loop at a point using Biot-Savart law
fn biot_savart_loop(loop_: &CurrentLoop, point: [f64; 3], num_segments: usize) -> [f64; 3] {
    let mut b = [0.0, 0.0, 0.0];

    // Convert mm to m for SI units
    let r_m = loop_.radius * 1e-3;
    let cx = loop_.center[0] * 1e-3;
    let cy = loop_.center[1] * 1e-3;
    let cz = loop_.center[2] * 1e-3;
    let px = point[0] * 1e-3;
    let py = point[1] * 1e-3;
    let pz = point[2] * 1e-3;

    // For Z-axis aligned loop (normal = [0, 0, 1])
    // Loop points: (cx + R*cos(θ), cy + R*sin(θ), cz)

    let dtheta = 2.0 * PI / num_segments as f64;

    for i in 0..num_segments {
        let theta = i as f64 * dtheta;
        let theta_mid = theta + dtheta / 2.0;

        // Wire element position
        let wx = cx + r_m * theta_mid.cos();
        let wy = cy + r_m * theta_mid.sin();
        let wz = cz;

        // dl vector (tangent to loop)
        let dlx = -r_m * theta_mid.sin() * dtheta;
        let dly = r_m * theta_mid.cos() * dtheta;
        let dlz = 0.0;

        // Vector from wire element to field point
        let rx = px - wx;
        let ry = py - wy;
        let rz = pz - wz;
        let r_mag = (rx * rx + ry * ry + rz * rz).sqrt();

        if r_mag < 1e-10 {
            continue;
        }

        // dl × r̂
        let r3 = r_mag * r_mag * r_mag;
        let cross_x = dly * rz - dlz * ry;
        let cross_y = dlz * rx - dlx * rz;
        let cross_z = dlx * ry - dly * rx;

        // dB = (μ₀/4π) * I * (dl × r) / r³
        let factor = MU0 / (4.0 * PI) * loop_.ampere_turns / r3;

        b[0] += factor * cross_x;
        b[1] += factor * cross_y;
        b[2] += factor * cross_z;
    }

    b
}

/// Compute total B-field from multiple loops at a point
fn compute_field(loops: &[CurrentLoop], point: [f64; 3]) -> [f64; 3] {
    let mut b = [0.0, 0.0, 0.0];

    for loop_ in loops {
        let b_loop = biot_savart_loop(loop_, point, 64);
        b[0] += b_loop[0];
        b[1] += b_loop[1];
        b[2] += b_loop[2];
    }

    b
}

/// Generate field visualization data for Helmholtz coil configuration
pub fn compute_helmholtz_field(
    _coil_radius: f64,     // Mean radius (mm) - unused, kept for API compatibility
    coil_inner_r: f64,     // Inner radius (mm)
    coil_outer_r: f64,     // Outer radius (mm)
    coil_width: f64,       // Axial width (mm)
    gap: f64,              // Gap between coils (mm)
    ampere_turns: f64,     // A·turns per coil
    num_layers: usize,     // Radial layers to model
    plane: PlaneType,      // Plane orientation for 2D slice
    plane_offset: f64,     // Offset along plane normal (mm)
) -> FieldData {
    // Create current loops to model the coils
    // Distribute loops across the cross-section
    let mut loops = Vec::new();

    let coil_z = gap / 2.0 + coil_width / 2.0;
    let dr = (coil_outer_r - coil_inner_r) / num_layers as f64;
    let at_per_layer = ampere_turns / num_layers as f64;

    for layer in 0..num_layers {
        let r = coil_inner_r + (layer as f64 + 0.5) * dr;

        // Upper coil
        loops.push(CurrentLoop {
            center: [0.0, 0.0, coil_z],
            radius: r,
            normal: [0.0, 0.0, 1.0],
            ampere_turns: at_per_layer,
        });

        // Lower coil
        loops.push(CurrentLoop {
            center: [0.0, 0.0, -coil_z],
            radius: r,
            normal: [0.0, 0.0, 1.0],
            ampere_turns: at_per_layer,
        });
    }

    // Compute 2D slice
    let slice_width = 80;
    let slice_height = 80;
    let extent = coil_outer_r * 2.5;
    let z_extent = (coil_z + coil_width) * 1.5;

    // Bounds depend on plane type
    let (axis1_min, axis1_max, axis2_min, axis2_max) = match plane {
        PlaneType::XZ => (-extent, extent, -z_extent, z_extent),
        PlaneType::XY => (-extent, extent, -extent, extent),
        PlaneType::YZ => (-extent, extent, -z_extent, z_extent),
    };

    let mut slice_bx = Vec::with_capacity(slice_width * slice_height);
    let mut slice_bz = Vec::with_capacity(slice_width * slice_height);
    let mut slice_magnitude = Vec::with_capacity(slice_width * slice_height);

    for j in 0..slice_height {
        let axis2 = axis2_min + (j as f64 + 0.5) * (axis2_max - axis2_min) / slice_height as f64;
        for i in 0..slice_width {
            let axis1 = axis1_min + (i as f64 + 0.5) * (axis1_max - axis1_min) / slice_width as f64;

            // Map axes to 3D point based on plane type
            let point = match plane {
                PlaneType::XZ => [axis1, plane_offset, axis2],  // x, y=offset, z
                PlaneType::XY => [axis1, axis2, plane_offset],  // x, y, z=offset
                PlaneType::YZ => [plane_offset, axis1, axis2],  // x=offset, y, z
            };

            let b = compute_field(&loops, point);
            let mag = (b[0] * b[0] + b[1] * b[1] + b[2] * b[2]).sqrt();

            // Store in-plane components based on plane type
            let (b1, b2) = match plane {
                PlaneType::XZ => (b[0], b[2]),  // Bx, Bz
                PlaneType::XY => (b[0], b[1]),  // Bx, By
                PlaneType::YZ => (b[1], b[2]),  // By, Bz
            };

            slice_bx.push(b1 as f32);
            slice_bz.push(b2 as f32);
            slice_magnitude.push(mag as f32);
        }
    }
    let slice_bounds = [axis1_min, axis1_max, axis2_min, axis2_max];

    // Compute 3D arrow field
    let arrow_grid = 10;
    let mut arrows_positions = Vec::new();
    let mut arrows_vectors = Vec::new();
    let mut arrows_magnitudes = Vec::new();

    let arrow_extent = coil_outer_r * 1.8;
    let arrow_z_extent = z_extent * 0.8;

    for k in 0..arrow_grid {
        let z = -arrow_z_extent + (k as f64 + 0.5) * 2.0 * arrow_z_extent / arrow_grid as f64;
        for j in 0..arrow_grid {
            let y = -arrow_extent + (j as f64 + 0.5) * 2.0 * arrow_extent / arrow_grid as f64;
            for i in 0..arrow_grid {
                let x = -arrow_extent + (i as f64 + 0.5) * 2.0 * arrow_extent / arrow_grid as f64;

                // Skip points inside the coils
                let rho = (x * x + y * y).sqrt();
                if rho > coil_inner_r * 0.9 && rho < coil_outer_r * 1.1 {
                    let z_abs = z.abs();
                    if z_abs > coil_z - coil_width && z_abs < coil_z + coil_width {
                        continue;
                    }
                }

                let b = compute_field(&loops, [x, y, z]);
                let mag = (b[0] * b[0] + b[1] * b[1] + b[2] * b[2]).sqrt();

                if mag > 1e-10 {
                    arrows_positions.push(x as f32);
                    arrows_positions.push(y as f32);
                    arrows_positions.push(z as f32);

                    // Normalize for display
                    arrows_vectors.push((b[0] / mag) as f32);
                    arrows_vectors.push((b[1] / mag) as f32);
                    arrows_vectors.push((b[2] / mag) as f32);

                    arrows_magnitudes.push(mag as f32);
                }
            }
        }
    }

    // Compute 1D line along Z axis
    let line_points = 101;
    let mut line_z = Vec::with_capacity(line_points);
    let mut line_bz = Vec::with_capacity(line_points);

    for i in 0..line_points {
        let z = -z_extent + i as f64 * (2.0 * z_extent) / (line_points - 1) as f64;
        let b = compute_field(&loops, [0.0, 0.0, z]);

        line_z.push(z as f32);
        line_bz.push(b[2] as f32);
    }

    FieldData {
        plane_type: plane,
        slice_width,
        slice_height,
        slice_bounds,
        slice_offset: plane_offset,
        slice_bx,
        slice_bz,
        slice_magnitude,
        arrows_positions,
        arrows_vectors,
        arrows_magnitudes,
        line_z,
        line_bz,
    }
}
