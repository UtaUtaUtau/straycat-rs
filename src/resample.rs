use crate::audio::read_write::read_audio;
use crate::consts;
use crate::parser::ResamplerArgs;
use crate::world::features::{generate_features, read_features};
use anyhow::Result;

pub fn run(args: ResamplerArgs) -> Result<()> {
    let feature_path = args.in_file.with_extension(consts::FEATURE_EXT);
    let features = if feature_path.exists() {
        read_features(&feature_path)?
    } else {
        let audio = read_audio(&args.in_file, None)?;
        generate_features(&args.in_file, audio)?
    };

    Ok(())
}
