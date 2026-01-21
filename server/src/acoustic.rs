//! Acoustic field computation using Rayleigh-Sommerfeld diffraction integral
//!
//! Computes pressure field from circular piston transducers with coverslip reflection

use std::f64::consts::PI;

use crate::field::{Colormap, FieldData, PlaneType};

// ===========================

const Z_WATER: f64 = 1.48e6; // Acoustic impedance of water (Rayl)
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

/// Compute acoustic pressure at a specific point for Hydrophone measurements
/// Returns (real, imag) complex pressure components
pub fn compute_pressure_at_point(r: f64, z: f64, config: &AcousticConfig) -> (f64, f64) {
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

    (
        total_real * config.drive_amplitude,
        total_imag * config.drive_amplitude,
    )
}

// ===========================

pub fn compute_acoustic_field(
    config: &AcousticConfig,
    plane: PlaneType,
    plane_offset: f64,
    colormap: Colormap,
) -> FieldData {
    let slice_width = 80;
    let slice_height = 80;

    // Bounds depend on plane type
    let (axis1_min, axis1_max, axis2_min, axis2_max) = match plane {
        PlaneType::XZ => (-config.medium_radius, config.medium_radius, 0.0, config.medium_height),
        PlaneType::XY => (-config.medium_radius, config.medium_radius, -config.medium_radius, config.medium_radius),
        PlaneType::YZ => (-config.medium_radius, config.medium_radius, 0.0, config.medium_height),
    };

    let mut slice_pressure_real = Vec::with_capacity(slice_width * slice_height);
    let mut slice_pressure_imag = Vec::with_capacity(slice_width * slice_height);
    let mut slice_magnitude = Vec::with_capacity(slice_width * slice_height);

    let mut max_magnitude: f64 = 0.0;

    for j in 0..slice_height {
        let axis2 = axis2_min + (j as f64 + 0.5) * (axis2_max - axis2_min) / slice_height as f64;
        for i in 0..slice_width {
            let axis1 = axis1_min + (i as f64 + 0.5) * (axis1_max - axis1_min) / slice_width as f64;

            // Map axes to cylindrical coordinates (r, z) based on plane type
            let (r, z) = match plane {
                PlaneType::XZ => (axis1.abs(), axis2), // x maps to r, axis2 is z
                PlaneType::XY => {
                    // XY plane at fixed z = plane_offset
                    let rho = (axis1 * axis1 + axis2 * axis2).sqrt();
                    (rho, plane_offset)
                }
                PlaneType::YZ => (axis1.abs(), axis2), // y maps to r, axis2 is z
            };

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
        plane_type: plane,
        colormap,
        slice_width,
        slice_height,
        slice_bounds: [axis1_min, axis1_max, axis2_min, axis2_max],
        slice_offset: plane_offset,
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

// ===========================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acoustic_impedance_values() {
        assert!(
            (Z_WATER - 1.48e6).abs() < 1e3,
            "Water impedance should be ~1.48 MRayl"
        );
        assert!(
            (Z_GLASS - 12.6e6).abs() < 1e4,
            "Glass impedance should be ~12.6 MRayl"
        );
        assert!(
            Z_GLASS > Z_WATER,
            "Glass impedance should be higher than water"
        );
    }

    #[test]
    fn test_reflection_coefficient_calculation() {
        let expected = (Z_GLASS - Z_WATER) / (Z_GLASS + Z_WATER);
        assert!(
            (REFLECTION_COEFF - expected).abs() < 1e-10,
            "Reflection coefficient mismatch"
        );
        assert!(
            REFLECTION_COEFF > 0.0 && REFLECTION_COEFF < 1.0,
            "Reflection coefficient should be in (0,1)"
        );
        assert!(
            (REFLECTION_COEFF - 0.79).abs() < 0.01,
            "Reflection coefficient should be ~0.79"
        );
    }

    #[test]
    fn test_pressure_at_single_point_on_axis() {
        let config = AcousticConfig::default();
        let r = 0.0;
        let z = config.transducer_z / 2.0;
        let (p_real, p_imag) = compute_pressure_at_point(r, z, &config);
        let magnitude = (p_real * p_real + p_imag * p_imag).sqrt();
        assert!(
            magnitude > 0.0,
            "Pressure magnitude should be positive on-axis"
        );
        assert!(magnitude.is_finite(), "Pressure should be finite");
    }

    #[test]
    fn test_pressure_varies_with_height() {
        let config = AcousticConfig::default();
        let r = 0.0;

        let z1 = 1.0;
        let z2 = 2.5;
        let z3 = 4.0;

        let (p1_real, p1_imag) = compute_pressure_at_point(r, z1, &config);
        let (p2_real, p2_imag) = compute_pressure_at_point(r, z2, &config);
        let (p3_real, p3_imag) = compute_pressure_at_point(r, z3, &config);

        let mag1 = (p1_real * p1_real + p1_imag * p1_imag).sqrt();
        let mag2 = (p2_real * p2_real + p2_imag * p2_imag).sqrt();
        let mag3 = (p3_real * p3_real + p3_imag * p3_imag).sqrt();

        // Due to standing wave patterns from coverslip reflection,
        // pressure doesn't monotonically decrease - verify all are finite and positive
        assert!(mag1 > 0.0 && mag1.is_finite(), "Pressure at z=1 should be positive and finite");
        assert!(mag2 > 0.0 && mag2.is_finite(), "Pressure at z=2.5 should be positive and finite");
        assert!(mag3 > 0.0 && mag3.is_finite(), "Pressure at z=4 should be positive and finite");

        // Verify there is variation (not all identical)
        let avg = (mag1 + mag2 + mag3) / 3.0;
        let variance = ((mag1 - avg).powi(2) + (mag2 - avg).powi(2) + (mag3 - avg).powi(2)) / 3.0;
        assert!(variance > 0.0, "Pressure should vary with height due to interference patterns");
    }

    #[test]
    fn test_rayleigh_piston_symmetry() {
        let k = 2.0 * PI * 1e6 / (1480.0 * 1000.0);
        let piston_z = 5.0;
        let piston_radius = 6.0;
        let n_rings = 12;
        let n_segments = 24;

        let (p_pos_real, p_pos_imag) =
            rayleigh_piston(3.0, 2.0, piston_z, piston_radius, k, n_rings, n_segments);
        let (p_neg_real, p_neg_imag) =
            rayleigh_piston(-3.0, 2.0, piston_z, piston_radius, k, n_rings, n_segments);

        let mag_pos = (p_pos_real * p_pos_real + p_pos_imag * p_pos_imag).sqrt();
        let mag_neg = (p_neg_real * p_neg_real + p_neg_imag * p_neg_imag).sqrt();

        assert!(
            (mag_pos - mag_neg).abs() / mag_pos < 0.01,
            "Pressure should be symmetric about axis"
        );
    }

    #[test]
    fn test_acoustic_field_generation() {
        let config = AcousticConfig::default();
        let field = compute_acoustic_field(&config, PlaneType::XZ, 0.0, Colormap::Jet);

        assert_eq!(field.slice_width, 80);
        assert_eq!(field.slice_height, 80);
        assert_eq!(field.slice_bx.len(), 80 * 80);
        assert_eq!(field.slice_magnitude.len(), 80 * 80);

        let max_mag = field.slice_magnitude.iter().cloned().fold(0.0f32, f32::max);
        assert!(
            (max_mag - 1.0).abs() < 1e-5,
            "Magnitude should be normalized to 1.0"
        );
    }

    #[test]
    fn test_acoustic_field_xy_plane() {
        let config = AcousticConfig::default();
        let field = compute_acoustic_field(&config, PlaneType::XY, 2.0, Colormap::Viridis);

        assert_eq!(field.plane_type, PlaneType::XY);
        assert_eq!(field.colormap, Colormap::Viridis);
        assert_eq!(field.slice_offset, 2.0);
        assert_eq!(field.slice_width, 80);
        assert_eq!(field.slice_height, 80);
    }
}
