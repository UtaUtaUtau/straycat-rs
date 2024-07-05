mod audio;
mod interpolator;
mod parser;
mod resample;
mod util;
use clap::Parser;
use parser::ResamplerArgs;
use resample::run;

fn main() {
    let args = ResamplerArgs::parse();
    run(args);
}
