#!/usr/bin/env python3
"""
MEEP FDTD simulation of the bridge gap resonator.

This translates the Mittens bridge_gap_resonator.lua geometry into
a full electromagnetic time-domain simulation.

Key features:
- 3D FDTD with real units (um scale, GHz frequencies)
- PEC conductors (copper at GHz is effectively perfect)
- Gaussian pulse excitation across the gap
- Field monitors to watch E/B evolve
- S-parameter extraction via flux monitors

Usage:
    python bridge_gap_resonator.py          # Run simulation
    python bridge_gap_resonator.py --plot   # Plot results after
    mpirun -np 4 python bridge_gap_resonator.py  # Parallel
"""

import meep as mp
import numpy as np
import argparse
import os
from datetime import datetime

# =============================================================================
# Configuration (matching bridge_gap_resonator.lua)
# =============================================================================

# MEEP works best with normalized units, but we'll use um as length unit
# This means: 1 length unit = 1 um, frequencies in units where c = 1 um/time_unit
# So f (in MEEP units) = f_real * (1 um / c) = f_real / 299.792 THz

# Convert from mm to um (MEEP length units)
MM_TO_UM = 1000

config = {
    # Bridge gap resonator (convert mm -> um)
    "gap": 2.5 * MM_TO_UM,           # 2500 um gap
    "bridge_w": 8 * MM_TO_UM,        # 8000 um bridge width
    "bridge_t": 0.5 * MM_TO_UM,      # 500 um bridge thickness
    "pad_length": 25 * MM_TO_UM,     # 25000 um pad length
    
    # Substrate tube
    "substrate_r": 15 * MM_TO_UM,    # 15 mm radius
    "substrate_h": 9.6 * MM_TO_UM,   # 9.6 mm height
    "wall_thickness": 2 * MM_TO_UM,  # 2 mm wall
    
    # Simulation parameters
    "resolution": 5,  # pixels per um (coarse for testing, use 20+ for production)
    "pml_thickness": 2000,  # um, PML absorbing boundary
    
    # Frequency range (GHz)
    "freq_center_ghz": 5.0,   # Center frequency
    "freq_width_ghz": 4.0,    # Bandwidth (1-9 GHz roughly)
}

# Convert GHz to MEEP frequency units
# f_meep = f_Hz * (length_unit / c) = f_GHz * 1e9 * (1e-6 m / 3e8 m/s) = f_GHz / 300
def ghz_to_meep(f_ghz):
    return f_ghz / 300.0

def meep_to_ghz(f_meep):
    return f_meep * 300.0


# =============================================================================
# Materials
# =============================================================================

# At GHz frequencies, copper is effectively a perfect conductor
# But we can also model it with a Drude model for more accuracy
# Copper conductivity: σ = 5.96e7 S/m
# In MEEP: D_conductivity in units of 2*pi*f (use meep.Conductivity for convenience)

# For simplicity, use perfect metal (mp.metal) initially
# Can switch to: mp.Medium(epsilon=1, D_conductivity=2*mp.pi*5.96e7 * 1e-6)
copper = mp.metal  # Perfect electric conductor

# FR4 substrate
fr4 = mp.Medium(epsilon=4.4)

# Air (default, epsilon=1)


# =============================================================================
# Geometry Construction
# =============================================================================

def build_geometry(cfg):
    """Build the MEEP geometry list from config."""
    
    geometry = []
    
    gap = cfg["gap"]
    bridge_w = cfg["bridge_w"]
    bridge_t = cfg["bridge_t"]
    pad_length = cfg["pad_length"]
    
    # --- Bridge Gap Resonator (at z=0 plane) ---
    
    # Left pad: centered at (-gap/2 - pad_length/2, 0, bridge_t/2)
    left_pad_center = mp.Vector3(-gap/2 - pad_length/2, 0, bridge_t/2)
    geometry.append(mp.Block(
        center=left_pad_center,
        size=mp.Vector3(pad_length, bridge_w, bridge_t),
        material=copper
    ))
    
    # Right pad: centered at (+gap/2 + pad_length/2, 0, bridge_t/2)  
    right_pad_center = mp.Vector3(gap/2 + pad_length/2, 0, bridge_t/2)
    geometry.append(mp.Block(
        center=right_pad_center,
        size=mp.Vector3(pad_length, bridge_w, bridge_t),
        material=copper
    ))
    
    # Bridge: thin strip over gap
    bridge_width = gap + 4 * MM_TO_UM  # gap + 4mm overhang
    bridge_height = bridge_t * 0.5
    bridge_y_size = bridge_w * 0.6
    geometry.append(mp.Block(
        center=mp.Vector3(0, 0, bridge_t * 1.25),
        size=mp.Vector3(bridge_width, bridge_y_size, bridge_height),
        material=copper
    ))
    
    # --- FR4 Tube Substrate (below z=0) ---
    # Outer cylinder - inner cylinder = tube wall
    # MEEP doesn't have native CSG, so we approximate with a thick-walled cylinder
    # Actually MEEP has Cylinder primitive
    
    substrate_r = cfg["substrate_r"]
    substrate_h = cfg["substrate_h"]
    wall_t = cfg["wall_thickness"]
    
    # Outer cylinder
    geometry.append(mp.Cylinder(
        center=mp.Vector3(0, 0, -substrate_h/2),
        radius=substrate_r,
        height=substrate_h,
        material=fr4
    ))
    
    # Inner cylinder (air, cuts out the tube interior)
    geometry.append(mp.Cylinder(
        center=mp.Vector3(0, 0, -substrate_h/2),
        radius=substrate_r - wall_t,
        height=substrate_h + 100,  # Slightly taller to ensure clean cut
        material=mp.air
    ))
    
    return geometry


# =============================================================================
# Simulation Setup
# =============================================================================

def create_simulation(cfg, output_dir="output"):
    """Create and configure the MEEP simulation."""
    
    # Calculate cell size (needs to encompass geometry + PML)
    pad_length = cfg["pad_length"]
    gap = cfg["gap"]
    bridge_w = cfg["bridge_w"]
    substrate_h = cfg["substrate_h"]
    substrate_r = cfg["substrate_r"]
    pml = cfg["pml_thickness"]
    
    # Cell dimensions
    cell_x = 2 * (pad_length + gap/2) + 2 * pml + 4000  # Extra margin
    cell_y = max(bridge_w, 2 * substrate_r) + 2 * pml + 4000
    cell_z = substrate_h + cfg["bridge_t"] * 2 + 2 * pml + 4000
    
    cell_size = mp.Vector3(cell_x, cell_y, cell_z)
    
    # Geometry
    geometry = build_geometry(cfg)
    
    # Source: Gaussian pulse current source across the gap
    # This mimics driving the resonator with a broadband signal
    fcen = ghz_to_meep(cfg["freq_center_ghz"])
    fwidth = ghz_to_meep(cfg["freq_width_ghz"])
    
    # Current source in the gap region (Ez component, driving vertical E-field)
    sources = [
        mp.Source(
            src=mp.GaussianSource(fcen, fwidth=fwidth),
            component=mp.Ez,
            center=mp.Vector3(0, 0, cfg["bridge_t"]/2),
            size=mp.Vector3(cfg["gap"] * 0.8, cfg["bridge_w"] * 0.5, 0)
        )
    ]
    
    # PML boundary layers
    pml_layers = [mp.PML(thickness=pml)]
    
    # Create simulation
    sim = mp.Simulation(
        cell_size=cell_size,
        geometry=geometry,
        sources=sources,
        boundary_layers=pml_layers,
        resolution=cfg["resolution"],
        default_material=mp.air,
    )
    
    return sim, fcen, fwidth


# =============================================================================
# Monitors and Output
# =============================================================================

def setup_monitors(sim, cfg, fcen, fwidth):
    """Set up field monitors and flux regions."""
    
    monitors = {}
    
    # Flux monitor across the gap (for S-parameter extraction)
    gap = cfg["gap"]
    bridge_w = cfg["bridge_w"]
    bridge_t = cfg["bridge_t"]
    
    # Reflected flux (between source and left pad)
    monitors["refl"] = sim.add_flux(
        fcen, fwidth, 50,  # 50 frequency points
        mp.FluxRegion(
            center=mp.Vector3(-gap/2 - 1000, 0, bridge_t/2),
            size=mp.Vector3(0, bridge_w, bridge_t * 3)
        )
    )
    
    # Transmitted flux (after gap, towards right pad)
    monitors["trans"] = sim.add_flux(
        fcen, fwidth, 50,
        mp.FluxRegion(
            center=mp.Vector3(gap/2 + 1000, 0, bridge_t/2),
            size=mp.Vector3(0, bridge_w, bridge_t * 3)
        )
    )
    
    return monitors


def run_reference(cfg, output_dir):
    """Run simulation without geometry to get reference (incident) flux."""
    
    print("Running reference simulation (no geometry)...")
    
    # Create sim without geometry
    sim, fcen, fwidth = create_simulation(cfg, output_dir)
    sim.geometry = []  # Remove geometry
    
    # Add flux monitor at same location
    gap = cfg["gap"]
    bridge_w = cfg["bridge_w"]
    bridge_t = cfg["bridge_t"]
    
    refl_fr = sim.add_flux(
        fcen, fwidth, 50,
        mp.FluxRegion(
            center=mp.Vector3(-gap/2 - 1000, 0, bridge_t/2),
            size=mp.Vector3(0, bridge_w, bridge_t * 3)
        )
    )
    
    # Run until source decays
    sim.run(until_after_sources=mp.stop_when_fields_decayed(50, mp.Ez, mp.Vector3(0, 0, bridge_t/2), 1e-6))
    
    # Save flux data
    incident_flux = mp.get_fluxes(refl_fr)
    freqs = mp.get_flux_freqs(refl_fr)
    
    sim.reset_meep()
    
    return freqs, incident_flux, sim.get_flux_data(refl_fr)


# =============================================================================
# Main Simulation
# =============================================================================

def run_simulation(cfg, output_dir="output", plot=False):
    """Run the full FDTD simulation."""
    
    os.makedirs(output_dir, exist_ok=True)
    
    print(f"Bridge Gap Resonator FDTD Simulation")
    print(f"=====================================")
    print(f"Gap: {cfg['gap']/MM_TO_UM:.1f} mm")
    print(f"Frequency: {cfg['freq_center_ghz']:.1f} ± {cfg['freq_width_ghz']/2:.1f} GHz")
    print(f"Resolution: {cfg['resolution']} pixels/um")
    print()
    
    # --- Run reference (no geometry) for S-parameter normalization ---
    freqs, incident_flux, refl_data = run_reference(cfg, output_dir)
    
    # --- Main simulation with geometry ---
    print("Running main simulation with geometry...")
    
    sim, fcen, fwidth = create_simulation(cfg, output_dir)
    monitors = setup_monitors(sim, cfg, fcen, fwidth)
    
    # Load reference flux data (for S11 calculation)
    sim.load_minus_flux_data(monitors["refl"], refl_data)
    
    # Field snapshots callback
    field_data = {"t": [], "ez_gap": []}
    
    def capture_field(sim):
        """Capture E-field at gap center."""
        ez = sim.get_field_point(mp.Ez, mp.Vector3(0, 0, cfg["bridge_t"]/2))
        field_data["t"].append(sim.meep_time())
        field_data["ez_gap"].append(ez)
    
    # Output E-field slices periodically
    slice_interval = 20  # time units
    
    def output_efield_slice(sim):
        """Save Ez field slice through gap."""
        sim.output_png(mp.Ez, f"-Zc bluered -C {output_dir}/ez")
    
    # Run simulation
    print("Running time-domain simulation...")
    sim.run(
        mp.at_every(1, capture_field),  # Capture field every time unit
        mp.at_every(slice_interval, mp.in_volume(
            mp.Volume(center=mp.Vector3(0, 0, cfg["bridge_t"]/2), size=mp.Vector3(60000, 30000, 0)),
            mp.output_efield_z
        )),
        until_after_sources=mp.stop_when_fields_decayed(50, mp.Ez, mp.Vector3(0, 0, cfg["bridge_t"]/2), 1e-6)
    )
    
    print("Simulation complete.")
    
    # --- Extract results ---
    
    # Reflected flux
    refl_flux = mp.get_fluxes(monitors["refl"])
    trans_flux = mp.get_fluxes(monitors["trans"])
    
    # Calculate S-parameters
    freqs_ghz = [meep_to_ghz(f) for f in freqs]
    s11 = [-r / i if i != 0 else 0 for r, i in zip(refl_flux, incident_flux)]
    s11_db = [10 * np.log10(abs(s)) if s != 0 else -100 for s in s11]
    s21 = [t / i if i != 0 else 0 for t, i in zip(trans_flux, incident_flux)]
    s21_db = [10 * np.log10(abs(s)) if s != 0 else -100 for s in s21]
    
    # Save results
    results = {
        "freqs_ghz": freqs_ghz,
        "s11_db": s11_db,
        "s21_db": s21_db,
        "field_t": field_data["t"],
        "field_ez": [complex(e).real for e in field_data["ez_gap"]],
    }
    
    np.savez(f"{output_dir}/results.npz", **results)
    print(f"Results saved to {output_dir}/results.npz")
    
    # --- Plot if requested ---
    if plot:
        plot_results(results, output_dir)
    
    return results


def plot_results(results, output_dir):
    """Plot simulation results."""
    
    try:
        import matplotlib.pyplot as plt
    except ImportError:
        print("matplotlib not available, skipping plots")
        return
    
    fig, axes = plt.subplots(2, 2, figsize=(12, 10))
    
    # S11 (reflection)
    ax = axes[0, 0]
    ax.plot(results["freqs_ghz"], results["s11_db"], 'b-', linewidth=1.5)
    ax.set_xlabel("Frequency (GHz)")
    ax.set_ylabel("S11 (dB)")
    ax.set_title("Reflection Coefficient")
    ax.grid(True, alpha=0.3)
    ax.set_ylim(-40, 5)
    
    # S21 (transmission)
    ax = axes[0, 1]
    ax.plot(results["freqs_ghz"], results["s21_db"], 'r-', linewidth=1.5)
    ax.set_xlabel("Frequency (GHz)")
    ax.set_ylabel("S21 (dB)")
    ax.set_title("Transmission Coefficient")
    ax.grid(True, alpha=0.3)
    
    # Time-domain E-field
    ax = axes[1, 0]
    t_ns = [t * 1e-3 for t in results["field_t"]]  # Convert to ns (rough)
    ax.plot(t_ns, results["field_ez"], 'g-', linewidth=0.5)
    ax.set_xlabel("Time (arb. units)")
    ax.set_ylabel("Ez at gap center")
    ax.set_title("E-field Time Response")
    ax.grid(True, alpha=0.3)
    
    # FFT of time-domain response
    ax = axes[1, 1]
    ez = np.array(results["field_ez"])
    if len(ez) > 10:
        fft = np.abs(np.fft.rfft(ez))
        fft_freqs = np.fft.rfftfreq(len(ez))  # Normalized
        ax.plot(fft_freqs[:len(fft)//2], fft[:len(fft)//2], 'm-', linewidth=1)
        ax.set_xlabel("Frequency (normalized)")
        ax.set_ylabel("|FFT(Ez)|")
        ax.set_title("Spectrum of Gap E-field")
        ax.grid(True, alpha=0.3)
    
    plt.tight_layout()
    plt.savefig(f"{output_dir}/results.png", dpi=150)
    print(f"Plot saved to {output_dir}/results.png")
    plt.close()


# =============================================================================
# Entry Point
# =============================================================================

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="MEEP FDTD simulation of bridge gap resonator")
    parser.add_argument("--plot", action="store_true", help="Generate plots after simulation")
    parser.add_argument("--resolution", type=int, default=5, help="Resolution (pixels/um)")
    parser.add_argument("--output", type=str, default="output", help="Output directory")
    parser.add_argument("--freq-center", type=float, default=5.0, help="Center frequency (GHz)")
    parser.add_argument("--freq-width", type=float, default=4.0, help="Frequency width (GHz)")
    
    args = parser.parse_args()
    
    # Update config from args
    config["resolution"] = args.resolution
    config["freq_center_ghz"] = args.freq_center
    config["freq_width_ghz"] = args.freq_width
    
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    output_dir = f"{args.output}/bgr_{timestamp}"
    
    run_simulation(config, output_dir=output_dir, plot=args.plot)
