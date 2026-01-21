# Acoustic Field Simulation

Specs for ultrasound pressure field visualization. Backend was in `acoustic.rs`, deleted to reduce scope.

## Physics Model

Rayleigh-Sommerfeld diffraction integral for circular piston transducers:
- Divides transducer face into ring elements
- Integrates complex pressure contribution from each element
- Includes reflection from coverslip boundary (water/glass interface, R ≈ 0.79)

## Pattern Matching

Backend triggers when main.rs detects:
- `acoustic(` or `Acoustic` in Lua content
- Global tables: `Acoustic`, `Transducer`, `PolyTube`, `Medium`

Required globals for computation:
```lua
Acoustic = { frequency = 1e6, drive_current = 0.1 }
Transducer = { diameter = 12.0, height_from_coverslip = 5.0 }
PolyTube = { inner_diameter = 26.0 }
Medium = { liquid_height = 8.0 }
```

## Config Structure (was AcousticConfig)

```rust
struct AcousticConfig {
    frequency: f64,           // Hz
    transducer_radius: f64,   // mm
    transducer_z: f64,        // mm (height above coverslip at z=0)
    medium_radius: f64,       // mm (cylindrical domain)
    medium_height: f64,       // mm
    speed_of_sound: f64,      // mm/s (default: 1480 * 1000 for water)
    drive_amplitude: f64,     // arbitrary scaling
}
```

## Binary Format

Header: `FIELD\0\0\0` (8 bytes)

Then sequential:
1. slice_width: u32
2. slice_height: u32
3. bounds: [f32; 4] = [x_min, x_max, z_min, z_max]
4. slice_pressure: [f32; width * height] (normalized 0-1)
5. (duplicate of slice_pressure for compatibility with magnetic field format)
6. slice_magnitude: [f32; width * height]
7. num_arrows: u32 = 0
8. line_points: u32 = 0

Reuses same binary format as magnetic field for renderer compatibility.

## Renderer Display

Renderer (`main.ts`) displays field plane:
- XZ plane at Y=0
- Jet colormap
- Transparent pixels where magnitude < 1e-6

## Lua API (deleted)

Was:
```lua
AcousticPressurePlane(plane, offset, config)
AcousticEnergyPlane(plane, offset, config)
AcousticIntensityPlane(plane, offset, config)
Hydrophone(position, config)
```

Only `AcousticPressurePlane` had backend support. Others were API-only.

## Rayleigh Integral Implementation

```rust
fn rayleigh_piston(field_r, field_z, piston_z, piston_radius, k, n_rings, n_segments) -> (real, imag) {
    // For each ring on transducer face
    for ring in 0..n_rings {
        rho = piston_radius * (ring + 0.5) / n_rings
        ring_area = 2π * rho * (piston_radius / n_rings)

        // For each segment on ring
        for seg in 0..n_segments {
            phi = 2π * seg / n_segments
            src_x = rho * cos(phi)
            src_y = rho * sin(phi)

            dist = sqrt((field_r - src_x)² + src_y² + (piston_z - field_z)²)
            phase = k * dist

            p_real += d_area * cos(phase) / dist
            p_imag += d_area * -sin(phase) / dist
        }
    }
    return (p_real, p_imag)
}
```

## Reflection Modeling

Mirror source technique:
- Real transducer at z = transducer_z
- Virtual transducer at z = -transducer_z (mirror across coverslip at z=0)
- Total field = direct + R * reflected
- R = (Z_glass - Z_water) / (Z_glass + Z_water) ≈ 0.79

## Typical Parameters

| Parameter | Typical Value | Notes |
|-----------|---------------|-------|
| frequency | 1 MHz | Ultrasound range |
| transducer_radius | 6 mm | Piezo disc |
| speed_of_sound | 1480 m/s | Water at 25°C |
| Z_water | 1.48 MRayl | Acoustic impedance |
| Z_glass | 12.6 MRayl | Borosilicate |
| n_rings | 12 | Integration resolution |
| n_segments | 24 | Angular resolution |

## Lessons Learned

1. Pattern matching approach works well for prototype physics
2. Reusing FIELD binary format simplified renderer
3. Circular piston assumption sufficient for disc transducers
4. Reflection model critical for standing wave patterns
5. Normalize field magnitude for consistent colormap display
