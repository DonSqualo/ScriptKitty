//! Python code generation for MEEP simulations

use anyhow::Result;
use minijinja::{Environment, context};
use crate::{TranslationConfig, MeepSimulation};
use crate::meep::{MeepPrimitive, MeepSourceType};

const MEEP_TEMPLATE: &str = r##"#!/usr/bin/env python3
"""
MEEP FDTD Simulation - Auto-generated from Mittens CAD
{{ header_comment }}

Generated: {{ timestamp }}
Source unit: {{ source_unit }}
MEEP unit: {{ meep_unit }}
"""

import meep as mp
import numpy as np
import argparse
import os
from datetime import datetime

# =============================================================================
# Simulation Parameters (MEEP normalized units)
# =============================================================================

# Cell size
CELL_X = {{ "%.6f"|format(cell_x) }}
CELL_Y = {{ "%.6f"|format(cell_y) }}
CELL_Z = {{ "%.6f"|format(cell_z) }}

# Resolution (pixels per length unit)
RESOLUTION = {{ "%.1f"|format(resolution) }}

# PML thickness
PML_THICKNESS = {{ "%.6f"|format(pml_thickness) }}

# Frequency (MEEP units: f_meep = f_Hz * length_unit / c)
FCEN = {{ "%.6e"|format(fcen) }}
FWIDTH = {{ "%.6e"|format(fwidth) }}

# Unit scale: multiply source units by this to get MEEP units
UNIT_SCALE = {{ "%.6f"|format(unit_scale) }}


# =============================================================================
# Geometry Definition
# =============================================================================

def build_geometry():
    """Build MEEP geometry objects."""
    geometry = []
{% for geom in geometry %}

    # {{ geom.name }}
{% if geom.type == "block" %}
    geometry.append(mp.Block(
        center=mp.Vector3({{ "%.6f, %.6f, %.6f"|format(geom.cx, geom.cy, geom.cz) }}),
        size=mp.Vector3({{ "%.6f, %.6f, %.6f"|format(geom.sx, geom.sy, geom.sz) }}),
        material={{ geom.material }}
    ))
{% elif geom.type == "cylinder" %}
    geometry.append(mp.Cylinder(
        center=mp.Vector3({{ "%.6f, %.6f, %.6f"|format(geom.cx, geom.cy, geom.cz) }}),
        radius={{ "%.6f"|format(geom.radius) }},
        height={{ "%.6f"|format(geom.height) }},
        axis=mp.Vector3({{ "%.1f, %.1f, %.1f"|format(geom.ax, geom.ay, geom.az) }}),
        material={{ geom.material }}
    ))
{% elif geom.type == "sphere" %}
    geometry.append(mp.Sphere(
        center=mp.Vector3({{ "%.6f, %.6f, %.6f"|format(geom.cx, geom.cy, geom.cz) }}),
        radius={{ "%.6f"|format(geom.radius) }},
        material={{ geom.material }}
    ))
{% endif %}
{% endfor %}

    return geometry


# =============================================================================
# Sources
# =============================================================================

def build_sources():
    """Build MEEP excitation sources."""
    sources = []
{% for src in sources %}

{% if src.type == "gaussian" %}
    sources.append(mp.Source(
        src=mp.GaussianSource(frequency={{ "%.6e"|format(src.fcen) }}, fwidth={{ "%.6e"|format(src.fwidth) }}),
        component=mp.{{ src.component }},
        center=mp.Vector3({{ "%.6f, %.6f, %.6f"|format(src.cx, src.cy, src.cz) }}),
        size=mp.Vector3({{ "%.6f, %.6f, %.6f"|format(src.sx, src.sy, src.sz) }})
    ))
{% elif src.type == "continuous" %}
    sources.append(mp.Source(
        src=mp.ContinuousSource(frequency={{ "%.6e"|format(src.frequency) }}),
        component=mp.{{ src.component }},
        center=mp.Vector3({{ "%.6f, %.6f, %.6f"|format(src.cx, src.cy, src.cz) }}),
        size=mp.Vector3({{ "%.6f, %.6f, %.6f"|format(src.sx, src.sy, src.sz) }})
    ))
{% endif %}
{% endfor %}

    return sources


# =============================================================================
# Monitors and Flux Regions
# =============================================================================
{% if monitors|length > 0 %}

def get_field_monitors():
    """Return list of (name, component, position) for field monitoring."""
    return [
{% for mon in monitors %}
        ("{{ mon.name }}", mp.{{ mon.component }}, mp.Vector3({{ "%.6f, %.6f, %.6f"|format(mon.cx, mon.cy, mon.cz) }})),
{% endfor %}
    ]
{% endif %}
{% if flux_monitors|length > 0 %}

def build_flux_monitors(sim, fcen, fwidth, nfreq=50):
    """Add flux monitors to simulation for S-parameter extraction."""
    monitors = {}
{% for flux in flux_monitors %}
    monitors["{{ flux.name }}"] = sim.add_flux(
        fcen, fwidth, nfreq,
        mp.FluxRegion(
            center=mp.Vector3({{ "%.6f, %.6f, %.6f"|format(flux.cx, flux.cy, flux.cz) }}),
            size=mp.Vector3({{ "%.6f, %.6f, %.6f"|format(flux.sx, flux.sy, flux.sz) }}),
            direction=mp.X,
            weight={{ flux.direction }}
        )
    )
{% endfor %}
    return monitors
{% endif %}


# =============================================================================
# Main Simulation
# =============================================================================

def run_simulation(output_dir="output", plot=False):
    """Run the FDTD simulation."""

    os.makedirs(output_dir, exist_ok=True)

    print("=" * 60)
    print("MEEP FDTD Simulation")
    print("=" * 60)
    print(f"Cell size: {CELL_X:.2f} x {CELL_Y:.2f} x {CELL_Z:.2f}")
    print(f"Resolution: {RESOLUTION} pixels/unit")
    print(f"Frequency: {FCEN:.4e} (MEEP units)")
    print()

    # Build simulation
    geometry = build_geometry()
    sources = build_sources()

    cell_size = mp.Vector3(CELL_X, CELL_Y, CELL_Z)
    pml_layers = [mp.PML(thickness=PML_THICKNESS)]

    sim = mp.Simulation(
        cell_size=cell_size,
        geometry=geometry,
        sources=sources,
        boundary_layers=pml_layers,
        resolution=RESOLUTION,
        default_material=mp.air,
    )

{% if flux_monitors|length > 0 %}
    # Add flux monitors
    flux_monitors = build_flux_monitors(sim, FCEN, FWIDTH)
{% endif %}

    # Field data collection
    field_data = {"t": [], "ez": []}

    def capture_fields(sim):
        """Capture field at center."""
        center = mp.Vector3(0, 0, 0)
        ez = sim.get_field_point(mp.Ez, center)
        field_data["t"].append(sim.meep_time())
        field_data["ez"].append(complex(ez).real)

    print("Running simulation...")
    sim.run(
        mp.at_every(1, capture_fields),
        until_after_sources=mp.stop_when_fields_decayed(
            50, mp.Ez, mp.Vector3(0, 0, 0), 1e-6
        )
    )

    print("Simulation complete.")

{% if flux_monitors|length > 0 %}
    # Extract flux data
    results = {
        "field_t": field_data["t"],
        "field_ez": field_data["ez"],
    }

    for name, flux in flux_monitors.items():
        freqs = mp.get_flux_freqs(flux)
        flux_data = mp.get_fluxes(flux)
        results[f"{name}_freqs"] = freqs
        results[f"{name}_flux"] = flux_data

    # Save results
    np.savez(f"{output_dir}/results.npz", **results)
    print(f"Results saved to {output_dir}/results.npz")
{% else %}
    # Save time-domain results
    np.savez(f"{output_dir}/results.npz",
             field_t=field_data["t"],
             field_ez=field_data["ez"])
    print(f"Results saved to {output_dir}/results.npz")
{% endif %}

    if plot:
        plot_results(field_data, output_dir)

    return field_data


def plot_results(field_data, output_dir):
    """Generate plots from simulation data."""
    try:
        import matplotlib.pyplot as plt
    except ImportError:
        print("matplotlib not available, skipping plots")
        return

    fig, ax = plt.subplots(figsize=(10, 6))

    ax.plot(field_data["t"], field_data["ez"], 'b-', linewidth=0.5)
    ax.set_xlabel("Time (MEEP units)")
    ax.set_ylabel("Ez at center")
    ax.set_title("E-field Time Response")
    ax.grid(True, alpha=0.3)

    plt.tight_layout()
    plt.savefig(f"{output_dir}/time_response.png", dpi=150)
    print(f"Plot saved to {output_dir}/time_response.png")
    plt.close()


# =============================================================================
# Entry Point
# =============================================================================

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="MEEP FDTD Simulation")
    parser.add_argument("--plot", action="store_true", help="Generate plots")
    parser.add_argument("--output", type=str, default="output", help="Output directory")
    parser.add_argument("--resolution", type=float, default=RESOLUTION, help="Override resolution")

    args = parser.parse_args()

    if args.resolution != RESOLUTION:
        RESOLUTION = args.resolution

    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    output_dir = f"{args.output}/sim_{timestamp}"

    run_simulation(output_dir=output_dir, plot=args.plot)
"##;

/// Generate a MEEP Python script from a simulation definition
pub fn generate_meep_script(sim: &MeepSimulation, config: &TranslationConfig) -> Result<String> {
    let mut env = Environment::new();
    env.add_template("meep", MEEP_TEMPLATE)?;

    let template = env.get_template("meep")?;

    // Convert geometry to template format
    let geometry: Vec<_> = sim.geometry.iter().map(|g| {
        let (geom_type, extra) = match &g.primitive {
            MeepPrimitive::Block { size } => ("block", serde_json::json!({
                "sx": size.x,
                "sy": size.y,
                "sz": size.z,
            })),
            MeepPrimitive::Cylinder { radius, height, axis } => ("cylinder", serde_json::json!({
                "radius": radius,
                "height": height,
                "ax": axis.x,
                "ay": axis.y,
                "az": axis.z,
            })),
            MeepPrimitive::Sphere { radius } => ("sphere", serde_json::json!({
                "radius": radius,
            })),
        };

        serde_json::json!({
            "name": g.name,
            "type": geom_type,
            "cx": g.center.x,
            "cy": g.center.y,
            "cz": g.center.z,
            "material": g.material,
            "sx": extra.get("sx").and_then(|v| v.as_f64()).unwrap_or(0.0),
            "sy": extra.get("sy").and_then(|v| v.as_f64()).unwrap_or(0.0),
            "sz": extra.get("sz").and_then(|v| v.as_f64()).unwrap_or(0.0),
            "radius": extra.get("radius").and_then(|v| v.as_f64()).unwrap_or(0.0),
            "height": extra.get("height").and_then(|v| v.as_f64()).unwrap_or(0.0),
            "ax": extra.get("ax").and_then(|v| v.as_f64()).unwrap_or(0.0),
            "ay": extra.get("ay").and_then(|v| v.as_f64()).unwrap_or(0.0),
            "az": extra.get("az").and_then(|v| v.as_f64()).unwrap_or(1.0),
        })
    }).collect();

    // Convert sources
    let sources: Vec<_> = sim.sources.iter().map(|s| {
        let (src_type, fcen, fwidth, frequency) = match &s.source_type {
            MeepSourceType::GaussianSource { fcen, fwidth } => ("gaussian", *fcen, *fwidth, 0.0),
            MeepSourceType::ContinuousSource { frequency } => ("continuous", 0.0, 0.0, *frequency),
        };
        serde_json::json!({
            "type": src_type,
            "component": s.component,
            "cx": s.center.x,
            "cy": s.center.y,
            "cz": s.center.z,
            "sx": s.size.x,
            "sy": s.size.y,
            "sz": s.size.z,
            "fcen": fcen,
            "fwidth": fwidth,
            "frequency": frequency,
        })
    }).collect();

    // Convert monitors
    let monitors: Vec<_> = sim.monitors.iter().map(|m| {
        serde_json::json!({
            "name": m.name,
            "component": m.component,
            "cx": m.center.x,
            "cy": m.center.y,
            "cz": m.center.z,
        })
    }).collect();

    // Convert flux monitors
    let flux_monitors: Vec<_> = sim.flux_monitors.iter().map(|f| {
        serde_json::json!({
            "name": f.name,
            "cx": f.center.x,
            "cy": f.center.y,
            "cz": f.center.z,
            "sx": f.size.x,
            "sy": f.size.y,
            "sz": f.size.z,
            "direction": f.direction,
        })
    }).collect();

    let source_unit = format!("{:?}", config.source_unit);
    let meep_unit = format!("{:?}", config.meep_unit);

    let output = template.render(context! {
        header_comment => "Translated from Mittens CAD geometry",
        timestamp => chrono::Utc::now().to_rfc3339(),
        source_unit => source_unit,
        meep_unit => meep_unit,
        cell_x => sim.cell_size.x,
        cell_y => sim.cell_size.y,
        cell_z => sim.cell_size.z,
        resolution => sim.resolution,
        pml_thickness => sim.pml_thickness,
        fcen => sim.fcen,
        fwidth => sim.fwidth,
        unit_scale => sim.unit_scale,
        geometry => geometry,
        sources => sources,
        monitors => monitors,
        flux_monitors => flux_monitors,
    })?;

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::MittensScene;

    #[test]
    fn test_empty_template_renders() {
        let sim = MeepSimulation {
            cell_size: nalgebra::Vector3::new(100.0, 100.0, 100.0),
            resolution: 10.0,
            pml_thickness: 10.0,
            geometry: vec![],
            sources: vec![],
            monitors: vec![],
            flux_monitors: vec![],
            fcen: 0.01,
            fwidth: 0.005,
            unit_scale: 1000.0,
        };

        let config = TranslationConfig::default();
        let script = generate_meep_script(&sim, &config).unwrap();

        assert!(script.contains("import meep as mp"));
        assert!(script.contains("RESOLUTION = 10.0"));
    }
}
