# MEEP Simulations for Mittens

> **⚠️ DEPRECATED:** This Python-based approach is being replaced by native Rust MEEP bindings.
> See `specs/meep/SPEC.md` for the new server-side implementation plan.
> These scripts remain for reference until the Rust implementation is complete.

Full-wave electromagnetic FDTD simulations using [MEEP](https://meep.readthedocs.io/).

## Setup

```bash
# Install MEEP (recommended: conda)
conda create -n meep -c conda-forge pymeep
conda activate meep

# Or with pip (may need HDF5/MPI deps)
pip install meep
```

## Bridge Gap Resonator

Simulates the bridge gap resonator geometry from `../multiphysics/bridge_gap_resonator.lua`.

### Quick Run (Low Resolution Test)

```bash
python bridge_gap_resonator.py --resolution 5 --plot
```

Takes ~5-10 minutes. Good for checking geometry and setup.

### Production Run (High Resolution)

```bash
# Single core (slow)
python bridge_gap_resonator.py --resolution 20 --plot

# Parallel (recommended)
mpirun -np 8 python bridge_gap_resonator.py --resolution 20 --plot
```

Takes 1-4 hours depending on cores and resolution.

### What It Does

1. **Reference run**: Empty simulation to capture incident flux
2. **Main run**: Full geometry with:
   - Gaussian pulse excitation across the gap (broadband)
   - E-field monitoring at gap center over time
   - Flux monitors for S-parameter extraction
   - Field slice output for visualization

### Outputs

```
output/bgr_YYYYMMDD_HHMMSS/
├── results.npz       # NumPy archive with S-params and time data
├── results.png       # Plots: S11, S21, time response, spectrum
└── ez-*.h5           # HDF5 field slices (if enabled)
```

### Parameters

| Flag | Default | Description |
|------|---------|-------------|
| `--resolution` | 5 | Pixels per µm (use 20+ for accuracy) |
| `--freq-center` | 5.0 | Center frequency in GHz |
| `--freq-width` | 4.0 | Bandwidth in GHz |
| `--plot` | false | Generate matplotlib plots |
| `--output` | output | Output directory |

### Understanding the Results

- **S11 (dB)**: Reflection coefficient. Dips indicate resonance (energy entering structure)
- **S21 (dB)**: Transmission coefficient. Peaks indicate efficient coupling
- **Time response**: How E-field at gap evolves after pulse excitation
- **Spectrum**: FFT of time response — should show resonant frequencies

### Scaling Notes

MEEP uses **normalized units** internally. This simulation uses:
- Length unit: 1 µm
- Frequency conversion: `f_GHz = f_meep × 300`

At resolution 20, a 60mm × 30mm × 20mm cell = 1.2B × 600M × 400k = **very large**.

For initial experiments, either:
1. Use low resolution (5-10) to see qualitative behavior
2. Reduce cell size / use 2D approximation
3. Run on a cluster with MPI

### Next Steps

- [ ] Add Drude model for realistic copper (matters at high freq)
- [ ] Add the resonance coil geometry
- [ ] Implement near-to-far-field transform for radiation pattern
- [ ] Adaptive mesh refinement near gap
