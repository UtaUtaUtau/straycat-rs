mod audio;
mod consts;
mod interpolator;
mod parser;
mod resample;
mod util;
mod world;
use clap::Parser;
use parser::ResamplerArgs;
use resample::run;

fn main() {
    let args = ResamplerArgs::parse();
    run(args);
}
