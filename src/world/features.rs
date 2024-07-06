use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

use anyhow::{anyhow, Result};
use rsworld::{cheaptrick, code_aperiodicity, code_spectral_envelope, d4c, harvest};
use rsworld_sys::{CheapTrickOption, D4COption, HarvestOption};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct WorldFeatures {
    f0: Vec<f64>,
    mgc: Vec<Vec<f64>>,
    bap: Vec<Vec<f64>>,
    fft_size: i32,
    time_step: f64,
}

pub fn generate_features<P: AsRef<Path>>(path: P, audio: Vec<f64>) -> Result<WorldFeatures> {
    let harvest_opts = HarvestOption {
        f0_floor: 71.,
        f0_ceil: 1760.,
        frame_period: 5.,
    };

    let mut cheaptrick_opts = CheapTrickOption {
        q1: -0.15,
        f0_floor: harvest_opts.f0_floor,
        fft_size: 2048,
    };

    let d4c_opts = D4COption { threshold: 0.1 };

    let (t, f0) = harvest(&audio, 44100, &harvest_opts);
    let sp = cheaptrick(&audio, 44100, &t, &f0, &mut cheaptrick_opts);
    let ap = d4c(&audio, 44100, &t, &f0, &d4c_opts);

    println!("{}, {}", sp.len(), sp[0].len());

    let mgc = code_spectral_envelope(&sp, f0.len() as i32, 44100, cheaptrick_opts.fft_size, 64);
    let bap = code_aperiodicity(&ap, f0.len() as i32, 44100);

    let features = WorldFeatures {
        f0,
        mgc,
        bap,
        fft_size: cheaptrick_opts.fft_size,
        time_step: harvest_opts.frame_period,
    };

    let feature_path = path.as_ref().with_extension("wav.sc");
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

    use crate::audio::read_write::{read_audio, write_audio};

    use super::{generate_features, read_features};
    use std::path::Path;

    #[test]
    fn test_world() {
        let path = Path::new("test/paul.wav");
        let feature_path = path.with_extension("wav.sc");
        let synth_path = path.with_extension("syn.wav");
        let audio = read_audio(path, None).expect("Cannot read audio");
        println!("gt: {}", audio.len());

        let features = generate_features(&path, audio).expect("Cannot generate WORLD features");
        let features = read_features(&feature_path).expect("Cannot read WORLD features");
        let f0_length = features.f0.len() as i32;

        let mut sp = decode_spectral_envelope(&features.mgc, f0_length, 44100, features.fft_size);
        let mut ap = decode_aperiodicity(&features.bap, f0_length, 44100);

        println!("{}, {}", sp.len(), sp[0].len());
        let syn = synthesis(&features.f0, &sp, &ap, features.time_step, 44100);
        println!("synthesis: {}", syn.len());

        write_audio(synth_path, &syn);
    }
}
