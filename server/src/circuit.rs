use anyhow::Result;
use std::f64::consts::PI;

pub struct CircuitData {
    pub size: [f32; 2],
    pub svg: String,
}

struct Component {
    config: ComponentConfig,
}

enum ComponentConfig {
    SignalGenerator { frequency: f64, amplitude: f64 },
    Amplifier { gain: f64 },
    MatchingNetwork { impedance_real: f64, impedance_imag: f64, frequency: f64 },
    Transducer { impedance_real: f64, impedance_imag: f64 },
}

struct MatchingValues {
    inductance: f64,
    capacitance: f64,
}

fn calculate_matching(impedance_real: f64, impedance_imag: f64, frequency: f64) -> MatchingValues {
    let omega = 2.0 * PI * frequency;
    let inductance = impedance_imag.abs() / omega;
    let capacitance = 1.0 / (omega * impedance_real);
    MatchingValues { inductance, capacitance }
}

fn format_value(value: f64, unit: &str) -> String {
    if value >= 1e-3 {
        format!("{:.1}m{}", value * 1e3, unit)
    } else if value >= 1e-6 {
        format!("{:.1}μ{}", value * 1e6, unit)
    } else if value >= 1e-9 {
        format!("{:.1}n{}", value * 1e9, unit)
    } else {
        format!("{:.1}p{}", value * 1e12, unit)
    }
}

const WIRE: &str = "rgba(255,255,255,0.4)";
const COMP: &str = "rgba(255,255,255,0.6)";
const TEXT: &str = "rgba(255,255,255,0.35)";

struct ComponentDraw {
    svg: String,
    input_x: f64,
    output_x: f64,
    signal_y: f64,
}

// Signal generator - just the circle with sine wave, no vertical rails
fn draw_signal_generator(x: f64, signal_y: f64) -> ComponentDraw {
    let r = 12.0;
    let svg = format!(
        r##"<g>
          <circle cx="{x}" cy="{signal_y}" r="{r}" fill="none" stroke="{COMP}" stroke-width="1"/>
          <path d="M{x0} {signal_y} q{q1} -{q2} {q3} 0 q{q1} {q2} {q3} 0" fill="none" stroke="{COMP}" stroke-width="1"/>
        </g>"##,
        x = x,
        signal_y = signal_y,
        r = r,
        COMP = COMP,
        x0 = x - 7.0,
        q1 = 3.5,
        q2 = 6.0,
        q3 = 7.0,
    );
    ComponentDraw {
        svg,
        input_x: x - r,
        output_x: x + r,
        signal_y,
    }
}

// Amplifier triangle
fn draw_amplifier(x: f64, signal_y: f64) -> ComponentDraw {
    let w = 24.0;
    let h = 18.0;
    let svg = format!(
        r##"<g>
          <path d="M{x0} {y0} L{x0} {y1} L{x1} {signal_y} Z" fill="none" stroke="{COMP}" stroke-width="1"/>
        </g>"##,
        x0 = x - w/2.0,
        x1 = x + w/2.0,
        y0 = signal_y - h/2.0,
        y1 = signal_y + h/2.0,
        signal_y = signal_y,
        COMP = COMP,
    );
    ComponentDraw {
        svg,
        input_x: x - w/2.0,
        output_x: x + w/2.0,
        signal_y,
    }
}

// L-match: series inductor, then shunt capacitor to ground
fn draw_matching_network(x: f64, signal_y: f64, gnd_y: f64, l_label: &str, c_label: &str) -> ComponentDraw {
    // Layout: input -> inductor -> junction -> output
    //                              |
    //                              capacitor
    //                              |
    //                              GND
    let l_start = x - 15.0;
    let l_end = x + 15.0;  // inductor ends here
    let junction_x = l_end + 12.0;  // junction point after inductor
    let output_x = junction_x + 35.0;  // more room for capacitor + gap

    let svg = format!(
        r##"<g>
          <!-- Series Inductor -->
          <path d="M{l_start} {signal_y} a5,5 0 0,1 10,0 a5,5 0 0,1 10,0 a5,5 0 0,1 10,0" fill="none" stroke="{COMP}" stroke-width="1"/>
          <text x="{l_center}" y="{l_label_y}" fill="{TEXT}" font-size="10" font-family="monospace" text-anchor="middle">{l_label}</text>

          <!-- Wire from inductor to junction -->
          <line x1="{l_end}" y1="{signal_y}" x2="{junction_x}" y2="{signal_y}" stroke="{WIRE}" stroke-width="1"/>

          <!-- Junction dot -->
          <circle cx="{junction_x}" cy="{signal_y}" r="2" fill="{WIRE}"/>

          <!-- Wire from junction to output -->
          <line x1="{junction_x}" y1="{signal_y}" x2="{output_x}" y2="{signal_y}" stroke="{WIRE}" stroke-width="1"/>

          <!-- Shunt Capacitor: junction down to GND (horizontal plates) -->
          <line x1="{junction_x}" y1="{signal_y}" x2="{junction_x}" y2="{c_top}" stroke="{WIRE}" stroke-width="1"/>
          <line x1="{c_left}" y1="{c_top}" x2="{c_right}" y2="{c_top}" stroke="{COMP}" stroke-width="2"/>
          <line x1="{c_left}" y1="{c_bot}" x2="{c_right}" y2="{c_bot}" stroke="{COMP}" stroke-width="2"/>
          <line x1="{junction_x}" y1="{c_bot}" x2="{junction_x}" y2="{gnd_y}" stroke="{WIRE}" stroke-width="1"/>
          <text x="{c_label_x}" y="{c_label_y}" fill="{TEXT}" font-size="10" font-family="monospace" text-anchor="start">{c_label}</text>
        </g>"##,
        COMP = COMP,
        TEXT = TEXT,
        WIRE = WIRE,
        l_start = l_start,
        l_end = l_end,
        l_center = x,
        signal_y = signal_y,
        l_label_y = signal_y - 12.0,
        l_label = l_label,
        junction_x = junction_x,
        output_x = output_x,
        c_left = junction_x - 6.0,
        c_right = junction_x + 6.0,
        c_top = signal_y + 12.0,
        c_bot = signal_y + 16.0,
        c_label_x = junction_x + 8.0,
        c_label_y = signal_y + 20.0,
        c_label = c_label,
        gnd_y = gnd_y,
    );
    ComponentDraw {
        svg,
        input_x: l_start,
        output_x: output_x,
        signal_y,
    }
}

// Transducer (piezo element) - shifted right to avoid overlap with matching network
fn draw_transducer(x: f64, signal_y: f64, gnd_y: f64) -> ComponentDraw {
    let x = x + 20.0;  // shift right
    let w = 28.0;
    let h = 16.0;
    let svg = format!(
        r##"<g>
          <rect x="{rx}" y="{ry}" width="{w}" height="{h}" fill="none" stroke="{COMP}" stroke-width="1"/>
          <line x1="{d0}" y1="{d1}" x2="{d2}" y2="{d3}" stroke="{COMP}" stroke-width="0.5"/>
          <!-- Ground connection -->
          <line x1="{x}" y1="{y1}" x2="{x}" y2="{gnd_y}" stroke="{WIRE}" stroke-width="1"/>
        </g>"##,
        x = x,
        rx = x - w/2.0,
        ry = signal_y - h/2.0,
        w = w,
        h = h,
        d0 = x - w/2.0 + 4.0,
        d1 = signal_y - h/2.0 + 4.0,
        d2 = x + w/2.0 - 4.0,
        d3 = signal_y + h/2.0 - 4.0,
        y1 = signal_y + h/2.0,
        gnd_y = gnd_y,
        COMP = COMP,
        WIRE = WIRE,
    );
    ComponentDraw {
        svg,
        input_x: x - w/2.0,
        output_x: x + w/2.0,
        signal_y,
    }
}

pub fn generate_circuit_from_lua(_lua: &mlua::Lua, table: &mlua::Table) -> Result<CircuitData> {
    let size: Vec<f32> = table.get::<_, mlua::Table>("size")?
        .sequence_values::<f32>()
        .filter_map(|r| r.ok())
        .collect();

    let components_table: mlua::Table = table.get("components")?;

    let mut components: Vec<Component> = Vec::new();

    for pair in components_table.pairs::<i32, mlua::Table>() {
        let (_, comp_table) = pair?;
        let comp_type: String = comp_table.get("component")?;
        let config_table: mlua::Table = comp_table.get("config")?;

        let config = match comp_type.as_str() {
            "signal_generator" => ComponentConfig::SignalGenerator {
                frequency: config_table.get("frequency").unwrap_or(1e6),
                amplitude: config_table.get("amplitude").unwrap_or(1.0),
            },
            "amplifier" => ComponentConfig::Amplifier {
                gain: config_table.get("gain").unwrap_or(10.0),
            },
            "matching_network" => ComponentConfig::MatchingNetwork {
                impedance_real: config_table.get("transducer_impedance_real").unwrap_or(50.0),
                impedance_imag: config_table.get("transducer_impedance_imag").unwrap_or(0.0),
                frequency: config_table.get("frequency").unwrap_or(1e6),
            },
            "transducer" => ComponentConfig::Transducer {
                impedance_real: config_table.get("impedance_real").unwrap_or(50.0),
                impedance_imag: config_table.get("impedance_imag").unwrap_or(0.0),
            },
            _ => continue,
        };

        components.push(Component { config });
    }

    let width = size.get(0).copied().unwrap_or(400.0) as f64;
    let height = size.get(1).copied().unwrap_or(100.0) as f64;
    let svg = generate_svg(&components, width, height);

    Ok(CircuitData {
        size: [width as f32, height as f32],
        svg,
    })
}

fn generate_svg(components: &[Component], width: f64, height: f64) -> String {
    let margin = 30.0;
    let signal_y = height * 0.35; // Signal line in upper portion
    let gnd_y = height - margin;

    let mut svg_parts: Vec<String> = Vec::new();

    // Draw GND rail only (no V+ rail - it's implicit)
    svg_parts.push(format!(
        r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="1"/>
        <text x="{}" y="{}" fill="{}" font-size="10" font-family="monospace">GND</text>"##,
        margin, gnd_y, width - margin, gnd_y, WIRE,
        5.0, gnd_y + 4.0, TEXT
    ));

    if components.is_empty() {
        return wrap_svg(&svg_parts, width, height);
    }

    let usable_width = width - 2.0 * margin - 60.0;
    let spacing = usable_width / components.len() as f64;

    let mut drawn: Vec<ComponentDraw> = Vec::new();

    for (i, comp) in components.iter().enumerate() {
        let x = margin + 40.0 + spacing * i as f64 + spacing / 2.0;

        let draw = match &comp.config {
            ComponentConfig::SignalGenerator { .. } => {
                draw_signal_generator(x, signal_y)
            }
            ComponentConfig::Amplifier { .. } => {
                draw_amplifier(x, signal_y)
            }
            ComponentConfig::MatchingNetwork { impedance_real, impedance_imag, frequency } => {
                let matching = calculate_matching(*impedance_real, *impedance_imag, *frequency);
                let l_label = format_value(matching.inductance, "H");
                let c_label = format_value(matching.capacitance, "F");
                draw_matching_network(x, signal_y, gnd_y, &l_label, &c_label)
            }
            ComponentConfig::Transducer { .. } => {
                draw_transducer(x, signal_y, gnd_y)
            }
        };

        svg_parts.push(draw.svg.clone());
        drawn.push(draw);
    }

    // Draw horizontal wires connecting components (signal path in series)
    for i in 0..drawn.len().saturating_sub(1) {
        let x1 = drawn[i].output_x;
        let x2 = drawn[i + 1].input_x;
        let y = drawn[i].signal_y;
        svg_parts.push(format!(
            r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="1"/>"##,
            x1, y, x2, y, WIRE
        ));
    }

    // Input wire from left edge
    if !drawn.is_empty() {
        svg_parts.push(format!(
            r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="1"/>"##,
            margin, signal_y, drawn[0].input_x, signal_y, WIRE
        ));
    }

    wrap_svg(&svg_parts, width, height)
}

fn wrap_svg(parts: &[String], width: f64, height: f64) -> String {
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}">
        {}
        </svg>"##,
        width, height, width, height,
        parts.join("\n        ")
    )
}

pub fn serialize_circuit(circuit: &CircuitData) -> Vec<u8> {
    let mut data = Vec::new();

    data.extend_from_slice(b"CIRCUIT\0");

    for &v in &circuit.size {
        data.extend_from_slice(&v.to_le_bytes());
    }

    let svg_bytes = circuit.svg.as_bytes();
    data.extend_from_slice(&(svg_bytes.len() as u32).to_le_bytes());
    data.extend_from_slice(svg_bytes);

    data
}
