use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

use crate::consts;
use anyhow::Result;
use rsworld::{cheaptrick, code_aperiodicity, code_spectral_envelope, d4c, harvest};
use rsworld_sys::{CheapTrickOption, D4COption, HarvestOption};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct WorldFeatures {
    base_f0: f64,
    f0: Vec<f64>,
    mgc: Vec<Vec<f64>>,
    bap: Vec<Vec<f64>>,
    fft_size: i32,
    time_step: f64,
}

fn calculate_base_f0(f0: &Vec<f64>) -> f64 {
    0.
}

pub fn generate_features<P: AsRef<Path>>(path: P, audio: Vec<f64>) -> Result<WorldFeatures> {
    let harvest_opts = HarvestOption {
        f0_floor: consts::F0_FLOOR,
        f0_ceil: consts::F0_CEIL,
        frame_period: consts::FRAME_PERIOD,
    };

    let mut cheaptrick_opts = CheapTrickOption {
        q1: consts::SPEC_Q1,
        f0_floor: consts::F0_FLOOR,
        fft_size: consts::FFT_SIZE,
    };

    let d4c_opts = D4COption {
        threshold: consts::D4C_THRESHOLD,
    };

    let (t, f0) = harvest(&audio, consts::SAMPLE_RATE as i32, &harvest_opts);
    let sp = cheaptrick(
        &audio,
        consts::SAMPLE_RATE as i32,
        &t,
        &f0,
        &mut cheaptrick_opts,
    );
    let ap = d4c(&audio, consts::SAMPLE_RATE as i32, &t, &f0, &d4c_opts);

    let base_f0 = calculate_base_f0(&f0);

    let mgc = code_spectral_envelope(
        &sp,
        f0.len() as i32,
        consts::SAMPLE_RATE as i32,
        cheaptrick_opts.fft_size,
        64,
    );
    let bap = code_aperiodicity(&ap, f0.len() as i32, consts::SAMPLE_RATE as i32);

    let features = WorldFeatures {
        base_f0,
        f0,
        mgc,
        bap,
        fft_size: cheaptrick_opts.fft_size,
        time_step: harvest_opts.frame_period,
    };

    let feature_path = path.as_ref().with_extension(consts::FEATURE_EXT);
    let bin = bincode::serialize(&features)?;

    let mut feature_file = File::create(feature_path)?;
    feature_file.write_all(&bin)?;
    Ok(features)
}

pub fn read_features<P: AsRef<Path>>(path: P) -> Result<WorldFeatures> {
    let mut bin = Vec::new();
    let mut f = File::open(path)?;
    f.read_to_end(&mut bin)?;

    let features: WorldFeatures = bincode::deserialize(&bin)?;
    Ok(features)
}

#[cfg(test)]
mod tests {
    use rsworld::{decode_aperiodicity, decode_spectral_envelope, synthesis};

    use super::{generate_features, read_features};
    use crate::audio::read_write::{read_audio, write_audio};
    use crate::consts;
    use std::path::Path;

    #[test]
    fn test_world() {
        let path = Path::new("test/paul.wav");
        let feature_path = path.with_extension(consts::FEATURE_EXT);
        let synth_path = path.with_extension("syn.wav");
        let audio = read_audio(path, None).expect("Cannot read audio");
        println!("gt: {}", audio.len());

        let features = generate_features(&path, audio).expect("Cannot generate WORLD features");
        let features = read_features(&feature_path).expect("Cannot read WORLD features");
        let f0_length = features.f0.len() as i32;

        let mut sp = decode_spectral_envelope(
            &features.mgc,
            f0_length,
            consts::SAMPLE_RATE as i32,
            features.fft_size,
        );
        let mut ap = decode_aperiodicity(&features.bap, f0_length, consts::SAMPLE_RATE as i32);

        println!("{}, {}", sp.len(), sp[0].len());
        let syn = synthesis(
            &features.f0,
            &sp,
            &ap,
            features.time_step,
            consts::SAMPLE_RATE as i32,
        );
        println!("synthesis: {}", syn.len());

        write_audio(synth_path, &syn);
    }
}
