//! Electromagnetic material properties and MEEP material mapping

use serde::{Deserialize, Serialize};
use crate::geometry::MaterialRef;

/// Electromagnetic properties for MEEP materials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmProperties {
    /// Relative permittivity (epsilon_r)
    pub epsilon: f64,
    /// Relative permeability (mu_r)
    pub mu: f64,
    /// Electric conductivity (S/m)
    pub conductivity: f64,
    /// Magnetic conductivity (Ohm/m) - rarely used
    pub magnetic_conductivity: f64,
    /// Loss tangent (tan δ) - alternative to conductivity
    pub loss_tangent: Option<f64>,
}

impl Default for EmProperties {
    fn default() -> Self {
        Self {
            epsilon: 1.0,
            mu: 1.0,
            conductivity: 0.0,
            magnetic_conductivity: 0.0,
            loss_tangent: None,
        }
    }
}

impl EmProperties {
    /// Air / vacuum
    pub fn air() -> Self {
        Self::default()
    }

    /// Perfect electric conductor (PEC)
    /// In MEEP, this is represented as mp.metal or infinite conductivity
    pub fn pec() -> Self {
        Self {
            epsilon: 1.0,
            mu: 1.0,
            conductivity: f64::INFINITY,
            magnetic_conductivity: 0.0,
            loss_tangent: None,
        }
    }

    /// Copper at DC (σ = 5.96×10⁷ S/m)
    /// At GHz frequencies, skin depth is ~µm scale, so effectively PEC for mm-scale geometry
    pub fn copper() -> Self {
        Self {
            epsilon: 1.0,
            mu: 1.0,
            conductivity: 5.96e7,
            magnetic_conductivity: 0.0,
            loss_tangent: None,
        }
    }

    /// FR4 PCB substrate (typical values)
    pub fn fr4() -> Self {
        Self {
            epsilon: 4.4,
            mu: 1.0,
            conductivity: 0.0,
            magnetic_conductivity: 0.0,
            loss_tangent: Some(0.02),
        }
    }

    /// Check if this material should be treated as PEC in MEEP
    pub fn is_pec(&self) -> bool {
        self.conductivity.is_infinite() || self.conductivity > 1e6
    }

    /// Convert loss tangent to conductivity at a given frequency
    /// σ = ω * ε₀ * ε_r * tan(δ)
    pub fn conductivity_from_loss_tangent(&self, freq_hz: f64) -> f64 {
        if let Some(tan_d) = self.loss_tangent {
            let omega = 2.0 * std::f64::consts::PI * freq_hz;
            let eps_0 = 8.854e-12; // F/m
            omega * eps_0 * self.epsilon * tan_d
        } else {
            self.conductivity
        }
    }
}

/// High-level material with name and properties
#[derive(Debug, Clone)]
pub struct Material {
    pub name: String,
    pub properties: EmProperties,
}

impl Material {
    /// Create from a Mittens MaterialRef
    pub fn from_ref(mat_ref: &MaterialRef) -> Self {
        let name = mat_ref.name.clone().unwrap_or_else(|| "custom".to_string());

        // Check for known material names first
        let properties = match name.to_lowercase().as_str() {
            "copper" | "cu" => EmProperties::copper(),
            "metal" | "pec" => EmProperties::pec(),
            "fr4" => EmProperties::fr4(),
            "air" => EmProperties::air(),
            _ => {
                // Use provided properties or defaults
                EmProperties {
                    epsilon: mat_ref.permittivity.unwrap_or(1.0),
                    mu: mat_ref.permeability.unwrap_or(1.0),
                    conductivity: mat_ref.conductivity.unwrap_or(0.0),
                    magnetic_conductivity: 0.0,
                    loss_tangent: mat_ref.loss_tangent,
                }
            }
        };

        Self { name, properties }
    }

    /// Generate MEEP Python code for this material
    pub fn to_meep_python(&self) -> String {
        if self.properties.is_pec() {
            "mp.metal".to_string()
        } else if self.properties.epsilon == 1.0 
            && self.properties.conductivity == 0.0 
            && self.properties.loss_tangent.is_none() 
        {
            "mp.air".to_string()
        } else {
            let mut args = vec![format!("epsilon={}", self.properties.epsilon)];

            if self.properties.mu != 1.0 {
                args.push(format!("mu={}", self.properties.mu));
            }

            if self.properties.conductivity > 0.0 {
                // MEEP uses D_conductivity for electric conductivity
                // D_conductivity = σ / (2π) in MEEP's normalized units
                args.push(format!("D_conductivity={:.6e}", self.properties.conductivity));
            }

            format!("mp.Medium({})", args.join(", "))
        }
    }
}

/// Material library with common materials
pub struct MaterialLibrary {
    materials: std::collections::HashMap<String, Material>,
}

impl MaterialLibrary {
    pub fn new() -> Self {
        let mut lib = Self {
            materials: std::collections::HashMap::new(),
        };

        // Pre-populate with common materials
        lib.add("air", EmProperties::air());
        lib.add("copper", EmProperties::copper());
        lib.add("pec", EmProperties::pec());
        lib.add("metal", EmProperties::pec());
        lib.add("fr4", EmProperties::fr4());

        lib
    }

    pub fn add(&mut self, name: &str, properties: EmProperties) {
        self.materials.insert(name.to_lowercase(), Material {
            name: name.to_string(),
            properties,
        });
    }

    pub fn get(&self, name: &str) -> Option<&Material> {
        self.materials.get(&name.to_lowercase())
    }
}

impl Default for MaterialLibrary {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copper_is_pec() {
        let copper = EmProperties::copper();
        assert!(copper.is_pec(), "Copper should be treated as PEC at RF frequencies");
    }

    #[test]
    fn test_material_to_meep() {
        let pec = Material {
            name: "pec".to_string(),
            properties: EmProperties::pec(),
        };
        assert_eq!(pec.to_meep_python(), "mp.metal");

        let fr4 = Material {
            name: "fr4".to_string(),
            properties: EmProperties::fr4(),
        };
        let code = fr4.to_meep_python();
        assert!(code.contains("epsilon=4.4"));
    }

    #[test]
    fn test_loss_tangent_to_conductivity() {
        let fr4 = EmProperties::fr4();
        let sigma = fr4.conductivity_from_loss_tangent(5e9); // 5 GHz
        // σ = 2π * 5e9 * 8.854e-12 * 4.4 * 0.02 ≈ 0.0245 S/m
        assert!((sigma - 0.0245).abs() < 0.001);
    }
}
