use crate::consts;
use crate::interpolator::interp::{Interpolator, Lanczos};
use anyhow::{anyhow, Result};
use biquad::{Biquad, Coefficients, DirectForm2Transposed, ToHertz};
use hound::{SampleFormat, WavSpec, WavWriter};
use std::{fs::File, path::Path};
use symphonia::{
    core::{
        audio::SampleBuffer, codecs::DecoderOptions, errors::Error, formats::FormatOptions,
        io::MediaSourceStream, meta::MetadataOptions, probe::Hint,
    },
    default::{get_codecs, get_probe},
};

fn resample_audio(
    audio: Vec<f64>,
    in_fs: u32,
    out_fs: u32,
    lanczos_size: Option<f64>,
) -> Result<Vec<f64>> {
    let in_samples = audio.len();
    let out_samples = (in_samples as f64 * out_fs as f64 / in_fs as f64) as usize;
    let mut resampled: Vec<f64> = Vec::with_capacity(out_samples);

    let interp = Lanczos::new(audio, lanczos_size);

    for i in 0..out_samples {
        let x = in_samples as f64 * i as f64 / out_samples as f64;
        resampled.push(interp.sample(x));
    }

    if in_fs < out_fs {
        let biquad_coeffs = match Coefficients::<f64>::from_params(
            biquad::Type::LowPass,
            out_fs.hz(),
            (0.5 * in_fs as f64).hz(),
            biquad::Q_BUTTERWORTH_F64,
        ) {
            Ok(coeffs) => coeffs,
            Err(_) => return Err(anyhow!("Error setting up biquad filter")),
        };

        let mut biquad = DirectForm2Transposed::<f64>::new(biquad_coeffs);

        for _ in 0..4 {
            biquad.reset_state();
            for s in &mut resampled {
                *s = biquad.run(*s);
            }
        }
    }
    Ok(resampled)
}

pub fn read_audio<P: AsRef<Path>>(path: P, lanczos_size: Option<f64>) -> Result<Vec<f64>> {
    let ext = path.as_ref().extension().unwrap().to_str().unwrap();

    let source = File::open(path.as_ref())?;

    let mss = MediaSourceStream::new(Box::new(source), Default::default());

    let mut hint = Hint::new();
    hint.with_extension(ext);

    let format_opts: FormatOptions = Default::default();
    let metadata_opts: MetadataOptions = Default::default();
    let decoder_opts: DecoderOptions = Default::default();

    let probed = get_probe().format(&hint, mss, &format_opts, &metadata_opts)?;

    let mut format = probed.format;

    let track = format.default_track().unwrap();

    let mut decoder = get_codecs().make(&track.codec_params, &decoder_opts)?;

    let track_id = track.id;

    let mut audio: Vec<f64> = Vec::new();
    let mut channels = 1;
    let mut fs = 1;
    let mut packet_buffer = None;
    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(_) => break,
        };

        if packet.track_id() != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(decoded_packet) => {
                if packet_buffer.is_none() {
                    let spec = *decoded_packet.spec();
                    channels = spec.channels.count();
                    fs = spec.rate;
                    let duration = decoded_packet.capacity() as u64;

                    packet_buffer = Some(SampleBuffer::<f64>::new(duration, spec));
                }

                if let Some(buffer) = &mut packet_buffer {
                    buffer.copy_interleaved_ref(decoded_packet);
                    let samples = buffer.samples();
                    for s in 0..samples.len() / channels {
                        let mut a = 0.;
                        for c in 0..channels {
                            a += samples[s * channels + c];
                        }
                        audio.push(a / channels as f64);
                    }
                }
            }
            Err(Error::DecodeError(_)) => continue,
            Err(_) => break,
        }
    }
    if fs == consts::SAMPLE_RATE {
        Ok(audio)
    } else {
        resample_audio(audio, fs, consts::SAMPLE_RATE, lanczos_size)
    }
}

pub fn write_audio<P: AsRef<Path>>(path: P, audio: &Vec<f64>) -> Result<()> {
    let out_spec = WavSpec {
        channels: 1,
        sample_rate: consts::SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut writer = WavWriter::create(path.as_ref(), out_spec)?;
    let mut scaled_audio: Vec<f64> = audio
        .into_iter()
        .map(|x| (x * i16::MAX as f64).clamp(i16::MIN as f64, i16::MAX as f64))
        .collect();
    for s in 0..scaled_audio.len() {
        if s + 1 < scaled_audio.len() {
            let q = scaled_audio[s].clamp(i16::MIN as f64, i16::MAX as f64) as i16;
            let error = scaled_audio[s] - q as f64;
            scaled_audio[s + 1] += error;
            writer.write_sample(q)?;
        } else {
            writer.write_sample(scaled_audio[s] as i16)?;
        }
    }
    writer.finalize()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{read_audio, write_audio};
    use std::path::Path;

    #[test]
    fn test_read_write() {
        let test_paths = vec![
            "test/01.wav",
            "test/pjs001.wav",
            "test/paul.wav",
            "test/ano ko wa akuma solanri.wav",
            "test/ano ko wa akuma solanri.mp3",
            "test/res.wav",
        ];
        let test_paths: Vec<&Path> = test_paths.into_iter().map(|x| Path::new(x)).collect();
        for path in test_paths {
            println!("Testing {:?}", path.as_os_str());
            let mut out_fname = path.file_name().expect("Failed to get filename").to_owned();
            out_fname.push(".out.wav");
            let out_path = path.with_file_name(out_fname);

            let audio = read_audio(path, None).expect("Failed to read file");
            write_audio(out_path, &audio).expect("Failed to write audio");
        }
    }
}
