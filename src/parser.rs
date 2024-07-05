use crate::util::{pitch_parser, tempo_parser};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "straycrab")]
#[command(version = "0.1.0")]
#[command(about = "WORLD-based UTAU resampler on Rust.")]
pub struct ResamplerArgs {
    pub in_file: PathBuf,
    pub out_file: PathBuf,
    #[arg(value_parser = pitch_parser)]
    pub pitch: i32,
    pub velocity: f64,
    #[arg(default_value_t = String::from(""))]
    pub flags: String,
    #[arg(default_value_t = 0.)]
    pub offset: f64,
    #[arg(default_value_t = 1000.)]
    pub length: f64,
    #[arg(default_value_t = 0.)]
    pub consonant: f64,
    #[arg(default_value_t = 0.)]
    pub cutoff: f64,
    #[arg(default_value_t = 100.)]
    pub volume: f64,
    #[arg(default_value_t = 0.)]
    pub modulation: f64,
    #[arg(value_parser = tempo_parser, default_value_t = 100.)]
    pub tempo: f64,
    #[arg(default_value_t = String::from("AA"))]
    pub pitchbend: String,
}
