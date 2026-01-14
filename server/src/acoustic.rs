//! Acoustic pressure field via Rayleigh integral
//!
//! Computes pressure from circular piston transducer using numerical
//! integration of the Rayleigh-Sommerfeld diffraction integral,
//! with reflection from the coverslip boundary.

use std::f64::consts::PI;

pub struct AcousticConfig {
    pub frequency: f64,           // Hz
    pub transducer_radius: f64,   // mm
    pub transducer_z: f64,        // mm (height above coverslip)
    pub medium_radius: f64,       // mm (cylindrical medium radius)
    pub medium_height: f64,       // mm
    pub speed_of_sound: f64,      // mm/s
    pub drive_amplitude: f64,     // arbitrary units
}

pub struct AcousticFieldData {
    pub slice_width: usize,
    pub slice_height: usize,
    pub slice_bounds: [f64; 4],
    pub slice_pressure: Vec<f32>,
    pub slice_magnitude: Vec<f32>,
}

impl AcousticFieldData {
    pub fn to_binary(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(b"FIELD\0\0\0");
        data.extend_from_slice(&(self.slice_width as u32).to_le_bytes());
        data.extend_from_slice(&(self.slice_height as u32).to_le_bytes());
        for &b in &self.slice_bounds {
            data.extend_from_slice(&(b as f32).to_le_bytes());
        }
        for &v in &self.slice_pressure {
            data.extend_from_slice(&v.to_le_bytes());
        }
        for &v in &self.slice_pressure {
            data.extend_from_slice(&v.to_le_bytes());
        }
        for &v in &self.slice_magnitude {
            data.extend_from_slice(&v.to_le_bytes());
        }
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());
        data
    }
}

/// Compute pressure at field point from circular piston via Rayleigh integral
/// Returns complex pressure (real, imag)
fn rayleigh_piston(
    field_r: f64,
    field_z: f64,
    piston_z: f64,
    piston_radius: f64,
    k: f64,
    n_rings: usize,
    n_segments: usize,
) -> (f64, f64) {
    let mut p_real = 0.0;
    let mut p_imag = 0.0;

    let dz = piston_z - field_z;

    for i_ring in 0..n_rings {
        let rho = piston_radius * (i_ring as f64 + 0.5) / n_rings as f64;
        let ring_area = 2.0 * PI * rho * (piston_radius / n_rings as f64);
        let d_area = ring_area / n_segments as f64;

        for i_seg in 0..n_segments {
            let phi = 2.0 * PI * i_seg as f64 / n_segments as f64;
            let src_x = rho * phi.cos();
            let src_y = rho * phi.sin();

            let dx = field_r - src_x;
            let dy = 0.0 - src_y;
            let dist = (dx * dx + dy * dy + dz * dz).sqrt();

            if dist < 1e-10 {
                continue;
            }

            let phase = k * dist;
            p_real += d_area * phase.cos() / dist;
            p_imag += d_area * (-phase.sin()) / dist;
        }
    }

    (p_real, p_imag)
}

pub fn compute_acoustic_field(config: &AcousticConfig) -> AcousticFieldData {
    let wavelength = config.speed_of_sound / config.frequency;
    let k = 2.0 * PI / wavelength;

    // Impedances for reflection coefficient
    let z_water = 1.48e6;  // kg/(m²·s)
    let z_glass = 12.6e6;  // kg/(m²·s)
    let r_coeff = (z_glass - z_water) / (z_glass + z_water); // ~0.79

    let slice_width = 80;
    let slice_height = 80;

    // Bounds: full cylindrical medium from coverslip to liquid surface
    let x_min = -config.medium_radius;
    let x_max = config.medium_radius;
    let z_min = 0.0;
    let z_max = config.medium_height;

    let n_rings = 12;
    let n_segments = 24;

    let mut slice_magnitude = Vec::with_capacity(slice_width * slice_height);

    for j in 0..slice_height {
        let z = z_min + (j as f64 + 0.5) * (z_max - z_min) / slice_height as f64;

        for i in 0..slice_width {
            let x = x_min + (i as f64 + 0.5) * (x_max - x_min) / slice_width as f64;
            let r = x.abs();

            // Outside cylindrical medium
            if r >= config.medium_radius {
                slice_magnitude.push(0.0);
                continue;
            }

            // Direct wave from transducer (at z = transducer_z, facing down)
            let (p_direct_re, p_direct_im) = rayleigh_piston(
                r,
                z,
                config.transducer_z,
                config.transducer_radius,
                k,
                n_rings,
                n_segments,
            );

            // Reflected wave: mirror source at z = -transducer_z
            let (p_reflect_re, p_reflect_im) = rayleigh_piston(
                r,
                z,
                -config.transducer_z,
                config.transducer_radius,
                k,
                n_rings,
                n_segments,
            );

            // Total field: direct + reflected (with reflection coefficient)
            let p_total_re = p_direct_re + r_coeff * p_reflect_re;
            let p_total_im = p_direct_im + r_coeff * p_reflect_im;

            let magnitude = (p_total_re * p_total_re + p_total_im * p_total_im).sqrt();
            slice_magnitude.push(magnitude as f32);
        }
    }

    // Normalize
    let max_mag = slice_magnitude.iter().cloned().fold(0.0f32, f32::max);
    if max_mag > 0.0 {
        for m in &mut slice_magnitude {
            *m /= max_mag;
        }
    }

    AcousticFieldData {
        slice_width,
        slice_height,
        slice_bounds: [x_min, x_max, z_min, z_max],
        slice_pressure: slice_magnitude.clone(),
        slice_magnitude,
    }
}
