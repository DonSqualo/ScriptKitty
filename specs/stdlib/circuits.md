# Circuit Diagrams

Specs for 2D circuit schematic overlays. SVG generation was in `circuit.rs`, deleted to reduce scope.

## Purpose

Display simplified circuit schematics for ultrasound drive electronics:
- Signal generator → Amplifier → Matching network → Transducer

## Renderer Integration

Circuit overlays use HTML/CSS positioning anchored to 3D scene:

```typescript
// Anchor point in 3D space
const circuit_anchor = new THREE.Vector3(0, 0, 60);

// Project to screen coordinates
const projected = circuit_anchor.project(camera);
const x = (projected.x * 0.5 + 0.5) * width;
const y = (-projected.y * 0.5 + 0.5) * height;

// Scale based on zoom
const scale = Math.max(0.5, Math.min(3.0, 160 / zoom_distance));
```

Position circuit to LEFT of anchor (right edge at anchor point).

## Binary Protocol

Header: `CIRCUIT\0` (8 bytes)

Then:
1. width: f32
2. height: f32
3. svg_len: u32
4. svg_data: [u8; svg_len] (UTF-8 SVG string)

## Lua API (kept)

```lua
SignalGenerator({ frequency = 1e6, amplitude = 1.0 })
Amplifier({ gain = 10 })
MatchingNetwork({
    impedance_real = 50,
    impedance_imag = -30,
    frequency = 1e6
})
TransducerLoad({
    impedance_real = 50,
    impedance_imag = 0
})

Circuit({
    components = { sig_gen, amp, match, load },
    size = { 400, 100 }
})
```

## Component Types

### SignalGenerator
- Circle with sine wave inside
- Represents RF source

### Amplifier
- Triangle pointing right
- Represents power amplifier

### MatchingNetwork
- L-network: series inductor + shunt capacitor
- Calculates component values from load impedance:
  - L = |X_load| / ω
  - C = 1 / (ω × R_load)

### TransducerLoad
- Rectangle with diagonal line (piezo symbol)
- Connected to ground

## Layout Algorithm

1. Components laid out horizontally with equal spacing
2. Signal line at y = 0.35 × height
3. Ground rail at y = height - margin
4. Wires connect output_x of component N to input_x of component N+1

## SVG Styling

```rust
const WIRE: &str = "rgba(255,255,255,0.4)";  // Connection wires
const COMP: &str = "rgba(255,255,255,0.6)";  // Component outlines
const TEXT: &str = "rgba(255,255,255,0.35)"; // Labels
```

Transparent styling for overlay on 3D scene.

## Value Formatting

```rust
fn format_value(value: f64, unit: &str) -> String {
    if value >= 1e-3 { format!("{:.1}m{}", value * 1e3, unit) }
    else if value >= 1e-6 { format!("{:.1}μ{}", value * 1e6, unit) }
    else if value >= 1e-9 { format!("{:.1}n{}", value * 1e9, unit) }
    else { format!("{:.1}p{}", value * 1e12, unit) }
}
```

## Pattern Matching

Backend triggers when `Circuit` appears in Lua content and result contains `circuit.type == "circuit"`.

## Future Implementation Notes

For regenerating circuit diagrams:
1. Keep Lua API in `stdlib/circuits.lua`
2. Implement SVG generation in `circuit.rs`
3. Key functions needed:
   - `draw_signal_generator(x, y)` - circle with sine
   - `draw_amplifier(x, y)` - triangle
   - `draw_matching_network(x, y, gnd_y, L, C)` - L-match with labels
   - `draw_transducer(x, y, gnd_y)` - piezo rectangle
4. Wire routing: horizontal lines between component ports
5. Ground rail: single horizontal line at bottom

## Serialization

Components serialize as:
```lua
{
    type = "circuit_component",
    component = "matching_network",
    config = { impedance_real = 50, ... },
    position = { x, z }
}
```

Circuit serializes as:
```lua
{
    type = "circuit",
    components = [...],
    size = { width, height }
}
```
