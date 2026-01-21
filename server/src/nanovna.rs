//! NanoVNA simulation module for S-parameter frequency sweep analysis
//!
//! Simulates a NanoVNA measuring S11 (reflection coefficient) for coupling coils
//! Used for characterizing impedance matching networks and coupling efficiency

use std::f64::consts::PI;

// ===========================

const Z0: f64 = 50.0; // Characteristic impedance (Ohms)
const MU0: f64 = 4.0 * PI * 1e-7; // Permeability of free space (H/m)

/// Configuration for NanoVNA frequency sweep
pub struct NanoVNAConfig {
    pub f_start: f64,        // Start frequency (Hz)
    pub f_stop: f64,         // Stop frequency (Hz)
    pub num_points: usize,   // Number of frequency points
    pub coil_radius: f64,    // Coil radius (mm)
    pub num_turns: u32,      // Number of turns
    pub wire_diameter: f64,  // Wire diameter (mm)
    pub coil_resistance: f64, // DC resistance (Ohms)
}

impl Default for NanoVNAConfig {
    fn default() -> Self {
        Self {
            f_start: 1e6,
            f_stop: 50e6,
            num_points: 101,
            coil_radius: 25.0,
            num_turns: 10,
            wire_diameter: 0.5,
            coil_resistance: 0.5,
        }
    }
}

/// S11 measurement result at a single frequency
#[derive(Clone, Debug)]
pub struct S11Point {
    pub frequency: f64,
    pub magnitude_db: f64,
    pub phase_deg: f64,
    pub z_real: f64,
    pub z_imag: f64,
}

/// Full frequency sweep result
pub struct FrequencySweep {
    pub points: Vec<S11Point>,
    pub min_s11_db: f64,
    pub min_s11_freq: f64,
}

impl FrequencySweep {
    pub fn to_binary(&self) -> Vec<u8> {
        let mut data = Vec::new();

        // Header
        data.extend_from_slice(b"NANOVNA\0");

        // Number of points
        data.extend_from_slice(&(self.points.len() as u32).to_le_bytes());

        // Min S11 info
        data.extend_from_slice(&(self.min_s11_db as f32).to_le_bytes());
        data.extend_from_slice(&(self.min_s11_freq as f32).to_le_bytes());

        // Frequency array
        for p in &self.points {
            data.extend_from_slice(&(p.frequency as f32).to_le_bytes());
        }

        // S11 magnitude (dB)
        for p in &self.points {
            data.extend_from_slice(&(p.magnitude_db as f32).to_le_bytes());
        }

        // S11 phase (degrees)
        for p in &self.points {
            data.extend_from_slice(&(p.phase_deg as f32).to_le_bytes());
        }

        // Impedance real
        for p in &self.points {
            data.extend_from_slice(&(p.z_real as f32).to_le_bytes());
        }

        // Impedance imaginary
        for p in &self.points {
            data.extend_from_slice(&(p.z_imag as f32).to_le_bytes());
        }

        data
    }
}

// ===========================

/// Calculate inductance of a single-layer solenoid coil (Wheeler formula)
/// Returns inductance in Henries
fn calculate_inductance(radius_mm: f64, num_turns: u32, wire_diameter_mm: f64) -> f64 {
    let radius_m = radius_mm * 1e-3;
    let length_m = num_turns as f64 * wire_diameter_mm * 1e-3;

    // Wheeler formula for single-layer coil: L = (mu0 * N^2 * A) / length
    // With Nagaoka correction for short coils
    let n = num_turns as f64;
    let a = PI * radius_m * radius_m;

    // Nagaoka correction factor (approximation for short coils)
    let k = length_m / (2.0 * radius_m);
    let nagaoka = 1.0 / (1.0 + 0.9 * k);

    MU0 * n * n * a * nagaoka / length_m
}

/// Calculate impedance at a given frequency
fn calculate_impedance(frequency: f64, inductance: f64, resistance: f64) -> (f64, f64) {
    let omega = 2.0 * PI * frequency;
    let x_l = omega * inductance; // Inductive reactance

    (resistance, x_l)
}

/// Calculate S11 from impedance
fn calculate_s11(z_real: f64, z_imag: f64) -> (f64, f64) {
    // S11 = (Z - Z0) / (Z + Z0)
    let num_real = z_real - Z0;
    let num_imag = z_imag;
    let den_real = z_real + Z0;
    let den_imag = z_imag;

    // Complex division
    let den_mag_sq = den_real * den_real + den_imag * den_imag;
    let s11_real = (num_real * den_real + num_imag * den_imag) / den_mag_sq;
    let s11_imag = (num_imag * den_real - num_real * den_imag) / den_mag_sq;

    // Convert to magnitude (dB) and phase (degrees)
    let magnitude = (s11_real * s11_real + s11_imag * s11_imag).sqrt();
    let magnitude_db = 20.0 * magnitude.log10();
    let phase_rad = s11_imag.atan2(s11_real);
    let phase_deg = phase_rad.to_degrees();

    (magnitude_db, phase_deg)
}

// ===========================

/// Compute full NanoVNA frequency sweep
pub fn compute_frequency_sweep(config: &NanoVNAConfig) -> FrequencySweep {
    let inductance = calculate_inductance(
        config.coil_radius,
        config.num_turns,
        config.wire_diameter,
    );

    let mut points = Vec::with_capacity(config.num_points);
    let mut min_s11_db = f64::MAX;
    let mut min_s11_freq = config.f_start;

    // Linear frequency sweep
    for i in 0..config.num_points {
        let t = i as f64 / (config.num_points - 1) as f64;
        let frequency = config.f_start + t * (config.f_stop - config.f_start);

        let (z_real, z_imag) = calculate_impedance(frequency, inductance, config.coil_resistance);
        let (magnitude_db, phase_deg) = calculate_s11(z_real, z_imag);

        if magnitude_db < min_s11_db {
            min_s11_db = magnitude_db;
            min_s11_freq = frequency;
        }

        points.push(S11Point {
            frequency,
            magnitude_db,
            phase_deg,
            z_real,
            z_imag,
        });
    }

    FrequencySweep {
        points,
        min_s11_db,
        min_s11_freq,
    }
}

// ===========================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inductance_calculation() {
        // Known coil: 25mm radius, 10 turns, 0.5mm wire
        let l = calculate_inductance(25.0, 10, 0.5);

        // Expected ~8uH for this configuration
        assert!(l > 1e-6 && l < 50e-6, "Inductance {:.2e} H out of expected range", l);
    }

    #[test]
    fn test_impedance_at_frequency() {
        let l = 10e-6; // 10 uH
        let r = 0.5;   // 0.5 Ohm

        let (z_real, z_imag) = calculate_impedance(1e6, l, r);

        assert!((z_real - 0.5).abs() < 0.01, "Resistance should be 0.5 Ohm");
        // X_L = 2*pi*1e6*10e-6 = 62.8 Ohm
        assert!((z_imag - 62.83).abs() < 0.1, "Reactance should be ~62.8 Ohm");
    }

    #[test]
    fn test_s11_matched_load() {
        // 50 Ohm purely resistive load should give S11 = 0 (perfect match)
        let (mag_db, _phase) = calculate_s11(50.0, 0.0);
        assert!(mag_db < -40.0, "Matched load should have very low S11");
    }

    #[test]
    fn test_s11_open_circuit() {
        // Very high impedance should give S11 ≈ 0 dB
        let (mag_db, _phase) = calculate_s11(1e6, 0.0);
        assert!(mag_db > -1.0, "Open circuit should have S11 near 0 dB");
    }

    #[test]
    fn test_s11_short_circuit() {
        // Very low impedance should give S11 ≈ 0 dB (but with phase shift)
        let (mag_db, _phase) = calculate_s11(0.001, 0.0);
        assert!(mag_db > -1.0, "Short circuit should have S11 near 0 dB");
    }

    #[test]
    fn test_frequency_sweep_structure() {
        let config = NanoVNAConfig {
            f_start: 1e6,
            f_stop: 10e6,
            num_points: 11,
            ..Default::default()
        };

        let sweep = compute_frequency_sweep(&config);

        assert_eq!(sweep.points.len(), 11);
        assert!((sweep.points[0].frequency - 1e6).abs() < 1.0);
        assert!((sweep.points[10].frequency - 10e6).abs() < 1.0);
    }

    #[test]
    fn test_frequency_sweep_to_binary() {
        let config = NanoVNAConfig {
            num_points: 5,
            ..Default::default()
        };

        let sweep = compute_frequency_sweep(&config);
        let binary = sweep.to_binary();

        // Check header
        assert_eq!(&binary[0..8], b"NANOVNA\0");

        // Check point count
        let count = u32::from_le_bytes([binary[8], binary[9], binary[10], binary[11]]);
        assert_eq!(count, 5);
    }
}
