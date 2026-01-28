//! Geometry types matching Mittens serialization format

use serde::{Deserialize, Serialize};
use nalgebra::{Matrix4, Vector3};

/// Top-level Mittens scene
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MittensScene {
    /// List of geometry objects
    pub objects: Vec<GeometryObject>,
    /// Optional view configuration (ignored for MEEP)
    #[serde(default)]
    pub view: Option<serde_json::Value>,
    /// Optional physics setup
    #[serde(default)]
    pub physics: Option<PhysicsSetup>,
    /// Export definitions (ignored, we generate our own)
    #[serde(default)]
    pub exports: Vec<serde_json::Value>,
}

/// Physics simulation setup from Mittens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsSetup {
    #[serde(rename = "type")]
    pub physics_type: Option<String>,
    pub frequencies: Option<Vec<f64>>,
    pub solver: Option<String>,
    pub formulation: Option<String>,
}

/// A geometry object (primitive, CSG, or group)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeometryObject {
    /// Object type: "box", "cylinder", "sphere", "csg", "group", etc.
    #[serde(rename = "type")]
    pub obj_type: String,
    /// Object name (optional)
    #[serde(default)]
    pub name: String,
    /// Primitive parameters (for primitive types)
    #[serde(default)]
    pub params: Option<PrimitiveParams>,
    /// Transform operations
    #[serde(default)]
    pub ops: Vec<Transform>,
    /// Material assignment
    #[serde(default)]
    pub material: Option<MaterialRef>,
    /// CSG operation type (for CSG objects)
    #[serde(default)]
    pub operation: Option<String>,
    /// Children (for CSG and group objects)
    #[serde(default)]
    pub children: Vec<GeometryObject>,
}

/// Parameters for primitive shapes
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PrimitiveParams {
    // Box
    pub w: Option<f64>,  // width (x)
    pub d: Option<f64>,  // depth (y)
    pub h: Option<f64>,  // height (z)

    // Cylinder / Sphere
    pub r: Option<f64>,  // radius

    // Ring (annulus)
    pub inner_radius: Option<f64>,
    pub outer_radius: Option<f64>,

    // Torus
    pub major_radius: Option<f64>,
    pub minor_radius: Option<f64>,

    // Helix (for coils)
    pub turns: Option<f64>,
    pub pitch: Option<f64>,
    pub wire_diameter: Option<f64>,
}

/// Transform operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transform {
    pub op: String,
    #[serde(default)]
    pub x: f64,
    #[serde(default)]
    pub y: f64,
    #[serde(default)]
    pub z: f64,
}

/// Material reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialRef {
    pub name: Option<String>,
    pub permittivity: Option<f64>,
    pub permeability: Option<f64>,
    pub conductivity: Option<f64>,
    pub loss_tangent: Option<f64>,
    #[serde(default)]
    pub color: Option<[f64; 3]>,
}

/// Parsed primitive types
#[derive(Debug, Clone)]
pub enum Primitive {
    Box { width: f64, depth: f64, height: f64 },
    Cylinder { radius: f64, height: f64 },
    Sphere { radius: f64 },
    Ring { inner_radius: f64, outer_radius: f64, height: f64 },
    Torus { major_radius: f64, minor_radius: f64 },
}

/// CSG operation types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CsgOperation {
    Union,
    Difference,
    Intersect,
}

impl GeometryObject {
    /// Check if this is a primitive type
    pub fn is_primitive(&self) -> bool {
        matches!(self.obj_type.as_str(), "box" | "cylinder" | "sphere" | "ring" | "torus")
    }

    /// Check if this is a CSG operation
    pub fn is_csg(&self) -> bool {
        self.obj_type == "csg"
    }

    /// Check if this is a group/assembly
    pub fn is_group(&self) -> bool {
        matches!(self.obj_type.as_str(), "group" | "assembly" | "component")
    }

    /// Parse as a primitive (if applicable)
    pub fn as_primitive(&self) -> Option<Primitive> {
        let params = self.params.as_ref()?;
        match self.obj_type.as_str() {
            "box" => Some(Primitive::Box {
                width: params.w.unwrap_or(1.0),
                depth: params.d.unwrap_or(params.w.unwrap_or(1.0)),
                height: params.h.unwrap_or(1.0),
            }),
            "cylinder" => Some(Primitive::Cylinder {
                radius: params.r.unwrap_or(1.0),
                height: params.h.unwrap_or(1.0),
            }),
            "sphere" => Some(Primitive::Sphere {
                radius: params.r.unwrap_or(1.0),
            }),
            "ring" => Some(Primitive::Ring {
                inner_radius: params.inner_radius.unwrap_or(0.5),
                outer_radius: params.outer_radius.unwrap_or(1.0),
                height: params.h.unwrap_or(1.0),
            }),
            "torus" => Some(Primitive::Torus {
                major_radius: params.major_radius.unwrap_or(1.0),
                minor_radius: params.minor_radius.unwrap_or(0.25),
            }),
            _ => None,
        }
    }

    /// Parse CSG operation type
    pub fn csg_operation(&self) -> Option<CsgOperation> {
        let op = self.operation.as_ref()?;
        match op.as_str() {
            "union" => Some(CsgOperation::Union),
            "difference" => Some(CsgOperation::Difference),
            "intersect" => Some(CsgOperation::Intersect),
            _ => None,
        }
    }

    /// Compute the 4x4 transform matrix from ops
    pub fn transform_matrix(&self) -> Matrix4<f64> {
        let mut matrix = Matrix4::identity();

        for op in &self.ops {
            let op_matrix = match op.op.as_str() {
                "translate" => Matrix4::new_translation(&Vector3::new(op.x, op.y, op.z)),
                "rotate" => {
                    // ZYX Euler angles in degrees
                    let rx = op.x.to_radians();
                    let ry = op.y.to_radians();
                    let rz = op.z.to_radians();

                    let rot_x = Matrix4::from_euler_angles(rx, 0.0, 0.0);
                    let rot_y = Matrix4::from_euler_angles(0.0, ry, 0.0);
                    let rot_z = Matrix4::from_euler_angles(0.0, 0.0, rz);

                    rot_z * rot_y * rot_x
                }
                "scale" => Matrix4::new_nonuniform_scaling(&Vector3::new(op.x, op.y, op.z)),
                _ => Matrix4::identity(),
            };
            matrix = matrix * op_matrix;
        }

        matrix
    }

    /// Get the center position from transform ops
    pub fn center(&self) -> Vector3<f64> {
        let matrix = self.transform_matrix();
        Vector3::new(matrix[(0, 3)], matrix[(1, 3)], matrix[(2, 3)])
    }

    /// Compute axis-aligned bounding box (min, max)
    pub fn aabb(&self) -> Option<(Vector3<f64>, Vector3<f64>)> {
        let primitive = self.as_primitive()?;
        let center = self.center();

        // Compute half-extents for each primitive type
        let half_extents = match primitive {
            Primitive::Box { width, depth, height } => {
                Vector3::new(width / 2.0, depth / 2.0, height / 2.0)
            }
            Primitive::Cylinder { radius, height } => {
                Vector3::new(radius, radius, height / 2.0)
            }
            Primitive::Sphere { radius } => {
                Vector3::new(radius, radius, radius)
            }
            Primitive::Ring { outer_radius, height, .. } => {
                Vector3::new(outer_radius, outer_radius, height / 2.0)
            }
            Primitive::Torus { major_radius, minor_radius } => {
                let r = major_radius + minor_radius;
                Vector3::new(r, r, minor_radius)
            }
        };

        // Note: This is simplified and doesn't account for rotation
        // For accurate bounds with rotation, we'd need to transform all 8 corners
        Some((center - half_extents, center + half_extents))
    }
}

/// Recursively compute the bounding box of a scene
pub fn scene_aabb(scene: &MittensScene) -> Option<(Vector3<f64>, Vector3<f64>)> {
    let mut min = Vector3::new(f64::MAX, f64::MAX, f64::MAX);
    let mut max = Vector3::new(f64::MIN, f64::MIN, f64::MIN);
    let mut found_any = false;

    fn visit_object(obj: &GeometryObject, min: &mut Vector3<f64>, max: &mut Vector3<f64>, found: &mut bool) {
        if let Some((obj_min, obj_max)) = obj.aabb() {
            *found = true;
            min.x = min.x.min(obj_min.x);
            min.y = min.y.min(obj_min.y);
            min.z = min.z.min(obj_min.z);
            max.x = max.x.max(obj_max.x);
            max.y = max.y.max(obj_max.y);
            max.z = max.z.max(obj_max.z);
        }

        // Recurse into children
        for child in &obj.children {
            visit_object(child, min, max, found);
        }
    }

    for obj in &scene.objects {
        visit_object(obj, &mut min, &mut max, &mut found_any);
    }

    if found_any {
        Some((min, max))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_box() {
        let json = r#"{
            "type": "box",
            "name": "test_box",
            "params": {"w": 10.0, "d": 5.0, "h": 2.0},
            "ops": [{"op": "translate", "x": 1.0, "y": 2.0, "z": 3.0}]
        }"#;

        let obj: GeometryObject = serde_json::from_str(json).unwrap();
        assert_eq!(obj.obj_type, "box");
        assert!(obj.is_primitive());

        let prim = obj.as_primitive().unwrap();
        match prim {
            Primitive::Box { width, depth, height } => {
                assert_eq!(width, 10.0);
                assert_eq!(depth, 5.0);
                assert_eq!(height, 2.0);
            }
            _ => panic!("Expected Box"),
        }

        let center = obj.center();
        assert!((center.x - 1.0).abs() < 1e-10);
        assert!((center.y - 2.0).abs() < 1e-10);
        assert!((center.z - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_parse_csg() {
        let json = r#"{
            "type": "csg",
            "operation": "difference",
            "children": [
                {"type": "cylinder", "params": {"r": 10.0, "h": 5.0}, "ops": []},
                {"type": "cylinder", "params": {"r": 8.0, "h": 6.0}, "ops": []}
            ],
            "ops": []
        }"#;

        let obj: GeometryObject = serde_json::from_str(json).unwrap();
        assert!(obj.is_csg());
        assert_eq!(obj.csg_operation(), Some(CsgOperation::Difference));
        assert_eq!(obj.children.len(), 2);
    }
}
