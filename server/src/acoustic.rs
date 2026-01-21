//! Acoustic field computation using Rayleigh-Sommerfeld diffraction integral
//!
//! Computes pressure field from circular piston transducers with coverslip reflection

use std::f64::consts::PI;

use crate::field::{FieldData, PlaneType};

// ===========================

const Z_WATER: f64 = 1.48e6;  // Acoustic impedance of water (Rayl)
const Z_GLASS: f64 = 12.6e6; // Acoustic impedance of borosilicate glass (Rayl)
const REFLECTION_COEFF: f64 = (Z_GLASS - Z_WATER) / (Z_GLASS + Z_WATER); // ~0.79

pub struct AcousticConfig {
    pub frequency: f64,         // Hz
    pub transducer_radius: f64, // mm
    pub transducer_z: f64,      // mm (height above coverslip at z=0)
    pub medium_radius: f64,     // mm (cylindrical domain)
    pub medium_height: f64,     // mm
    pub speed_of_sound: f64,    // mm/s (default: 1480 * 1000 for water)
    pub drive_amplitude: f64,   // arbitrary scaling
}

impl Default for AcousticConfig {
    fn default() -> Self {
        Self {
            frequency: 1e6,
            transducer_radius: 6.0,
            transducer_z: 5.0,
            medium_radius: 13.0,
            medium_height: 8.0,
            speed_of_sound: 1480.0 * 1000.0,
            drive_amplitude: 1.0,
        }
    }
}

// ===========================

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

    let dr = piston_radius / n_rings as f64;

    for ring in 0..n_rings {
        let rho = piston_radius * (ring as f64 + 0.5) / n_rings as f64;
        let ring_area = 2.0 * PI * rho * dr;
        let d_area = ring_area / n_segments as f64;

        for seg in 0..n_segments {
            let phi = 2.0 * PI * seg as f64 / n_segments as f64;
            let src_x = rho * phi.cos();
            let src_y = rho * phi.sin();

            let dx = field_r - src_x;
            let dz = piston_z - field_z;
            let dist = (dx * dx + src_y * src_y + dz * dz).sqrt();

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

fn compute_pressure_at_point(r: f64, z: f64, config: &AcousticConfig) -> (f64, f64) {
    let k = 2.0 * PI * config.frequency / config.speed_of_sound;
    let n_rings = 12;
    let n_segments = 24;

    // Direct contribution from real transducer
    let (direct_real, direct_imag) = rayleigh_piston(
        r,
        z,
        config.transducer_z,
        config.transducer_radius,
        k,
        n_rings,
        n_segments,
    );

    // Reflected contribution from virtual transducer (mirrored across z=0)
    let (reflect_real, reflect_imag) = rayleigh_piston(
        r,
        z,
        -config.transducer_z,
        config.transducer_radius,
        k,
        n_rings,
        n_segments,
    );

    let total_real = direct_real + REFLECTION_COEFF * reflect_real;
    let total_imag = direct_imag + REFLECTION_COEFF * reflect_imag;

    (total_real * config.drive_amplitude, total_imag * config.drive_amplitude)
}

// ===========================

pub fn compute_acoustic_field(config: &AcousticConfig) -> FieldData {
    let slice_width = 80;
    let slice_height = 80;

    let x_min = -config.medium_radius;
    let x_max = config.medium_radius;
    let z_min = 0.0;
    let z_max = config.medium_height;

    let mut slice_pressure_real = Vec::with_capacity(slice_width * slice_height);
    let mut slice_pressure_imag = Vec::with_capacity(slice_width * slice_height);
    let mut slice_magnitude = Vec::with_capacity(slice_width * slice_height);

    let mut max_magnitude: f64 = 0.0;

    for j in 0..slice_height {
        let z = z_min + (j as f64 + 0.5) * (z_max - z_min) / slice_height as f64;
        for i in 0..slice_width {
            let x = x_min + (i as f64 + 0.5) * (x_max - x_min) / slice_width as f64;
            let r = x.abs();

            let (p_real, p_imag) = compute_pressure_at_point(r, z, config);
            let mag = (p_real * p_real + p_imag * p_imag).sqrt();

            slice_pressure_real.push(p_real);
            slice_pressure_imag.push(p_imag);
            slice_magnitude.push(mag);

            if mag > max_magnitude {
                max_magnitude = mag;
            }
        }
    }

    // Normalize magnitude for display (0-1 range)
    let norm_factor = if max_magnitude > 1e-10 {
        1.0 / max_magnitude
    } else {
        1.0
    };

    let slice_bx: Vec<f32> = slice_magnitude
        .iter()
        .map(|&m| (m * norm_factor) as f32)
        .collect();

    let slice_bz = slice_bx.clone();

    let slice_magnitude: Vec<f32> = slice_magnitude
        .iter()
        .map(|&m| (m * norm_factor) as f32)
        .collect();

    FieldData {
        plane_type: PlaneType::XZ,
        slice_width,
        slice_height,
        slice_bounds: [x_min, x_max, z_min, z_max],
        slice_offset: 0.0,
        slice_bx,
        slice_bz,
        slice_magnitude,
        arrows_positions: Vec::new(),
        arrows_vectors: Vec::new(),
        arrows_magnitudes: Vec::new(),
        line_z: Vec::new(),
        line_bz: Vec::new(),
    }
}
