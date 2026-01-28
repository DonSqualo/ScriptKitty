//! meep-export: Translate Mittens CAD geometry to MEEP FDTD simulations
//!
//! This crate provides:
//! - Parsing of Mittens serialized geometry (JSON)
//! - Translation to MEEP geometry primitives
//! - Python script generation for MEEP simulations
//!
//! MEEP uses normalized units by default. This translator preserves
//! physical units (µm) and handles the conversion factors.

pub mod geometry;
pub mod material;
pub mod meep;
pub mod codegen;

pub use geometry::{MittensScene, GeometryObject, Primitive, Transform, CsgOperation};
pub use material::{Material, EmProperties};
pub use meep::{MeepGeometry, MeepSource, MeepSimulation};
pub use codegen::generate_meep_script;

use anyhow::Result;

/// Main entry point: parse Mittens JSON and generate MEEP Python script
pub fn translate(json: &str, config: &TranslationConfig) -> Result<String> {
    let scene: MittensScene = serde_json::from_str(json)?;
    let simulation = MeepSimulation::from_mittens(&scene, config)?;
    let script = generate_meep_script(&simulation, config)?;
    Ok(script)
}

/// Configuration for the translation process
#[derive(Debug, Clone)]
pub struct TranslationConfig {
    /// Length unit in the source geometry (default: mm)
    pub source_unit: LengthUnit,
    /// MEEP length unit (default: µm)
    pub meep_unit: LengthUnit,
    /// Resolution in pixels per MEEP length unit
    pub resolution: f64,
    /// PML thickness in MEEP units
    pub pml_thickness: f64,
    /// Center frequency in Hz
    pub freq_center_hz: f64,
    /// Frequency width in Hz (for Gaussian pulse)
    pub freq_width_hz: f64,
    /// Simulation cell padding beyond geometry bounds
    pub cell_padding: f64,
    /// Include field monitors
    pub field_monitors: bool,
    /// Include flux monitors for S-parameters
    pub flux_monitors: bool,
    /// Circular segment count for curved primitives
    pub circular_segments: u32,
}

impl Default for TranslationConfig {
    fn default() -> Self {
        Self {
            source_unit: LengthUnit::Millimeter,
            meep_unit: LengthUnit::Micrometer,
            resolution: 10.0,
            pml_thickness: 1000.0, // 1mm in µm
            freq_center_hz: 5e9,   // 5 GHz
            freq_width_hz: 4e9,    // 4 GHz bandwidth
            cell_padding: 2000.0,  // 2mm in µm
            field_monitors: true,
            flux_monitors: true,
            circular_segments: 32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LengthUnit {
    Meter,
    Millimeter,
    Micrometer,
    Nanometer,
}

impl LengthUnit {
    /// Convert from this unit to meters
    pub fn to_meters(&self, value: f64) -> f64 {
        match self {
            LengthUnit::Meter => value,
            LengthUnit::Millimeter => value * 1e-3,
            LengthUnit::Micrometer => value * 1e-6,
            LengthUnit::Nanometer => value * 1e-9,
        }
    }

    /// Convert from meters to this unit
    pub fn from_meters(&self, value: f64) -> f64 {
        match self {
            LengthUnit::Meter => value,
            LengthUnit::Millimeter => value * 1e3,
            LengthUnit::Micrometer => value * 1e6,
            LengthUnit::Nanometer => value * 1e9,
        }
    }

    /// Get scale factor to convert from one unit to another
    pub fn scale_to(&self, target: &LengthUnit) -> f64 {
        target.from_meters(self.to_meters(1.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_conversion() {
        assert!((LengthUnit::Millimeter.scale_to(&LengthUnit::Micrometer) - 1000.0).abs() < 1e-10);
        assert!((LengthUnit::Meter.scale_to(&LengthUnit::Millimeter) - 1000.0).abs() < 1e-10);
    }
}
