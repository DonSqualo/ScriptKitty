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

// ===========================

pub struct CircuitAnalysis {
    pub frequency: f64,
    pub input_impedance: (f64, f64),
    pub output_impedance: (f64, f64),
    pub voltage_gain_db: f64,
    pub power_transfer_efficiency: f64,
    pub s11_db: f64,
}

fn complex_add(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    (a.0 + b.0, a.1 + b.1)
}

fn complex_sub(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    (a.0 - b.0, a.1 - b.1)
}

fn complex_mul(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0)
}

fn complex_div(num: (f64, f64), den: (f64, f64)) -> (f64, f64) {
    let den_mag_sq = den.0 * den.0 + den.1 * den.1;
    let real = (num.0 * den.0 + num.1 * den.1) / den_mag_sq;
    let imag = (num.1 * den.0 - num.0 * den.1) / den_mag_sq;
    (real, imag)
}

fn complex_mag(z: (f64, f64)) -> f64 {
    (z.0 * z.0 + z.1 * z.1).sqrt()
}

fn complex_parallel(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    let num = complex_mul(a, b);
    let den = complex_add(a, b);
    complex_div(num, den)
}

pub fn analyze_circuit_ac(components: &[CircuitComponent], frequency: f64) -> CircuitAnalysis {
    let omega = 2.0 * PI * frequency;
    let z_source = 50.0;

    let mut z_load = (50.0, 0.0);
    for comp in components.iter().rev() {
        if let CircuitComponent::TransducerLoad { impedance_real, impedance_imag } = comp {
            z_load = (*impedance_real, *impedance_imag);
            break;
        }
    }

    let mut z_current = z_load;

    for comp in components.iter().rev() {
        match comp {
            CircuitComponent::MatchingNetwork { impedance_real, impedance_imag, frequency: match_freq } => {
                let match_omega = 2.0 * PI * match_freq;
                let l_value = impedance_imag.abs() / match_omega;
                let c_value = 1.0 / (match_omega * impedance_real);

                let x_l = omega * l_value;
                let x_c = -1.0 / (omega * c_value);

                let z_c = (0.0, x_c);
                let z_parallel = complex_parallel(z_current, z_c);

                z_current = (z_parallel.0, z_parallel.1 + x_l);
            }
            CircuitComponent::TransducerLoad { .. } => {}
            CircuitComponent::Amplifier { .. } | CircuitComponent::SignalGenerator { .. } => {}
        }
    }

    let z_in = z_current;

    let s11_num = complex_sub(z_in, (z_source, 0.0));
    let s11_den = complex_add(z_in, (z_source, 0.0));
    let s11 = complex_div(s11_num, s11_den);
    let s11_mag = complex_mag(s11);
    let s11_db = 20.0 * s11_mag.log10();

    let power_transfer = 1.0 - s11_mag * s11_mag;

    let mut voltage_gain = 1.0;
    for comp in components {
        if let CircuitComponent::Amplifier { gain } = comp {
            voltage_gain *= gain;
        }
    }
    let voltage_gain_db = 20.0 * voltage_gain.log10();

    CircuitAnalysis {
        frequency,
        input_impedance: z_in,
        output_impedance: z_load,
        voltage_gain_db,
        power_transfer_efficiency: power_transfer,
        s11_db,
    }
}

// ===========================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_value_millis() {
        assert_eq!(format_value(0.001, "H"), "1.0mH");
        assert_eq!(format_value(0.0025, "F"), "2.5mF");
    }

    #[test]
    fn test_format_value_micros() {
        assert_eq!(format_value(0.000001, "H"), "1.0uH");
        assert_eq!(format_value(0.0000047, "F"), "4.7uF");
    }

    #[test]
    fn test_format_value_nanos() {
        assert_eq!(format_value(0.000000001, "H"), "1.0nH");
        assert_eq!(format_value(0.00000001, "F"), "10.0nF");
    }

    #[test]
    fn test_format_value_picos() {
        assert_eq!(format_value(0.000000000001, "F"), "1.0pF");
        assert_eq!(format_value(0.00000000022, "F"), "220.0pF");
    }

    #[test]
    fn test_signal_generator_creates_circle_with_sine() {
        let (svg, input_x, output_x) = draw_signal_generator(100.0, 50.0, 1e6);

        assert!(svg.contains("<circle"), "SignalGenerator should contain circle");
        assert!(svg.contains("r=\"15\""), "Circle should have radius 15");
        assert!(svg.contains("<polyline"), "SignalGenerator should contain sine wave polyline");
        assert!(svg.contains("1.0MHz"), "Should display frequency label");
        assert!((input_x - 100.0).abs() < 1e-6, "Input should be at x position");
        assert!((output_x - 130.0).abs() < 1e-6, "Output should be at x + 2*radius");
    }

    #[test]
    fn test_signal_generator_frequency_labels() {
        let (svg_mhz, _, _) = draw_signal_generator(0.0, 0.0, 2.5e6);
        assert!(svg_mhz.contains("2.5MHz"), "Should format MHz correctly");

        let (svg_khz, _, _) = draw_signal_generator(0.0, 0.0, 500e3);
        assert!(svg_khz.contains("500.0kHz"), "Should format kHz correctly");

        let (svg_hz, _, _) = draw_signal_generator(0.0, 0.0, 60.0);
        assert!(svg_hz.contains("60.0Hz"), "Should format Hz correctly");
    }

    #[test]
    fn test_amplifier_creates_triangle() {
        let (svg, input_x, output_x) = draw_amplifier(100.0, 50.0, 10.0);

        assert!(svg.contains("<polygon"), "Amplifier should contain polygon (triangle)");
        assert!(svg.contains("points=\""), "Polygon should have points attribute");
        assert!(svg.contains("x10"), "Should display gain label");
        assert!((input_x - 100.0).abs() < 1e-6, "Input should be at x position");
        assert!((output_x - 130.0).abs() < 1e-6, "Output should be at x + width (30)");
    }

    #[test]
    fn test_amplifier_triangle_points() {
        let x = 50.0;
        let y = 100.0;
        let (svg, _, _) = draw_amplifier(x, y, 5.0);

        let h = 24.0;
        let w = 30.0;
        let expected_top = format!("{},{}", x, y - h / 2.0);
        let expected_tip = format!("{},{}", x + w, y);
        let expected_bottom = format!("{},{}", x, y + h / 2.0);

        assert!(svg.contains(&expected_top), "Triangle should have top vertex");
        assert!(svg.contains(&expected_tip), "Triangle should have tip vertex");
        assert!(svg.contains(&expected_bottom), "Triangle should have bottom vertex");
    }

    #[test]
    fn test_matching_network_creates_l_network() {
        let (svg, input_x, output_x) = draw_matching_network(100.0, 50.0, 150.0, 50.0, 100.0, 1e6);

        assert!(svg.contains("<path"), "MatchingNetwork should contain inductor coils (path elements)");
        assert!(svg.contains("a 3.5,3"), "Inductor should use arc paths for coils");
        assert!(svg.contains("stroke-width=\"2\""), "Capacitor plates should have stroke-width 2");
        assert!((input_x - 100.0).abs() < 1e-6, "Input should be at x position");
        assert!((output_x - 145.0).abs() < 1e-6, "Output should be at x + inductor_w + 15");
    }

    #[test]
    fn test_matching_network_computes_l_value() {
        let frequency = 1e6;
        let impedance_imag = 100.0;
        let omega = 2.0 * PI * frequency;
        let expected_l = impedance_imag / omega;

        let (svg, _, _) = draw_matching_network(0.0, 50.0, 150.0, 50.0, impedance_imag, frequency);

        let l_label = format_value(expected_l, "H");
        assert!(svg.contains(&l_label), "Should display computed inductance value: {}", l_label);
    }

    #[test]
    fn test_matching_network_computes_c_value() {
        let frequency = 1e6;
        let impedance_real = 50.0;
        let omega = 2.0 * PI * frequency;
        let expected_c = 1.0 / (omega * impedance_real);

        let (svg, _, _) = draw_matching_network(0.0, 50.0, 150.0, impedance_real, 100.0, frequency);

        let c_label = format_value(expected_c, "F");
        assert!(svg.contains(&c_label), "Should display computed capacitance value: {}", c_label);
    }

    #[test]
    fn test_impedance_to_lc_formulas() {
        let frequency: f64 = 2e6;
        let impedance_real: f64 = 75.0;
        let impedance_imag: f64 = 200.0;
        let omega = 2.0 * PI * frequency;

        let l_value = impedance_imag.abs() / omega;
        let c_value = 1.0 / (omega * impedance_real);

        let expected_l = 200.0 / (2.0 * PI * 2e6);
        let expected_c = 1.0 / (2.0 * PI * 2e6 * 75.0);

        assert!((l_value - expected_l).abs() < 1e-12, "L formula: X_L = omega * L => L = X_L / omega");
        assert!((c_value - expected_c).abs() < 1e-18, "C formula: X_C = 1/(omega*C) => C = 1/(omega*R)");
    }

    #[test]
    fn test_transducer_creates_rectangle() {
        let (svg, input_x, output_x) = draw_transducer(100.0, 50.0, 150.0);

        assert!(svg.contains("<rect"), "Transducer should contain rectangle");
        assert!(svg.contains("width=\"20\""), "Rectangle should have width 20");
        assert!(svg.contains("height=\"25\""), "Rectangle should have height 25");
        assert!(svg.contains("<line"), "Transducer should have ground connection line");
        assert!((input_x - 100.0).abs() < 1e-6, "Input should be at x position");
        assert!((output_x - 120.0).abs() < 1e-6, "Output should be at x + width (20)");
    }

    #[test]
    fn test_generate_circuit_svg_basic_structure() {
        let components = vec![
            CircuitComponent::SignalGenerator { frequency: 1e6, amplitude: 1.0 },
        ];
        let result = generate_circuit_svg(&components, 400.0, 200.0);

        assert!((result.width - 400.0).abs() < 1e-6, "Width should match");
        assert!((result.height - 200.0).abs() < 1e-6, "Height should match");
        assert!(result.svg.contains("<svg"), "Should contain SVG opening tag");
        assert!(result.svg.contains("</svg>"), "Should contain SVG closing tag");
        assert!(result.svg.contains("viewBox=\"0 0 400 200\""), "Should have correct viewBox");
    }

    #[test]
    fn test_generate_circuit_svg_full_chain() {
        let components = vec![
            CircuitComponent::SignalGenerator { frequency: 1e6, amplitude: 1.0 },
            CircuitComponent::Amplifier { gain: 10.0 },
            CircuitComponent::MatchingNetwork { impedance_real: 50.0, impedance_imag: 100.0, frequency: 1e6 },
            CircuitComponent::TransducerLoad { impedance_real: 50.0, impedance_imag: -100.0 },
        ];
        let result = generate_circuit_svg(&components, 600.0, 200.0);

        assert!(result.svg.contains("<circle"), "Should contain signal generator circle");
        assert!(result.svg.contains("<polygon"), "Should contain amplifier triangle");
        assert!(result.svg.contains("<path"), "Should contain inductor paths");
        assert!(result.svg.contains("<rect"), "Should contain transducer rectangle");
    }

    #[test]
    fn test_generate_circuit_svg_ground_line() {
        let components = vec![
            CircuitComponent::SignalGenerator { frequency: 1e6, amplitude: 1.0 },
        ];
        let result = generate_circuit_svg(&components, 400.0, 200.0);

        let margin = 20.0;
        let gnd_y = 200.0 - margin;
        assert!(result.svg.contains(&format!("y1=\"{}\"", gnd_y)), "Ground line should be at correct y position");
        assert!(result.svg.contains(&format!("y2=\"{}\"", gnd_y)), "Ground line should span horizontally");
    }

    #[test]
    fn test_circuit_data_to_binary() {
        let data = CircuitData {
            width: 100.0,
            height: 50.0,
            svg: "<svg></svg>".to_string(),
        };

        let binary = data.to_binary();

        assert!(binary.starts_with(b"CIRCUIT\0"), "Binary should start with magic header");
        let width = f32::from_le_bytes([binary[8], binary[9], binary[10], binary[11]]);
        let height = f32::from_le_bytes([binary[12], binary[13], binary[14], binary[15]]);
        assert!((width - 100.0).abs() < 1e-6, "Width should be encoded correctly");
        assert!((height - 50.0).abs() < 1e-6, "Height should be encoded correctly");

        let svg_len = u32::from_le_bytes([binary[16], binary[17], binary[18], binary[19]]) as usize;
        assert_eq!(svg_len, 11, "SVG length should be encoded correctly");
        assert_eq!(&binary[20..], b"<svg></svg>", "SVG content should be appended");
    }

    #[test]
    fn test_wire_connections_between_components() {
        let components = vec![
            CircuitComponent::SignalGenerator { frequency: 1e6, amplitude: 1.0 },
            CircuitComponent::Amplifier { gain: 10.0 },
        ];
        let result = generate_circuit_svg(&components, 400.0, 200.0);

        let wire_count = result.svg.matches(&format!("stroke=\"{}\"", WIRE)).count();
        assert!(wire_count >= 2, "Should have wire connections between components and ground");
    }

    #[test]
    fn test_analyze_circuit_ac_matched_load() {
        let components = vec![
            CircuitComponent::SignalGenerator { frequency: 1e6, amplitude: 1.0 },
            CircuitComponent::TransducerLoad { impedance_real: 50.0, impedance_imag: 0.0 },
        ];

        let analysis = analyze_circuit_ac(&components, 1e6);

        assert!((analysis.input_impedance.0 - 50.0).abs() < 1e-6);
        assert!(analysis.input_impedance.1.abs() < 1e-6);
        assert!(analysis.s11_db < -40.0);
        assert!((analysis.power_transfer_efficiency - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_analyze_circuit_ac_with_amplifier_gain() {
        let components = vec![
            CircuitComponent::SignalGenerator { frequency: 1e6, amplitude: 1.0 },
            CircuitComponent::Amplifier { gain: 10.0 },
            CircuitComponent::TransducerLoad { impedance_real: 50.0, impedance_imag: 0.0 },
        ];

        let analysis = analyze_circuit_ac(&components, 1e6);

        assert!((analysis.voltage_gain_db - 20.0).abs() < 0.1);
    }

    #[test]
    fn test_analyze_circuit_ac_mismatched_load_reflection() {
        let components = vec![
            CircuitComponent::SignalGenerator { frequency: 1e6, amplitude: 1.0 },
            CircuitComponent::TransducerLoad { impedance_real: 200.0, impedance_imag: 0.0 },
        ];

        let analysis = analyze_circuit_ac(&components, 1e6);

        assert!(analysis.s11_db > -10.0);
        assert!(analysis.power_transfer_efficiency < 0.7);
    }
}
