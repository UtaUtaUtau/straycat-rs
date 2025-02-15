use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use crate::consts;
use anyhow::Result;
use rsworld::{cheaptrick, code_aperiodicity, code_spectral_envelope, d4c, harvest};
use rsworld_sys::{CheapTrickOption, D4COption, HarvestOption};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct WorldFeatures {
    // for UTAU modulation
    pub base_f0: f64,
    // Actual WORLD features
    pub f0: Vec<f64>,
    pub mgc: Vec<Vec<f64>>,
    pub bap: Vec<Vec<f64>>,
}

pub fn to_feature_path<P: AsRef<Path>>(path: P) -> PathBuf {
    // Converts any path to the feature file path
    let path = path.as_ref();
    let mut fname = path.file_stem().unwrap().to_owned();
    fname.push("_wav");
    path.with_file_name(fname)
        .with_extension(consts::FEATURE_EXT)
}

fn calculate_base_f0(f0: &Vec<f64>) -> f64 {
    // Get base F0. Averages the whole F0 curve with strong bias on flat areas.
    let n = f0.len();
    let mut base_f0 = 0.;
    let mut tally = 0.;

    for i in 0..n {
        if f0[i] >= consts::F0_FLOOR && f0[i] <= consts::F0_CEIL {
            let q = if i == 0 {
                f0[1] - f0[0]
            } else if i == n - 1 {
                f0[n - 2] - f0[n - 1]
            } else {
                0.5 * (f0[i + 1] - f0[i - 1])
            };

            let weight = (-q * q).exp2(); // Quicker bell curve
            base_f0 += f0[i] * weight;
            tally += weight;
        }
    }

    if tally > 0. {
        base_f0 /= tally;
    }
    base_f0
}

pub fn generate_features<P: AsRef<Path>>(
    path: P,
    audio: Vec<f64>,
    threshold: Option<f64>,
) -> Result<WorldFeatures> {
    // Generate all required WORLD features
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
        threshold: threshold.unwrap_or(consts::D4C_THRESHOLD),
    };

    let (t, f0) = harvest(&audio, consts::SAMPLE_RATE as i32, &harvest_opts);
    let sp = cheaptrick(
        &audio,
        consts::SAMPLE_RATE as i32,
        &t,
        &f0,
        &mut cheaptrick_opts,
    );

    let mut ap = d4c(&audio, consts::SAMPLE_RATE as i32, &t, &f0, &d4c_opts);

    // Ensure no NaNs are present in AP. Happens when a signal doesn't have higher frequencies.
    // It should be safe to assume that it does not have aperiodicity in those frequencies.
    ap.iter_mut().for_each(|ap_frame| {
        ap_frame.iter_mut().for_each(|a| {
            if a.is_nan() {
                *a = 0.
            }
        })
    });

    let base_f0 = calculate_base_f0(&f0);

    // Code features to reduce feature file size
    let mgc = code_spectral_envelope(
        &sp,
        f0.len() as i32,
        consts::SAMPLE_RATE as i32,
        consts::FFT_SIZE,
        consts::MGC_DIMS,
    );
    let bap = code_aperiodicity(&ap, f0.len() as i32, consts::SAMPLE_RATE as i32);

    let features = WorldFeatures {
        base_f0,
        f0,
        mgc,
        bap,
    };

    let feature_path = to_feature_path(path);
    let bin = bincode::serialize(&features)?;

    let mut feature_file = File::create(feature_path)?;
    feature_file.write_all(&bin)?;
    Ok(features)
}

pub fn read_features<P: AsRef<Path>>(path: P) -> Result<WorldFeatures> {
    // Read WORLD feature file
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
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;
    use std::time::Instant;

    #[test]
    fn test_world() {
        let path = Path::new("test/test.wav");
        let feature_path = path.with_extension(consts::FEATURE_EXT);
        let synth_path = path.with_extension("syn.wav");
        let audio = read_audio(path).expect("Cannot read audio");
        println!("gt: {}", audio.len());

        let now = Instant::now();
        let features =
            generate_features(&path, audio, None).expect("Cannot generate WORLD features");
        println!("Feature Generation: {:.2?}", now.elapsed());
        let now = Instant::now();
        let features = read_features(&feature_path).expect("Cannot read WORLD features");
        println!("Read features from file: {:.2?}", now.elapsed());
        let f0_length = features.f0.len() as i32;

        let sp = decode_spectral_envelope(
            &features.mgc,
            f0_length,
            consts::SAMPLE_RATE as i32,
            consts::FFT_SIZE,
        );

        println!("sp shape: ({}, {})", sp.len(), sp[0].len());
        let ap = decode_aperiodicity(&features.bap, f0_length, consts::SAMPLE_RATE as i32);

        println!("{}, {}", sp.len(), sp[0].len());
        let syn = synthesis(
            &features.f0,
            &sp,
            &ap,
            consts::FRAME_PERIOD,
            consts::SAMPLE_RATE as i32,
        );
        println!("synthesis: {}", syn.len());

        write_audio(synth_path, &syn).expect("Cannot write");
    }

    #[test]
    fn test_decode_features() {
        let test_path =
            r"F:\funny personal\utau folder\demo usts\あめふり.cache\35_i+ぴ_C5_dDhIWv.wav.sc";
        let features = read_features(test_path).expect("Cannot read features");

        let ap = decode_aperiodicity(
            &features.bap,
            features.f0.len() as i32,
            consts::SAMPLE_RATE as i32,
        );

        let mut ap_file = File::create("test/ap.csv").expect("Cannot create file");
        let ap_csv = ap
            .into_iter()
            .map(|line| {
                line.into_iter()
                    .map(|x| format!("{}", x))
                    .collect::<Vec<String>>()
                    .join(",")
            })
            .collect::<Vec<String>>()
            .join("\n");
        ap_file.write_all(ap_csv.as_bytes()).expect("Cannot write");

        let sp = decode_spectral_envelope(
            &features.mgc,
            features.f0.len() as i32,
            consts::SAMPLE_RATE as i32,
            consts::FFT_SIZE,
        );

        let mut sp_file = File::create("test/sp.csv").expect("Cannot create file");
        let sp_csv = sp
            .into_iter()
            .map(|line| {
                line.into_iter()
                    .map(|x| format!("{}", x))
                    .collect::<Vec<String>>()
                    .join(",")
            })
            .collect::<Vec<String>>()
            .join("\n");
        sp_file.write_all(sp_csv.as_bytes()).expect("Cannot write");

        let mut f0_file = File::create("test/f0.csv").expect("Cannot create file");
        let f0_csv = features
            .f0
            .iter()
            .map(|x| format!("{}", x))
            .collect::<Vec<String>>()
            .join("\n");
        f0_file.write_all(f0_csv.as_bytes()).expect("Cannot write");
    }
}
