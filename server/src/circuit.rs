//! Circuit diagram SVG generation
//!
//! Generates SVG schematics for ultrasound drive electronics:
//! SignalGenerator -> Amplifier -> MatchingNetwork -> TransducerLoad

use std::f64::consts::PI;

const WIRE: &str = "rgba(255,255,255,0.4)";
const COMP: &str = "rgba(255,255,255,0.6)";
const TEXT: &str = "rgba(255,255,255,0.35)";

pub struct CircuitData {
    pub width: f32,
    pub height: f32,
    pub svg: String,
}

impl CircuitData {
    pub fn to_binary(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(b"CIRCUIT\0");
        data.extend_from_slice(&self.width.to_le_bytes());
        data.extend_from_slice(&self.height.to_le_bytes());
        let svg_bytes = self.svg.as_bytes();
        data.extend_from_slice(&(svg_bytes.len() as u32).to_le_bytes());
        data.extend_from_slice(svg_bytes);
        data
    }
}

#[derive(Clone)]
#[allow(dead_code)]
pub enum CircuitComponent {
    SignalGenerator { frequency: f64, amplitude: f64 },
    Amplifier { gain: f64 },
    MatchingNetwork { impedance_real: f64, impedance_imag: f64, frequency: f64 },
    TransducerLoad { impedance_real: f64, impedance_imag: f64 },
}

fn format_value(value: f64, unit: &str) -> String {
    if value >= 1e-3 {
        format!("{:.1}m{}", value * 1e3, unit)
    } else if value >= 1e-6 {
        format!("{:.1}u{}", value * 1e6, unit)
    } else if value >= 1e-9 {
        format!("{:.1}n{}", value * 1e9, unit)
    } else {
        format!("{:.1}p{}", value * 1e12, unit)
    }
}

fn draw_signal_generator(x: f64, y: f64, frequency: f64) -> (String, f64, f64) {
    let r = 15.0;
    let input_x = x;
    let output_x = x + 2.0 * r;
    let cx = x + r;

    let sine_points: Vec<String> = (0..=20)
        .map(|i| {
            let t = i as f64 / 20.0;
            let sx = cx - r * 0.6 + t * r * 1.2;
            let sy = y - (t * 2.0 * PI).sin() * r * 0.4;
            format!("{:.1},{:.1}", sx, sy)
        })
        .collect();

    let freq_label = if frequency >= 1e6 {
        format!("{:.1}MHz", frequency / 1e6)
    } else if frequency >= 1e3 {
        format!("{:.1}kHz", frequency / 1e3)
    } else {
        format!("{:.1}Hz", frequency)
    };

    let svg = format!(
        r#"<circle cx="{cx}" cy="{y}" r="{r}" fill="none" stroke="{COMP}" stroke-width="1.5"/>
<polyline points="{sine}" fill="none" stroke="{COMP}" stroke-width="1"/>
<text x="{cx}" y="{ty}" fill="{TEXT}" font-size="9" text-anchor="middle">{freq_label}</text>"#,
        cx = cx,
        y = y,
        r = r,
        sine = sine_points.join(" "),
        ty = y + r + 12.0,
    );

    (svg, input_x, output_x)
}

fn draw_amplifier(x: f64, y: f64, gain: f64) -> (String, f64, f64) {
    let w = 30.0;
    let h = 24.0;
    let input_x = x;
    let output_x = x + w;

    let points = format!(
        "{},{} {},{} {},{}",
        x, y - h / 2.0,
        x + w, y,
        x, y + h / 2.0
    );

    let gain_label = format!("x{:.0}", gain);

    let svg = format!(
        r#"<polygon points="{points}" fill="none" stroke="{COMP}" stroke-width="1.5"/>
<text x="{tx}" y="{ty}" fill="{TEXT}" font-size="9" text-anchor="middle">{gain_label}</text>"#,
        points = points,
        tx = x + w / 3.0,
        ty = y + 3.0,
    );

    (svg, input_x, output_x)
}

fn draw_matching_network(x: f64, y: f64, gnd_y: f64, impedance_real: f64, impedance_imag: f64, frequency: f64) -> (String, f64, f64) {
    let omega = 2.0 * PI * frequency;
    let l_value = impedance_imag.abs() / omega;
    let c_value = 1.0 / (omega * impedance_real);

    let inductor_w = 30.0;
    let _cap_h = 15.0;
    let input_x = x;
    let output_x = x + inductor_w + 15.0;

    let coils: String = (0..4)
        .map(|i| {
            let cx = x + 5.0 + i as f64 * 7.0;
            format!(r#"<path d="M {},{} a 3.5,3 0 1 1 7,0" fill="none" stroke="{}" stroke-width="1.5"/>"#, cx, y, COMP)
        })
        .collect::<Vec<_>>()
        .join("\n");

    let cap_x = x + inductor_w + 7.5;
    let cap_top = y + 5.0;
    let cap_bot = gnd_y - 5.0;
    let cap_mid = (cap_top + cap_bot) / 2.0;
    let plate_gap = 3.0;

    let l_label = format_value(l_value, "H");
    let c_label = format_value(c_value, "F");

    let svg = format!(
        r#"{coils}
<line x1="{lx1}" y1="{y}" x2="{lx2}" y2="{y}" stroke="{WIRE}" stroke-width="1"/>
<line x1="{cap_x}" y1="{y}" x2="{cap_x}" y2="{cap_top}" stroke="{WIRE}" stroke-width="1"/>
<line x1="{plate_left}" y1="{plate_top}" x2="{plate_right}" y2="{plate_top}" stroke="{COMP}" stroke-width="2"/>
<line x1="{plate_left}" y1="{plate_bot}" x2="{plate_right}" y2="{plate_bot}" stroke="{COMP}" stroke-width="2"/>
<line x1="{cap_x}" y1="{cap_bot}" x2="{cap_x}" y2="{gnd_y}" stroke="{WIRE}" stroke-width="1"/>
<text x="{ltx}" y="{lty}" fill="{TEXT}" font-size="8" text-anchor="middle">{l_label}</text>
<text x="{ctx}" y="{cty}" fill="{TEXT}" font-size="8" text-anchor="start">{c_label}</text>"#,
        coils = coils,
        y = y,
        lx1 = x + inductor_w - 2.0,
        lx2 = output_x,
        cap_x = cap_x,
        cap_top = cap_top,
        plate_left = cap_x - 5.0,
        plate_right = cap_x + 5.0,
        plate_top = cap_mid - plate_gap,
        plate_bot = cap_mid + plate_gap,
        cap_bot = cap_bot,
        gnd_y = gnd_y,
        ltx = x + inductor_w / 2.0,
        lty = y - 8.0,
        ctx = cap_x + 8.0,
        cty = cap_mid + 3.0,
    );

    (svg, input_x, output_x)
}

fn draw_transducer(x: f64, y: f64, gnd_y: f64) -> (String, f64, f64) {
    let w = 20.0;
    let h = 25.0;
    let input_x = x;
    let output_x = x + w;
    let rect_top = y - h / 2.0;

    let svg = format!(
        r#"<rect x="{x}" y="{rect_top}" width="{w}" height="{h}" fill="none" stroke="{COMP}" stroke-width="1.5"/>
<line x1="{x}" y1="{diag_top}" x2="{diag_right}" y2="{diag_bot}" stroke="{COMP}" stroke-width="1"/>
<line x1="{cx}" y1="{rect_bot}" x2="{cx}" y2="{gnd_y}" stroke="{WIRE}" stroke-width="1"/>"#,
        x = x,
        rect_top = rect_top,
        w = w,
        h = h,
        diag_top = rect_top,
        diag_right = x + w,
        diag_bot = rect_top + h,
        cx = x + w / 2.0,
        rect_bot = rect_top + h,
        gnd_y = gnd_y,
    );

    (svg, input_x, output_x)
}

pub fn generate_circuit_svg(components: &[CircuitComponent], width: f64, height: f64) -> CircuitData {
    let margin = 20.0;
    let signal_y = height * 0.35;
    let gnd_y = height - margin;

    let num_components = components.len() as f64;
    let available_width = width - 2.0 * margin;
    let spacing = available_width / (num_components + 1.0);

    let mut svg_parts = Vec::new();
    let mut last_output_x: Option<f64> = None;
    let mut wire_segments = Vec::new();

    for (i, comp) in components.iter().enumerate() {
        let base_x = margin + spacing * (i as f64 + 0.5);

        let (comp_svg, input_x, output_x) = match comp {
            CircuitComponent::SignalGenerator { frequency, .. } => {
                draw_signal_generator(base_x, signal_y, *frequency)
            }
            CircuitComponent::Amplifier { gain } => {
                draw_amplifier(base_x, signal_y, *gain)
            }
            CircuitComponent::MatchingNetwork { impedance_real, impedance_imag, frequency } => {
                draw_matching_network(base_x, signal_y, gnd_y, *impedance_real, *impedance_imag, *frequency)
            }
            CircuitComponent::TransducerLoad { .. } => {
                draw_transducer(base_x, signal_y, gnd_y)
            }
        };

        svg_parts.push(comp_svg);

        if let Some(prev_x) = last_output_x {
            wire_segments.push(format!(
                r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="1"/>"#,
                prev_x, signal_y, input_x, signal_y, WIRE
            ));
        }

        last_output_x = Some(output_x);
    }

    let ground_svg = format!(
        r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="1.5"/>"#,
        margin, gnd_y, width - margin, gnd_y, WIRE
    );

    let full_svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">
{}
{}
{}
</svg>"#,
        width, height, width, height,
        wire_segments.join("\n"),
        svg_parts.join("\n"),
        ground_svg
    );

    CircuitData {
        width: width as f32,
        height: height as f32,
        svg: full_svg,
    }
}
