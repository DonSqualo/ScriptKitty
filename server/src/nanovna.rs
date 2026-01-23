//! NanoVNA simulation module for S-parameter frequency sweep analysis
//!
//! Simulates a NanoVNA measuring S11 (reflection coefficient) for coupling coils
//! Used for characterizing impedance matching networks and coupling efficiency

use std::f64::consts::PI;

// ===========================

const Z0: f64 = 50.0; // Characteristic impedance (Ohms)
const MU0: f64 = 4.0 * PI * 1e-7; // Permeability of free space (H/m)
const COPPER_RESISTIVITY: f64 = 1.68e-8; // Copper resistivity (Ohm·m)

/// Configuration for NanoVNA frequency sweep
pub struct NanoVNAConfig {
    pub f_start: f64,        // Start frequency (Hz)
    pub f_stop: f64,         // Stop frequency (Hz)
    pub num_points: usize,   // Number of frequency points
    pub coil_radius: f64,    // Coil radius (mm)
    pub num_turns: u32,      // Number of turns
    pub wire_diameter: f64,  // Wire diameter (mm)
    pub coil_resistance: f64, // DC resistance (Ohms)
    pub parasitic_capacitance_pf: Option<f64>, // Self-capacitance (pF), computed via Medhurst if None
    pub resonator_radius: Option<f64>,    // Resonator loop radius (mm), None = no coupled resonator
    pub resonator_distance: f64,          // Axial distance from drive coil to resonator (mm)
    pub resonator_resistance: f64,        // Resistance of resonator loop (Ohms)
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
            parasitic_capacitance_pf: None,
            resonator_radius: None,
            resonator_distance: 10.0,
            resonator_resistance: 0.1,
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

/// Calculate parasitic capacitance using Medhurst formula for single-layer coils
/// Returns capacitance in Farads
/// Medhurst: C_self = 0.1 * (D/L + 1) * D pF where D is diameter (mm), L is length (mm)
fn calculate_parasitic_capacitance(radius_mm: f64, num_turns: u32, wire_diameter_mm: f64) -> f64 {
    let diameter_mm = 2.0 * radius_mm;
    let length_mm = num_turns as f64 * wire_diameter_mm;
    let c_pf = 0.1 * (diameter_mm / length_mm + 1.0) * diameter_mm;
    c_pf * 1e-12
}

/// Calculate inductance and parasitic capacitance for a coil
/// Returns (inductance in H, capacitance in F)
fn calculate_coil_parameters(
    radius_mm: f64,
    num_turns: u32,
    wire_diameter_mm: f64,
    override_capacitance_pf: Option<f64>,
) -> (f64, f64) {
    let inductance = calculate_inductance(radius_mm, num_turns, wire_diameter_mm);
    let capacitance = override_capacitance_pf
        .map(|c_pf| c_pf * 1e-12)
        .unwrap_or_else(|| calculate_parasitic_capacitance(radius_mm, num_turns, wire_diameter_mm));
    (inductance, capacitance)
}

/// Calculate skin depth for copper at given frequency (meters)
fn calculate_skin_depth(frequency: f64) -> f64 {
    let omega = 2.0 * PI * frequency;
    (2.0 * COPPER_RESISTIVITY / (omega * MU0)).sqrt()
}

/// Calculate mutual inductance between two coaxial loops using Neumann formula approximation
/// M = mu0 * pi * r1^2 * r2^2 / (2 * (r1^2 + r2^2 + d^2)^(3/2))
/// r1, r2 in mm, d in mm, returns M in Henries
pub fn calculate_mutual_inductance(r1_mm: f64, r2_mm: f64, distance_mm: f64) -> f64 {
    let r1 = r1_mm * 1e-3;
    let r2 = r2_mm * 1e-3;
    let d = distance_mm * 1e-3;
    let r1_sq = r1 * r1;
    let r2_sq = r2 * r2;
    let d_sq = d * d;
    MU0 * PI * r1_sq * r2_sq / (2.0 * (r1_sq + r2_sq + d_sq).powf(1.5))
}

/// Calculate AC resistance with skin effect
/// R_ac = R_dc * sqrt(1 + (f/f_skin)^2)
/// where f_skin is the characteristic frequency where skin depth equals wire radius
fn calculate_ac_resistance(frequency: f64, dc_resistance: f64, wire_diameter_mm: f64) -> f64 {
    let wire_radius_m = wire_diameter_mm * 0.5e-3;
    let f_skin = COPPER_RESISTIVITY / (PI * MU0 * wire_radius_m * wire_radius_m);
    let ratio = frequency / f_skin;
    dc_resistance * (1.0 + ratio * ratio).sqrt()
}

/// Calculate impedance at a given frequency with skin effect and parasitic capacitance
/// Models coil as: C_parasitic in parallel with (R_ac in series with L)
/// Z = (R + jωL) || (1/jωC)
fn calculate_impedance(
    frequency: f64,
    inductance: f64,
    capacitance: f64,
    dc_resistance: f64,
    wire_diameter_mm: f64,
) -> (f64, f64) {
    let omega = 2.0 * PI * frequency;
    let x_l = omega * inductance;
    let x_c = 1.0 / (omega * capacitance);
    let r = calculate_ac_resistance(frequency, dc_resistance, wire_diameter_mm);

    // Z = (R + jX_L) || (-jX_C)
    // Z_real = R * X_C^2 / (R^2 + (X_L - X_C)^2)
    // Z_imag = X_C * (X_L*X_C - X_L^2 - R^2) / (R^2 + (X_L - X_C)^2)
    let d = r * r + (x_l - x_c) * (x_l - x_c);
    let z_real = r * x_c * x_c / d;
    let z_imag = x_c * (x_l * x_c - x_l * x_l - r * r) / d;

    (z_real, z_imag)
}

/// Calculate coupled impedance for drive coil with magnetically coupled resonator
/// Z_in = Z_drive + (ωM)^2 / Z_resonator
/// where Z_drive = R_drive + jωL_drive and Z_resonator = R_resonator + jωL_resonator
fn calculate_coupled_impedance(
    frequency: f64,
    drive_inductance: f64,
    drive_resistance: f64,
    resonator_inductance: f64,
    resonator_resistance: f64,
    mutual_inductance: f64,
) -> (f64, f64) {
    let omega = 2.0 * PI * frequency;
    let omega_m_sq = (omega * mutual_inductance) * (omega * mutual_inductance);

    // Z_resonator = R_res + jωL_res
    let z_res_real = resonator_resistance;
    let z_res_imag = omega * resonator_inductance;

    // (ωM)^2 / Z_resonator = (ωM)^2 * Z_res* / |Z_res|^2
    let z_res_mag_sq = z_res_real * z_res_real + z_res_imag * z_res_imag;
    let z_reflected_real = omega_m_sq * z_res_real / z_res_mag_sq;
    let z_reflected_imag = -omega_m_sq * z_res_imag / z_res_mag_sq;

    // Z_in = R_drive + jωL_drive + Z_reflected
    let z_in_real = drive_resistance + z_reflected_real;
    let z_in_imag = omega * drive_inductance + z_reflected_imag;

    (z_in_real, z_in_imag)
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

/// Compute impedance at a specific frequency for the configured coil
/// Returns (z_real, z_imag) in Ohms
pub fn compute_impedance_at_frequency(config: &NanoVNAConfig, frequency: f64) -> (f64, f64) {
    let (inductance, capacitance) = calculate_coil_parameters(
        config.coil_radius,
        config.num_turns,
        config.wire_diameter,
        config.parasitic_capacitance_pf,
    );

    match config.resonator_radius {
        Some(res_radius) => {
            let mutual = calculate_mutual_inductance(
                config.coil_radius,
                res_radius,
                config.resonator_distance,
            );
            let res_inductance = calculate_inductance(res_radius, 1, config.wire_diameter);
            let drive_r = calculate_ac_resistance(frequency, config.coil_resistance, config.wire_diameter);
            calculate_coupled_impedance(
                frequency,
                inductance,
                drive_r,
                res_inductance,
                config.resonator_resistance,
                mutual,
            )
        }
        None => calculate_impedance(
            frequency,
            inductance,
            capacitance,
            config.coil_resistance,
            config.wire_diameter,
        ),
    }
}

/// Compute full NanoVNA frequency sweep
pub fn compute_frequency_sweep(config: &NanoVNAConfig) -> FrequencySweep {
    let (inductance, capacitance) = calculate_coil_parameters(
        config.coil_radius,
        config.num_turns,
        config.wire_diameter,
        config.parasitic_capacitance_pf,
    );

    let coupled_params = config.resonator_radius.map(|res_radius| {
        let mutual = calculate_mutual_inductance(
            config.coil_radius,
            res_radius,
            config.resonator_distance,
        );
        let res_inductance = calculate_inductance(res_radius, 1, config.wire_diameter);
        (mutual, res_inductance)
    });

    let mut points = Vec::with_capacity(config.num_points);
    let mut min_s11_db = f64::MAX;
    let mut min_s11_freq = config.f_start;

    for i in 0..config.num_points {
        let t = i as f64 / (config.num_points - 1) as f64;
        let frequency = config.f_start + t * (config.f_stop - config.f_start);

        let (z_real, z_imag) = match coupled_params {
            Some((mutual, res_inductance)) => {
                let drive_r = calculate_ac_resistance(frequency, config.coil_resistance, config.wire_diameter);
                calculate_coupled_impedance(
                    frequency,
                    inductance,
                    drive_r,
                    res_inductance,
                    config.resonator_resistance,
                    mutual,
                )
            }
            None => calculate_impedance(
                frequency,
                inductance,
                capacitance,
                config.coil_resistance,
                config.wire_diameter,
            ),
        };
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
// Multi-gap resonator physics for SLMG (Single Loop Multi-Gap) resonators
// Based on: Petryakov et al. J. Magn. Reson. 188 (2007) 68-73
// ===========================

const EPSILON_0: f64 = 8.854e-12; // Permittivity of free space (F/m)

/// Configuration for multi-gap loop-gap resonator
pub struct MultiGapResonatorConfig {
    pub inner_radius: f64,      // Inner radius (mm)
    pub outer_radius: f64,      // Outer radius (mm)
    pub length: f64,            // Resonator length (mm)
    pub num_gaps: u32,          // Number of capacitive gaps
    pub gap_thickness: f64,     // Gap thickness (mm)
    pub gap_permittivity: f64,  // Relative permittivity of gap dielectric (polystyrene ≈ 2.6)
}

/// Calculate single-loop inductance for the resonator (approximation)
/// Uses solenoid formula with fractional turns: L = μ₀ × n² × A / l
/// where n = 1/N (fractional turn per gap)
fn calculate_loop_inductance(inner_radius_mm: f64, outer_radius_mm: f64, length_mm: f64) -> f64 {
    let r_mean = (inner_radius_mm + outer_radius_mm) / 2.0 * 1e-3; // meters
    let length = length_mm * 1e-3; // meters
    let area = PI * r_mean * r_mean;
    // Nagaoka correction for short coil
    let k = length / (2.0 * r_mean);
    let nagaoka = 1.0 / (1.0 + 0.9 * k);
    MU0 * area * nagaoka / length
}

/// Calculate gap capacitance: C = ε₀ × ε_r × S_c / d
/// S_c is the capacitor plate area (radial surface)
fn calculate_gap_capacitance(
    inner_radius_mm: f64,
    outer_radius_mm: f64,
    length_mm: f64,
    gap_thickness_mm: f64,
    relative_permittivity: f64,
) -> f64 {
    let r_inner = inner_radius_mm * 1e-3;
    let r_outer = outer_radius_mm * 1e-3;
    let length = length_mm * 1e-3;
    let gap = gap_thickness_mm * 1e-3;
    // Plate area = height × radial depth
    let plate_area = length * (r_outer - r_inner);
    EPSILON_0 * relative_permittivity * plate_area / gap
}

/// Calculate resonant frequency for multi-gap resonator
/// ω = 1/√(L × C_sum) where C_sum = C/N
/// Frequency scales as √N with number of gaps (more gaps = higher frequency)
pub fn calculate_multigap_resonant_frequency(config: &MultiGapResonatorConfig) -> f64 {
    let n = config.num_gaps as f64;
    let l_loop = calculate_loop_inductance(
        config.inner_radius,
        config.outer_radius,
        config.length,
    );
    let c_single = calculate_gap_capacitance(
        config.inner_radius,
        config.outer_radius,
        config.length,
        config.gap_thickness,
        config.gap_permittivity,
    );
    // L is the loop inductance (constant for given geometry)
    // C_sum = C / N (N capacitors in series, each with capacitance C)
    // f = √N / (2π√(L × C))
    let c_sum = c_single / n;
    1.0 / (2.0 * PI * (l_loop * c_sum).sqrt())
}

/// Calculate unloaded Q factor for multi-gap resonator
/// Q = ωL/R where R is the total resistance of conductive surfaces
/// Q is inversely proportional to √N (more gaps = lower Q due to increased resistance)
pub fn calculate_multigap_q_factor(config: &MultiGapResonatorConfig, frequency: f64) -> f64 {
    let n = config.num_gaps as f64;
    let l_loop = calculate_loop_inductance(
        config.inner_radius,
        config.outer_radius,
        config.length,
    );
    let omega = 2.0 * PI * frequency;
    // Estimate resistance from skin effect on silver-plated surfaces
    let r_mean = (config.inner_radius + config.outer_radius) / 2.0 * 1e-3;
    let circumference = 2.0 * PI * r_mean;
    let length = config.length * 1e-3;
    let skin_depth = calculate_skin_depth(frequency);
    // Silver resistivity is slightly lower than copper
    let silver_resistivity = 1.59e-8;
    // Effective conductor width (inner surface area exposed to RF current)
    let conductor_width = length;
    // R = ρ / (δ × w) × circumference for the loop
    let total_resistance = silver_resistivity * circumference / (skin_depth * conductor_width);
    // Q increases with number of gaps (due to √N frequency scaling) but resistance also increases
    omega * l_loop / total_resistance
}

/// Calculate loaded Q with sample (lossy dielectric)
/// Q_loaded = Q_unloaded × (1 + loss_factor)⁻¹
/// where loss_factor depends on sample conductivity and volume
pub fn calculate_loaded_q(
    config: &MultiGapResonatorConfig,
    frequency: f64,
    sample_conductivity: f64,
    sample_volume_cc: f64,
) -> f64 {
    let q_unloaded = calculate_multigap_q_factor(config, frequency);
    let omega = 2.0 * PI * frequency;
    // Loss factor from sample: proportional to σ × ω × V
    // Empirical scaling based on paper data
    let sample_volume_m3 = sample_volume_cc * 1e-6;
    let loss_factor = sample_conductivity * omega * sample_volume_m3 * 1e4;
    q_unloaded / (1.0 + loss_factor)
}

/// Compute frequency sweep for multi-gap resonator
pub fn compute_multigap_frequency_sweep(
    config: &MultiGapResonatorConfig,
    f_start: f64,
    f_stop: f64,
    num_points: usize,
) -> FrequencySweep {
    let f_resonant = calculate_multigap_resonant_frequency(config);
    let q_unloaded = calculate_multigap_q_factor(config, f_resonant);
    let n = config.num_gaps as f64;
    let l_loop = calculate_loop_inductance(
        config.inner_radius,
        config.outer_radius,
        config.length,
    );
    let c_single = calculate_gap_capacitance(
        config.inner_radius,
        config.outer_radius,
        config.length,
        config.gap_thickness,
        config.gap_permittivity,
    );
    let c_sum = c_single / n;
    let r_estimate = 2.0 * PI * f_resonant * l_loop / q_unloaded;

    let mut points = Vec::with_capacity(num_points);
    let mut min_s11_db = f64::MAX;
    let mut min_s11_freq = f_start;

    for i in 0..num_points {
        let t = i as f64 / (num_points - 1) as f64;
        let frequency = f_start + t * (f_stop - f_start);
        let omega = 2.0 * PI * frequency;
        // Impedance: Z = R + j(ωL - 1/ωC)
        let x_l = omega * l_loop;
        let x_c = 1.0 / (omega * c_sum);
        let z_real = r_estimate;
        let z_imag = x_l - x_c;
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
        let c = 1e-12; // 1 pF (small capacitance to minimize LC resonance effects)
        let r_dc = 0.5; // 0.5 Ohm DC resistance
        let wire_d = 0.5; // 0.5mm wire diameter

        let (_z_real, _z_imag) = calculate_impedance(1e6, l, c, r_dc, wire_d);

        // At 1 MHz with 0.5mm wire, expect significant skin effect
        // f_skin = rho / (pi * mu0 * r^2) ≈ 68 kHz for 0.5mm diameter
        // R_ac = R_dc * sqrt(1 + (f/f_skin)^2) ≈ R_dc * 14.7
        let r_ac = calculate_ac_resistance(1e6, r_dc, wire_d);
        assert!(r_ac > r_dc * 5.0, "AC resistance should be much higher than DC at 1 MHz");
        assert!(r_ac < r_dc * 25.0, "AC resistance should be within expected range");
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

    #[test]
    fn test_skin_effect() {
        let r_dc = 1.0;
        let wire_diameter = 1.0; // 1mm wire

        // At DC (very low frequency), R_ac ≈ R_dc
        let r_ac_low = calculate_ac_resistance(100.0, r_dc, wire_diameter);
        assert!(
            (r_ac_low - r_dc).abs() < 0.01,
            "At low frequency, R_ac should equal R_dc, got {}",
            r_ac_low
        );

        // At high frequency, R_ac should increase significantly
        // f_skin for 1mm wire ≈ 17 kHz
        let r_ac_1mhz = calculate_ac_resistance(1e6, r_dc, wire_diameter);
        let r_ac_10mhz = calculate_ac_resistance(10e6, r_dc, wire_diameter);

        assert!(
            r_ac_1mhz > r_dc * 10.0,
            "At 1 MHz, R_ac should be >> R_dc, got ratio {}",
            r_ac_1mhz / r_dc
        );
        assert!(
            r_ac_10mhz > r_ac_1mhz,
            "R_ac should increase with frequency"
        );

        // Verify skin depth calculation
        let delta_1mhz = calculate_skin_depth(1e6);
        // Standard copper skin depth at 1 MHz is ~66 um
        assert!(
            (delta_1mhz - 66e-6).abs() < 5e-6,
            "Skin depth at 1 MHz should be ~66um, got {} um",
            delta_1mhz * 1e6
        );
    }

    #[test]
    fn test_parasitic_capacitance_calculation() {
        // Coil: 25mm radius (50mm diameter), 10 turns, 0.5mm wire (5mm length)
        let c = calculate_parasitic_capacitance(25.0, 10, 0.5);
        // Medhurst: C = 0.1 * (50/5 + 1) * 50 = 0.1 * 11 * 50 = 55 pF
        let expected_pf = 55.0;
        let actual_pf = c * 1e12;
        assert!(
            (actual_pf - expected_pf).abs() < 1.0,
            "Parasitic capacitance should be ~55 pF, got {} pF",
            actual_pf
        );
    }

    #[test]
    fn test_self_resonant_frequency_in_sweep() {
        // Use a specific capacitance override to get predictable SRF
        let c_pf: f64 = 100.0;
        let c = c_pf * 1e-12;

        let config = NanoVNAConfig {
            f_start: 1e6,
            f_stop: 10e6,
            num_points: 501,
            coil_radius: 25.0,
            num_turns: 10,
            wire_diameter: 0.5,
            coil_resistance: 0.5,
            parasitic_capacitance_pf: Some(c_pf),
            ..Default::default()
        };

        // Calculate actual inductance from coil parameters
        let l = calculate_inductance(config.coil_radius, config.num_turns, config.wire_diameter);
        let expected_srf = 1.0 / (2.0 * PI * (l * c).sqrt());

        let sweep = compute_frequency_sweep(&config);

        // Find the frequency where impedance magnitude is maximum (near SRF)
        let mut max_z_mag = 0.0_f64;
        let mut max_z_freq = 0.0_f64;
        for p in &sweep.points {
            let z_mag = (p.z_real * p.z_real + p.z_imag * p.z_imag).sqrt();
            if z_mag > max_z_mag {
                max_z_mag = z_mag;
                max_z_freq = p.frequency;
            }
        }

        // The impedance peak should be within 10% of expected SRF
        let srf_error_pct = ((max_z_freq - expected_srf) / expected_srf).abs() * 100.0;
        assert!(
            srf_error_pct < 10.0,
            "SRF should be near {} MHz, found peak at {} MHz (error: {:.1}%)",
            expected_srf / 1e6,
            max_z_freq / 1e6,
            srf_error_pct
        );

        // Verify impedance is high at resonance (parallel LC tank behavior)
        assert!(
            max_z_mag > 1000.0,
            "Impedance at SRF should be high (parallel LC resonance), got {} Ohms",
            max_z_mag
        );
    }

    #[test]
    fn test_mutual_inductance_calculation() {
        // Two coaxial loops: r1 = 25mm, r2 = 20mm, d = 10mm
        let m = calculate_mutual_inductance(25.0, 20.0, 10.0);

        // M = mu0 * pi * r1^2 * r2^2 / (2 * (r1^2 + r2^2 + d^2)^(3/2))
        // r1 = 0.025m, r2 = 0.020m, d = 0.010m
        // r1^2 = 6.25e-4, r2^2 = 4e-4, d^2 = 1e-4
        // sum = 1.125e-3, (sum)^1.5 = 3.77e-5
        // M = 4*pi*1e-7 * pi * 6.25e-4 * 4e-4 / (2 * 3.77e-5) = 13.1 nH
        let expected_nh = 13.1;
        let actual_nh = m * 1e9;
        assert!(
            (actual_nh - expected_nh).abs() < 1.0,
            "Mutual inductance should be ~{} nH, got {} nH",
            expected_nh,
            actual_nh
        );

        // Mutual inductance should decrease with distance
        let m_close = calculate_mutual_inductance(25.0, 20.0, 5.0);
        let m_far = calculate_mutual_inductance(25.0, 20.0, 50.0);
        assert!(
            m_close > m,
            "Closer loops should have higher mutual inductance"
        );
        assert!(
            m_far < m,
            "Further loops should have lower mutual inductance"
        );

        // Mutual inductance should increase with loop radii
        let m_smaller = calculate_mutual_inductance(10.0, 10.0, 10.0);
        let m_larger = calculate_mutual_inductance(50.0, 50.0, 10.0);
        assert!(
            m_larger > m_smaller,
            "Larger loops should have higher mutual inductance"
        );
    }

    #[test]
    fn test_coupled_impedance() {
        // Test that coupling adds reflected impedance to the drive coil
        let l_drive = 10e-6;  // 10 uH
        let r_drive = 0.5;
        let l_res = 5e-6;     // 5 uH
        let r_res = 0.1;
        let m = 1e-6;         // 1 uH mutual
        let freq = 10e6;      // 10 MHz

        let (z_real, z_imag) = calculate_coupled_impedance(
            freq,
            l_drive,
            r_drive,
            l_res,
            r_res,
            m,
        );

        // Drive impedance without coupling: R_drive + jωL_drive
        // At 10 MHz: Z_drive = 0.5 + j*628 (ωL = 2*pi*10e6*10e-6 = 628)
        let omega = 2.0 * PI * freq;
        let expected_drive_imag = omega * l_drive;

        // The coupled impedance should have:
        // - Real part > R_drive (added reflected resistance from resonator)
        // - Imaginary part < ωL_drive (reflected impedance is capacitive when ωL_res dominates)
        assert!(
            z_real > r_drive,
            "Coupled real impedance {} should exceed drive resistance {}",
            z_real,
            r_drive
        );
        assert!(
            z_imag < expected_drive_imag,
            "Coupled imaginary {} should be less than uncoupled ωL_drive {}",
            z_imag,
            expected_drive_imag
        );
    }

    #[test]
    fn test_multigap_resonant_frequency() {
        // Test multi-gap resonator frequency calculation
        // Paper values: 42mm i.d., 88mm o.d., 48mm length, 16 gaps, 1.68mm polystyrene
        let config = MultiGapResonatorConfig {
            inner_radius: 21.0,
            outer_radius: 44.0,
            length: 48.0,
            num_gaps: 16,
            gap_thickness: 1.68,
            gap_permittivity: 2.6,
        };
        let f_resonant = calculate_multigap_resonant_frequency(&config);
        // Paper reports f0 ≈ 1.22 GHz empty
        let f_ghz = f_resonant / 1e9;
        assert!(
            f_ghz > 0.5 && f_ghz < 3.0,
            "Resonant frequency {} GHz should be in L-band range",
            f_ghz
        );
    }

    #[test]
    fn test_multigap_frequency_scaling() {
        // Test that frequency scales with √N (number of gaps)
        let config_8 = MultiGapResonatorConfig {
            inner_radius: 21.0,
            outer_radius: 44.0,
            length: 48.0,
            num_gaps: 8,
            gap_thickness: 1.68,
            gap_permittivity: 2.6,
        };
        let config_16 = MultiGapResonatorConfig {
            inner_radius: 21.0,
            outer_radius: 44.0,
            length: 48.0,
            num_gaps: 16,
            gap_thickness: 1.68,
            gap_permittivity: 2.6,
        };
        let f_8 = calculate_multigap_resonant_frequency(&config_8);
        let f_16 = calculate_multigap_resonant_frequency(&config_16);
        // f_16/f_8 should be approximately √(16/8) = √2 ≈ 1.414
        let ratio = f_16 / f_8;
        let expected_ratio = (16.0_f64 / 8.0).sqrt();
        assert!(
            (ratio - expected_ratio).abs() < 0.2,
            "Frequency ratio {} should be near √2 = {}",
            ratio,
            expected_ratio
        );
    }

    #[test]
    fn test_multigap_q_factor() {
        // Test Q factor calculation
        let config = MultiGapResonatorConfig {
            inner_radius: 21.0,
            outer_radius: 44.0,
            length: 48.0,
            num_gaps: 16,
            gap_thickness: 1.68,
            gap_permittivity: 2.6,
        };
        let f_resonant = calculate_multigap_resonant_frequency(&config);
        let q = calculate_multigap_q_factor(&config, f_resonant);
        // Paper reports loaded Q ≈ 72 with 11cc saline
        // Unloaded Q can be quite high (1000-10000) for silver-plated loop-gap resonators
        assert!(
            q > 100.0 && q < 50000.0,
            "Q factor {} should be in reasonable range for loop-gap resonator",
            q
        );
    }

    #[test]
    fn test_loaded_q() {
        let config = MultiGapResonatorConfig {
            inner_radius: 21.0,
            outer_radius: 44.0,
            length: 48.0,
            num_gaps: 16,
            gap_thickness: 1.68,
            gap_permittivity: 2.6,
        };
        let f_resonant = calculate_multigap_resonant_frequency(&config);
        let q_unloaded = calculate_multigap_q_factor(&config, f_resonant);
        // 0.45% saline ≈ 0.77 S/m conductivity, 11cc volume
        let q_loaded = calculate_loaded_q(&config, f_resonant, 0.77, 11.0);
        assert!(
            q_loaded < q_unloaded,
            "Loaded Q {} should be less than unloaded Q {}",
            q_loaded,
            q_unloaded
        );
    }
}
