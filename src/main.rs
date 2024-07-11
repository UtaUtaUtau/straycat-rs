// Module import
mod audio;
mod consts;
mod flags;
mod interpolator;
mod parser;
mod pitchbend;
mod resample;
mod util;
mod world;
use clap::Parser;
use parser::ResamplerArgs;
use resample::run;

fn main() {
    let args = ResamplerArgs::parse(); // Parse arguments using clap
    run(args).expect("Cannot render note"); // "Resample"
}
