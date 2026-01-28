//! MEEP simulation representation and geometry translation

use anyhow::{anyhow, Result};
use nalgebra::Vector3;
use crate::TranslationConfig;
use crate::geometry::{MittensScene, GeometryObject, Primitive, CsgOperation, scene_aabb};
use crate::material::{Material, EmProperties};

/// A MEEP simulation setup
#[derive(Debug, Clone)]
pub struct MeepSimulation {
    /// Cell size in MEEP units
    pub cell_size: Vector3<f64>,
    /// Resolution (pixels per length unit)
    pub resolution: f64,
    /// PML thickness
    pub pml_thickness: f64,
    /// Geometry objects
    pub geometry: Vec<MeepGeometry>,
    /// Excitation sources
    pub sources: Vec<MeepSource>,
    /// Field monitors
    pub monitors: Vec<MeepMonitor>,
    /// Flux monitors (for S-parameters)
    pub flux_monitors: Vec<MeepFluxMonitor>,
    /// Center frequency (MEEP units)
    pub fcen: f64,
    /// Frequency width (MEEP units)
    pub fwidth: f64,
    /// Length unit scale factor (source -> MEEP)
    pub unit_scale: f64,
}

/// MEEP geometry object
#[derive(Debug, Clone)]
pub struct MeepGeometry {
    pub name: String,
    pub primitive: MeepPrimitive,
    pub center: Vector3<f64>,
    pub material: String, // MEEP Python expression
}

/// MEEP primitive types (subset supported by MEEP)
#[derive(Debug, Clone)]
pub enum MeepPrimitive {
    Block { size: Vector3<f64> },
    Cylinder { radius: f64, height: f64, axis: Vector3<f64> },
    Sphere { radius: f64 },
    // CSG operations result in multiple primitives
    // MEEP doesn't have native CSG, so we approximate or warn
}

/// MEEP source definition
#[derive(Debug, Clone)]
pub struct MeepSource {
    pub source_type: MeepSourceType,
    pub component: String, // "Ez", "Hy", etc.
    pub center: Vector3<f64>,
    pub size: Vector3<f64>,
}

#[derive(Debug, Clone)]
pub enum MeepSourceType {
    GaussianSource { fcen: f64, fwidth: f64 },
    ContinuousSource { frequency: f64 },
}

/// Field monitor
#[derive(Debug, Clone)]
pub struct MeepMonitor {
    pub name: String,
    pub component: String,
    pub center: Vector3<f64>,
}

/// Flux monitor for S-parameter extraction
#[derive(Debug, Clone)]
pub struct MeepFluxMonitor {
    pub name: String,
    pub center: Vector3<f64>,
    pub size: Vector3<f64>,
    pub direction: i32, // +1 or -1
}

impl MeepSimulation {
    /// Create a MEEP simulation from a Mittens scene
    pub fn from_mittens(scene: &MittensScene, config: &TranslationConfig) -> Result<Self> {
        let unit_scale = config.source_unit.scale_to(&config.meep_unit);

        // Calculate cell size from geometry bounds
        let (min, max) = scene_aabb(scene)
            .ok_or_else(|| anyhow!("No geometry found in scene"))?;

        // Scale to MEEP units
        let min = min * unit_scale;
        let max = max * unit_scale;

        // Add padding and PML
        let padding = config.cell_padding + config.pml_thickness;
        let cell_size = Vector3::new(
            (max.x - min.x) + 2.0 * padding,
            (max.y - min.y) + 2.0 * padding,
            (max.z - min.z) + 2.0 * padding,
        );

        // Convert frequency to MEEP units
        // f_meep = f_Hz * (length_unit_in_m / c)
        let length_unit_m = config.meep_unit.to_meters(1.0);
        let c = 299792458.0; // m/s
        let fcen = config.freq_center_hz * length_unit_m / c;
        let fwidth = config.freq_width_hz * length_unit_m / c;

        // Translate geometry
        let mut geometry = Vec::new();
        for obj in &scene.objects {
            translate_object(obj, unit_scale, &mut geometry)?;
        }

        // Create default source (can be customized)
        let center = (min + max) / 2.0;
        let sources = vec![MeepSource {
            source_type: MeepSourceType::GaussianSource { fcen, fwidth },
            component: "Ez".to_string(),
            center,
            size: Vector3::new(0.0, 0.0, 0.0), // Point source by default
        }];

        // Create monitors
        let monitors = if config.field_monitors {
            vec![MeepMonitor {
                name: "E_center".to_string(),
                component: "Ez".to_string(),
                center,
            }]
        } else {
            vec![]
        };

        // Create flux monitors
        let flux_monitors = if config.flux_monitors {
            vec![
                MeepFluxMonitor {
                    name: "flux_x_pos".to_string(),
                    center: Vector3::new(max.x + config.pml_thickness / 2.0, center.y, center.z),
                    size: Vector3::new(0.0, max.y - min.y, max.z - min.z),
                    direction: 1,
                },
                MeepFluxMonitor {
                    name: "flux_x_neg".to_string(),
                    center: Vector3::new(min.x - config.pml_thickness / 2.0, center.y, center.z),
                    size: Vector3::new(0.0, max.y - min.y, max.z - min.z),
                    direction: -1,
                },
            ]
        } else {
            vec![]
        };

        Ok(Self {
            cell_size,
            resolution: config.resolution,
            pml_thickness: config.pml_thickness,
            geometry,
            sources,
            monitors,
            flux_monitors,
            fcen,
            fwidth,
            unit_scale,
        })
    }
}

/// Recursively translate a Mittens geometry object to MEEP geometry
fn translate_object(
    obj: &GeometryObject,
    unit_scale: f64,
    output: &mut Vec<MeepGeometry>,
) -> Result<()> {
    if obj.is_primitive() {
        if let Some(prim) = obj.as_primitive() {
            let center = obj.center() * unit_scale;
            let material = obj.material.as_ref()
                .map(|m| Material::from_ref(m))
                .unwrap_or_else(|| Material {
                    name: "air".to_string(),
                    properties: EmProperties::air(),
                });

            let meep_prim = match prim {
                Primitive::Box { width, depth, height } => MeepPrimitive::Block {
                    size: Vector3::new(width, depth, height) * unit_scale,
                },
                Primitive::Cylinder { radius, height } => MeepPrimitive::Cylinder {
                    radius: radius * unit_scale,
                    height: height * unit_scale,
                    axis: Vector3::new(0.0, 0.0, 1.0), // Default Z axis
                },
                Primitive::Sphere { radius } => MeepPrimitive::Sphere {
                    radius: radius * unit_scale,
                },
                Primitive::Ring { inner_radius: _, outer_radius, height } => {
                    // MEEP doesn't have native rings - create as two cylinders (outer - inner)
                    // The caller will need to handle this specially or use material overlapping
                    // For now, emit the outer cylinder with a comment
                    // TODO: Proper ring support via overlapping geometry
                    tracing::warn!("Ring primitive approximated as solid cylinder in MEEP");
                    MeepPrimitive::Cylinder {
                        radius: outer_radius * unit_scale,
                        height: height * unit_scale,
                        axis: Vector3::new(0.0, 0.0, 1.0),
                    }
                }
                Primitive::Torus { .. } => {
                    // MEEP doesn't support tori - would need to mesh or use geometric function
                    tracing::warn!("Torus primitive not directly supported in MEEP, skipping");
                    return Ok(());
                }
            };

            output.push(MeepGeometry {
                name: obj.name.clone(),
                primitive: meep_prim,
                center,
                material: material.to_meep_python(),
            });
        }
    } else if obj.is_csg() {
        // CSG operations in MEEP are handled by overlapping geometry
        // Later geometry takes precedence
        // For union: just emit both children
        // For difference: emit first, then second with air material
        // For intersection: complex, may need custom approach

        let op = obj.csg_operation();

        match op {
            Some(CsgOperation::Union) => {
                // Just emit all children
                for child in &obj.children {
                    translate_object(child, unit_scale, output)?;
                }
            }
            Some(CsgOperation::Difference) => {
                // Emit first child, then emit rest as air (to cut)
                if let Some(first) = obj.children.first() {
                    translate_object(first, unit_scale, output)?;
                }
                for child in obj.children.iter().skip(1) {
                    // Clone and override material to air
                    let mut air_child = child.clone();
                    air_child.material = Some(crate::geometry::MaterialRef {
                        name: Some("air".to_string()),
                        permittivity: None,
                        permeability: None,
                        conductivity: None,
                        loss_tangent: None,
                        color: None,
                    });
                    translate_object(&air_child, unit_scale, output)?;
                }
            }
            Some(CsgOperation::Intersect) => {
                tracing::warn!("CSG intersection not directly supported in MEEP, emitting first child only");
                if let Some(first) = obj.children.first() {
                    translate_object(first, unit_scale, output)?;
                }
            }
            None => {
                tracing::warn!("Unknown CSG operation, skipping");
            }
        }
    } else if obj.is_group() {
        // Groups/assemblies: just recurse into children
        for child in &obj.children {
            translate_object(child, unit_scale, output)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frequency_conversion() {
        // At 5 GHz with µm units:
        // f_meep = 5e9 * 1e-6 / 3e8 = 5e9 * 1e-6 / 3e8 = 0.0167
        let f_hz = 5e9;
        let length_unit_m = 1e-6; // µm
        let c = 299792458.0;
        let f_meep = f_hz * length_unit_m / c;

        assert!((f_meep - 0.0167).abs() < 0.001);
    }

    #[test]
    fn test_unit_scaling() {
        // mm to µm should be 1000x
        let scale = LengthUnit::Millimeter.scale_to(&LengthUnit::Micrometer);
        assert!((scale - 1000.0).abs() < 1e-10);
    }
}
