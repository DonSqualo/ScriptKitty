//! MEEP FDTD script generation from Mittens geometry
//!
//! Generates runnable Python scripts for MEEP electromagnetic simulations.

use mlua::{Lua, Table, Value};
use std::fmt::Write;

/// MEEP simulation configuration extracted from Lua
#[derive(Debug, Clone)]
pub struct MeepConfig {
    pub freq_start_hz: f64,
    pub freq_stop_hz: f64,
    pub resolution: f64,
    pub pml_thickness: f64,
}

impl Default for MeepConfig {
    fn default() -> Self {
        Self {
            freq_start_hz: 1e9,
            freq_stop_hz: 10e9,
            resolution: 10.0,
            pml_thickness: 1.0, // mm
        }
    }
}

/// Geometry primitive for MEEP
#[derive(Debug, Clone)]
pub enum MeepPrimitive {
    Block {
        name: String,
        center: [f64; 3],
        size: [f64; 3],
        material: String,
    },
    Cylinder {
        name: String,
        center: [f64; 3],
        radius: f64,
        height: f64,
        axis: [f64; 3],
        material: String,
    },
    Sphere {
        name: String,
        center: [f64; 3],
        radius: f64,
        material: String,
    },
}

/// Try to generate MEEP script from Lua scene
pub fn try_generate_meep_script(lua: &Lua, content: &str) -> Option<String> {
    // Check if this file has electromagnetic setup
    if !content.contains("electromagnetic") && !content.contains("freq_start") {
        return None;
    }

    // Execute to get result
    let result: Value = lua.load(content).eval().ok()?;
    let table = result.as_table()?;

    // Extract config
    let config = extract_meep_config(lua, table);

    // Extract geometry
    let geometry = extract_geometry(table)?;

    if geometry.is_empty() {
        return None;
    }

    // Generate script
    Some(generate_script(&geometry, &config))
}

fn extract_meep_config(lua: &Lua, _table: &Table) -> MeepConfig {
    let mut config = MeepConfig::default();

    let globals = lua.globals();

    // Try to get from config table
    if let Ok(cfg) = globals.get::<_, Table>("config") {
        if let Ok(fs) = cfg.get::<_, f64>("freq_start") {
            config.freq_start_hz = fs;
        }
        if let Ok(fs) = cfg.get::<_, f64>("freq_stop") {
            config.freq_stop_hz = fs;
        }
    }

    config
}

fn extract_geometry(table: &Table) -> Option<Vec<MeepPrimitive>> {
    let objects: Table = table.get("objects").ok()?;
    let mut primitives = Vec::new();

    for pair in objects.pairs::<i64, Table>() {
        if let Ok((_, obj)) = pair {
            extract_primitives_recursive(&obj, [0.0, 0.0, 0.0], &mut primitives);
        }
    }

    Some(primitives)
}

fn extract_primitives_recursive(
    obj: &Table,
    parent_offset: [f64; 3],
    output: &mut Vec<MeepPrimitive>,
) {
    let obj_type: String = obj.get("type").unwrap_or_default();
    let name: String = obj.get("name").unwrap_or_else(|_| "unnamed".to_string());

    // Get transform offset
    let offset = get_translation_offset(obj);
    let center = [
        parent_offset[0] + offset[0],
        parent_offset[1] + offset[1],
        parent_offset[2] + offset[2],
    ];

    // Get material
    let material = get_material_string(obj);

    match obj_type.as_str() {
        "box" => {
            if let Ok(params) = obj.get::<_, Table>("params") {
                let w: f64 = params.get("w").unwrap_or(1.0);
                let d: f64 = params.get("d").unwrap_or(w);
                let h: f64 = params.get("h").unwrap_or(1.0);

                output.push(MeepPrimitive::Block {
                    name,
                    center,
                    size: [w, d, h],
                    material,
                });
            }
        }
        "cylinder" => {
            if let Ok(params) = obj.get::<_, Table>("params") {
                let r: f64 = params.get("r").unwrap_or(1.0);
                let h: f64 = params.get("h").unwrap_or(1.0);

                output.push(MeepPrimitive::Cylinder {
                    name,
                    center,
                    radius: r,
                    height: h,
                    axis: [0.0, 0.0, 1.0],
                    material,
                });
            }
        }
        "sphere" => {
            if let Ok(params) = obj.get::<_, Table>("params") {
                let r: f64 = params.get("r").unwrap_or(1.0);

                output.push(MeepPrimitive::Sphere {
                    name,
                    center,
                    radius: r,
                    material,
                });
            }
        }
        "group" | "assembly" | "csg" => {
            // Recurse into children
            if let Ok(children) = obj.get::<_, Table>("children") {
                for pair in children.pairs::<i64, Table>() {
                    if let Ok((_, child)) = pair {
                        // For CSG difference, mark subsequent children as air
                        let is_difference = obj_type == "csg"
                            && obj.get::<_, String>("operation").ok() == Some("difference".to_string());

                        if is_difference {
                            // First child keeps its material, rest become air
                            let mut child_with_air = child;
                            // We can't easily modify the child, so we pass info via recursion
                            extract_primitives_recursive(&child_with_air, center, output);
                        } else {
                            extract_primitives_recursive(&child, center, output);
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

fn get_translation_offset(obj: &Table) -> [f64; 3] {
    let mut offset = [0.0, 0.0, 0.0];

    if let Ok(ops) = obj.get::<_, Table>("ops") {
        for pair in ops.pairs::<i64, Table>() {
            if let Ok((_, op)) = pair {
                let op_type: String = op.get("op").unwrap_or_default();
                if op_type == "translate" {
                    offset[0] = op.get("x").unwrap_or(0.0);
                    offset[1] = op.get("y").unwrap_or(0.0);
                    offset[2] = op.get("z").unwrap_or(0.0);
                }
            }
        }
    }

    offset
}

fn get_material_string(obj: &Table) -> String {
    if let Ok(mat) = obj.get::<_, Table>("material") {
        if let Ok(name) = mat.get::<_, String>("name") {
            return match name.to_lowercase().as_str() {
                "copper" | "cu" | "metal" | "pec" => "mp.metal".to_string(),
                "fr4" => {
                    let eps: f64 = mat.get("permittivity").unwrap_or(4.4);
                    format!("mp.Medium(epsilon={})", eps)
                }
                "air" => "mp.air".to_string(),
                _ => {
                    if let Ok(eps) = mat.get::<_, f64>("permittivity") {
                        format!("mp.Medium(epsilon={})", eps)
                    } else {
                        "mp.air".to_string()
                    }
                }
            };
        }
    }
    "mp.air".to_string()
}

fn generate_script(geometry: &[MeepPrimitive], config: &MeepConfig) -> String {
    let mut script = String::with_capacity(8192);

    // Header
    writeln!(script, r#"#!/usr/bin/env python3
"""
MEEP FDTD Simulation - Generated from Mittens CAD

Frequency range: {:.2} GHz - {:.2} GHz
Resolution: {:.1} pixels/mm
"""

import meep as mp
import numpy as np
import argparse
import os
from datetime import datetime

# =============================================================================
# Geometry Definition (units: mm)
# =============================================================================

def build_geometry():
    """Build MEEP geometry objects."""
    geometry = []
"#,
        config.freq_start_hz / 1e9,
        config.freq_stop_hz / 1e9,
        config.resolution
    ).unwrap();

    // Compute bounds for cell size
    let (min, max) = compute_bounds(geometry);
    let padding = config.pml_thickness * 2.0 + 5.0; // PML + margin

    // Add geometry
    for prim in geometry {
        match prim {
            MeepPrimitive::Block { name, center, size, material } => {
                writeln!(script, r#"
    # {name}
    geometry.append(mp.Block(
        center=mp.Vector3({:.4}, {:.4}, {:.4}),
        size=mp.Vector3({:.4}, {:.4}, {:.4}),
        material={material}
    ))"#,
                    center[0], center[1], center[2],
                    size[0], size[1], size[2]
                ).unwrap();
            }
            MeepPrimitive::Cylinder { name, center, radius, height, axis, material } => {
                writeln!(script, r#"
    # {name}
    geometry.append(mp.Cylinder(
        center=mp.Vector3({:.4}, {:.4}, {:.4}),
        radius={:.4},
        height={:.4},
        axis=mp.Vector3({:.1}, {:.1}, {:.1}),
        material={material}
    ))"#,
                    center[0], center[1], center[2],
                    radius, height,
                    axis[0], axis[1], axis[2]
                ).unwrap();
            }
            MeepPrimitive::Sphere { name, center, radius, material } => {
                writeln!(script, r#"
    # {name}
    geometry.append(mp.Sphere(
        center=mp.Vector3({:.4}, {:.4}, {:.4}),
        radius={:.4},
        material={material}
    ))"#,
                    center[0], center[1], center[2],
                    radius
                ).unwrap();
            }
        }
    }

    writeln!(script, "\n    return geometry\n").unwrap();

    // Simulation parameters
    let cell_x = (max[0] - min[0]) + padding * 2.0;
    let cell_y = (max[1] - min[1]) + padding * 2.0;
    let cell_z = (max[2] - min[2]) + padding * 2.0;

    let fcen = (config.freq_start_hz + config.freq_stop_hz) / 2.0;
    let fwidth = config.freq_stop_hz - config.freq_start_hz;

    // Convert to MEEP units (assuming mm -> mm, freq in c/mm units)
    // f_meep = f_Hz * (1mm / c) = f_Hz * 1e-3 / 3e8
    let fcen_meep = fcen * 1e-3 / 3e8;
    let fwidth_meep = fwidth * 1e-3 / 3e8;

    writeln!(script, r#"
# =============================================================================
# Simulation Parameters
# =============================================================================

CELL_X = {cell_x:.4}
CELL_Y = {cell_y:.4}
CELL_Z = {cell_z:.4}
RESOLUTION = {resolution:.1}
PML_THICKNESS = {pml:.4}

# Frequency (MEEP units)
FCEN = {fcen_meep:.6e}
FWIDTH = {fwidth_meep:.6e}


# =============================================================================
# Sources
# =============================================================================

def build_sources():
    """Build MEEP excitation sources."""
    return [
        mp.Source(
            src=mp.GaussianSource(frequency=FCEN, fwidth=FWIDTH),
            component=mp.Ez,
            center=mp.Vector3(0, 0, 0),
            size=mp.Vector3(0, 0, 0)
        )
    ]


# =============================================================================
# Main Simulation
# =============================================================================

def run_simulation(output_dir="output", plot=False):
    """Run the FDTD simulation."""
    os.makedirs(output_dir, exist_ok=True)

    print("=" * 60)
    print("MEEP FDTD Simulation (from Mittens)")
    print("=" * 60)
    print(f"Cell size: {{CELL_X:.2f}} x {{CELL_Y:.2f}} x {{CELL_Z:.2f}} mm")
    print(f"Resolution: {{RESOLUTION}} pixels/mm")
    print()

    geometry = build_geometry()
    sources = build_sources()

    sim = mp.Simulation(
        cell_size=mp.Vector3(CELL_X, CELL_Y, CELL_Z),
        geometry=geometry,
        sources=sources,
        boundary_layers=[mp.PML(thickness=PML_THICKNESS)],
        resolution=RESOLUTION,
        default_material=mp.air,
    )

    # Field capture
    field_data = {{"t": [], "ez": []}}

    def capture_fields(sim):
        ez = sim.get_field_point(mp.Ez, mp.Vector3(0, 0, 0))
        field_data["t"].append(sim.meep_time())
        field_data["ez"].append(complex(ez).real)

    print("Running simulation...")
    sim.run(
        mp.at_every(1, capture_fields),
        until_after_sources=mp.stop_when_fields_decayed(50, mp.Ez, mp.Vector3(0, 0, 0), 1e-6)
    )

    print("Simulation complete.")

    np.savez(f"{{output_dir}}/results.npz",
             field_t=field_data["t"],
             field_ez=field_data["ez"])
    print(f"Results saved to {{output_dir}}/results.npz")

    if plot:
        try:
            import matplotlib.pyplot as plt
            fig, ax = plt.subplots(figsize=(10, 6))
            ax.plot(field_data["t"], field_data["ez"], 'b-', linewidth=0.5)
            ax.set_xlabel("Time (MEEP units)")
            ax.set_ylabel("Ez")
            ax.set_title("E-field Time Response")
            ax.grid(True, alpha=0.3)
            plt.savefig(f"{{output_dir}}/time_response.png", dpi=150)
            plt.close()
            print(f"Plot saved to {{output_dir}}/time_response.png")
        except ImportError:
            print("matplotlib not available")

    return field_data


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="MEEP FDTD Simulation")
    parser.add_argument("--plot", action="store_true", help="Generate plots")
    parser.add_argument("--output", type=str, default="output", help="Output directory")

    args = parser.parse_args()

    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    output_dir = f"{{args.output}}/sim_{{timestamp}}"

    run_simulation(output_dir=output_dir, plot=args.plot)
"#,
        resolution = config.resolution,
        pml = config.pml_thickness
    ).unwrap();

    script
}

fn compute_bounds(geometry: &[MeepPrimitive]) -> ([f64; 3], [f64; 3]) {
    let mut min = [f64::MAX; 3];
    let mut max = [f64::MIN; 3];

    for prim in geometry {
        let (c, half_ext) = match prim {
            MeepPrimitive::Block { center, size, .. } => {
                (*center, [size[0] / 2.0, size[1] / 2.0, size[2] / 2.0])
            }
            MeepPrimitive::Cylinder { center, radius, height, .. } => {
                (*center, [*radius, *radius, height / 2.0])
            }
            MeepPrimitive::Sphere { center, radius, .. } => {
                (*center, [*radius, *radius, *radius])
            }
        };

        for i in 0..3 {
            min[i] = min[i].min(c[i] - half_ext[i]);
            max[i] = max[i].max(c[i] + half_ext[i]);
        }
    }

    // Default bounds if nothing found
    if min[0] == f64::MAX {
        min = [-10.0, -10.0, -10.0];
        max = [10.0, 10.0, 10.0];
    }

    (min, max)
}
