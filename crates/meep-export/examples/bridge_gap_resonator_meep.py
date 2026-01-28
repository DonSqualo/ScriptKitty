#!/usr/bin/env python3
"""
MEEP FDTD Simulation - Auto-generated from Mittens CAD
Translated from Mittens CAD geometry

Generated: 2026-01-27T23:55:40.766116345+00:00
Source unit: Millimeter
MEEP unit: Micrometer
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
CELL_X = 58500.000000
CELL_Y = 36000.000000
CELL_Z = 16000.000000

# Resolution (pixels per length unit)
RESOLUTION = 10.0

# PML thickness
PML_THICKNESS = 1000.000000

# Frequency (MEEP units: f_meep = f_Hz * length_unit / c)
FCEN = 1.667820e-05
FWIDTH = 1.334256e-05

# Unit scale: multiply source units by this to get MEEP units
UNIT_SCALE = 1000.000000


# =============================================================================
# Geometry Definition
# =============================================================================

def build_geometry():
    """Build MEEP geometry objects."""
    geometry = []


    # left_pad

    geometry.append(mp.Block(
        center=mp.Vector3(-13750.000000, 0.000000, 250.000000),
        size=mp.Vector3(25000.000000, 8000.000000, 500.000000),
        material=mp.metal
    ))



    # right_pad

    geometry.append(mp.Block(
        center=mp.Vector3(13750.000000, 0.000000, 250.000000),
        size=mp.Vector3(25000.000000, 8000.000000, 500.000000),
        material=mp.metal
    ))



    # bridge

    geometry.append(mp.Block(
        center=mp.Vector3(0.000000, 0.000000, 625.000000),
        size=mp.Vector3(6500.000000, 4800.000000, 250.000000),
        material=mp.metal
    ))



    # tube_outer

    geometry.append(mp.Cylinder(
        center=mp.Vector3(0.000000, 0.000000, 0.000000),
        radius=15000.000000,
        height=9600.000000,
        axis=mp.Vector3(0.0, 0.0, 1.0),
        material=mp.Medium(epsilon=4.4)
    ))



    # tube_inner

    geometry.append(mp.Cylinder(
        center=mp.Vector3(0.000000, 0.000000, 0.000000),
        radius=13000.000000,
        height=10000.000000,
        axis=mp.Vector3(0.0, 0.0, 1.0),
        material=mp.air
    ))



    return geometry


# =============================================================================
# Sources
# =============================================================================

def build_sources():
    """Build MEEP excitation sources."""
    sources = []



    sources.append(mp.Source(
        src=mp.GaussianSource(frequency=1.667820e-05, fwidth=1.334256e-05),
        component=mp.Ez,
        center=mp.Vector3(0.000000, 0.000000, 0.000000),
        size=mp.Vector3(0.000000, 0.000000, 0.000000)
    ))



    return sources


# =============================================================================
# Monitors and Flux Regions
# =============================================================================


def get_field_monitors():
    """Return list of (name, component, position) for field monitoring."""
    return [

        ("E_center", mp.Ez, mp.Vector3(0.000000, 0.000000, 0.000000)),

    ]



def build_flux_monitors(sim, fcen, fwidth, nfreq=50):
    """Add flux monitors to simulation for S-parameter extraction."""
    monitors = {}

    monitors["flux_x_pos"] = sim.add_flux(
        fcen, fwidth, nfreq,
        mp.FluxRegion(
            center=mp.Vector3(26750.000000, 0.000000, 0.000000),
            size=mp.Vector3(0.000000, 30000.000000, 10000.000000),
            direction=mp.X,
            weight=1
        )
    )

    monitors["flux_x_neg"] = sim.add_flux(
        fcen, fwidth, nfreq,
        mp.FluxRegion(
            center=mp.Vector3(-26750.000000, 0.000000, 0.000000),
            size=mp.Vector3(0.000000, 30000.000000, 10000.000000),
            direction=mp.X,
            weight=-1
        )
    )

    return monitors



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


    # Add flux monitors
    flux_monitors = build_flux_monitors(sim, FCEN, FWIDTH)


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