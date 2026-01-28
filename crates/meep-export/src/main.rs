//! mittens-to-meep: CLI tool for translating Mittens CAD to MEEP simulations

use anyhow::{Context, Result};
use clap::Parser;
use meep_export::{translate, TranslationConfig, LengthUnit};
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "mittens-to-meep")]
#[command(about = "Translate Mittens CAD geometry to MEEP FDTD simulations")]
#[command(version)]
struct Args {
    /// Input JSON file (Mittens serialized scene)
    #[arg(short, long)]
    input: PathBuf,

    /// Output Python file
    #[arg(short, long)]
    output: PathBuf,

    /// Resolution in pixels per length unit
    #[arg(long, default_value = "10")]
    resolution: f64,

    /// PML thickness in µm
    #[arg(long, default_value = "1000")]
    pml_thickness: f64,

    /// Center frequency in GHz
    #[arg(long, default_value = "5")]
    freq_center: f64,

    /// Frequency width in GHz
    #[arg(long, default_value = "4")]
    freq_width: f64,

    /// Cell padding in µm
    #[arg(long, default_value = "2000")]
    cell_padding: f64,

    /// Source length unit (m, mm, um, nm)
    #[arg(long, default_value = "mm")]
    source_unit: String,

    /// MEEP length unit (m, mm, um, nm)
    #[arg(long, default_value = "um")]
    meep_unit: String,

    /// Include field monitors
    #[arg(long, default_value = "true")]
    field_monitors: bool,

    /// Include flux monitors for S-parameters
    #[arg(long, default_value = "true")]
    flux_monitors: bool,

    /// Print generated script to stdout instead of file
    #[arg(long)]
    stdout: bool,
}

fn parse_unit(s: &str) -> Result<LengthUnit> {
    match s.to_lowercase().as_str() {
        "m" | "meter" | "meters" => Ok(LengthUnit::Meter),
        "mm" | "millimeter" | "millimeters" => Ok(LengthUnit::Millimeter),
        "um" | "µm" | "micrometer" | "micrometers" => Ok(LengthUnit::Micrometer),
        "nm" | "nanometer" | "nanometers" => Ok(LengthUnit::Nanometer),
        _ => anyhow::bail!("Unknown unit: {}. Use: m, mm, um, or nm", s),
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Read input
    let json = fs::read_to_string(&args.input)
        .with_context(|| format!("Failed to read input file: {:?}", args.input))?;

    // Build config
    let config = TranslationConfig {
        source_unit: parse_unit(&args.source_unit)?,
        meep_unit: parse_unit(&args.meep_unit)?,
        resolution: args.resolution,
        pml_thickness: args.pml_thickness,
        freq_center_hz: args.freq_center * 1e9,
        freq_width_hz: args.freq_width * 1e9,
        cell_padding: args.cell_padding,
        field_monitors: args.field_monitors,
        flux_monitors: args.flux_monitors,
        circular_segments: 32,
    };

    // Translate
    let script = translate(&json, &config)
        .context("Translation failed")?;

    // Output
    if args.stdout {
        println!("{}", script);
    } else {
        fs::write(&args.output, &script)
            .with_context(|| format!("Failed to write output file: {:?}", args.output))?;
        eprintln!("Generated MEEP script: {:?}", args.output);
    }

    Ok(())
}
