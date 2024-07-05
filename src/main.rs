use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "straycrab")]
#[command(version = "0.1.0")]
#[command(about = "WORLD-based UTAU resampler on Rust.")]
struct ResamplerArgs {
    in_file: PathBuf,
    out_file: PathBuf,
    pitch: i64,
    velocity: f64,
    #[arg(default_value_t = String::from(""))]
    flags: String,
    #[arg(default_value_t = 0.)]
    offset: f64,
    #[arg(default_value_t = 1000.)]
    length: f64,
    #[arg(default_value_t = 0.)]
    consonant: f64,
    #[arg(default_value_t = 0.)]
    cutoff: f64,
    #[arg(default_value_t = 100.)]
    volume: f64,
    #[arg(default_value_t = 0.)]
    modulation: f64,
    #[arg(default_value_t = 100., value_parser = tempo_parser)]
    tempo: f64,
    pitchbend: String,
}

fn tempo_parser(arg: &str) -> Result<f64> {
    let tempo: f64 = arg[1..].parse()?;
    Ok(tempo)
}

fn main() {
    let args = ResamplerArgs::parse();
    println!("{}", args.in_file.to_str().unwrap());
}
