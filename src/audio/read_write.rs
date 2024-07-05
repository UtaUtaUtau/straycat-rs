use anyhow::{anyhow, Result};
use hound::{SampleFormat, WavSpec, WavWriter};
use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};
use std::{f64::consts, fs::File, path::Path};
use symphonia::{
    core::{
        audio::SampleBuffer, codecs::DecoderOptions, errors::Error, formats::FormatOptions,
        io::MediaSourceStream, meta::MetadataOptions, probe::Hint,
    },
    default::{get_codecs, get_probe},
};

static SUPPORTED_FILETYPES: &'static [&'static str] = &[
    "wav", "aac", "adpcm", "aiff", "alac", "caf", "flac", "mkv", "mp1", "mp2", "mp3", "mp4", "m4a",
    "ogg", "oga", "webm",
];

fn is_supported_filetype<S: AsRef<str>>(ext: S) -> Result<()> {
    for t in SUPPORTED_FILETYPES {
        if *t == ext.as_ref() {
            return Ok(());
        }
    }
    Err(anyhow!("Unsupported filetype."))
}

fn sinc(x: f64) -> f64 {
    if x == 0. {
        1.
    } else {
        let x = consts::PI * x;
        x.sin() / x
    }
}

fn lanczos_window(x: f64, a: isize) -> f64 {
    sinc(x) * sinc(x / a as f64)
}

fn resample_audio(
    audio: Vec<f64>,
    in_fs: u32,
    out_fs: u32,
    lanczos_size: Option<isize>,
) -> Result<Vec<f64>> {
    let in_samples = audio.len();
    let out_samples = (in_samples as f64 * out_fs as f64 / in_fs as f64) as usize;
    let lanczos_size = lanczos_size.unwrap_or(2);
    let mut resampled: Vec<f64> = Vec::with_capacity(out_samples);

    for i in 0..out_samples {
        let findex = i as f64 * in_fs as f64 / out_fs as f64;
        let index = findex.floor() as isize;
        let mut sample = 0.;
        for j in -lanczos_size..=lanczos_size {
            let k = if index < lanczos_size {
                0
            } else if index + j >= out_samples as isize {
                out_samples as isize - 1
            } else {
                index + j
            } as usize;
            sample += audio[k] * lanczos_window(findex - k as f64, lanczos_size);
        }
        resampled.push(sample);
    }
    Ok(resampled)
}

pub fn read_audio<P: AsRef<Path>>(path: P, lanczos_size: Option<isize>) -> Result<Vec<f64>> {
    let ext = path.as_ref().extension().unwrap().to_str().unwrap();
    is_supported_filetype(ext)?;

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
    if fs == 44100 {
        Ok(audio)
    } else {
        resample_audio(audio, fs, 44100, lanczos_size)
    }
}

pub fn write_audio<P: AsRef<Path>>(path: P, audio: Vec<f64>) -> Result<()> {
    let out_spec = WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut writer = WavWriter::create(path.as_ref(), out_spec)?;
    for s in audio {
        writer.write_sample((s * i16::MAX as f64).clamp(i16::MIN as f64, i16::MAX as f64) as i16)?
    }
    writer.finalize()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{read_audio, write_audio};
    use std::env;
    use std::path::Path;

    #[test]
    fn test_read_write() {
        let test_paths = vec!["test/01.wav"];
        let test_paths: Vec<&Path> = test_paths.into_iter().map(|x| Path::new(x)).collect();
        for path in test_paths {
            let mut out_fname = path.file_stem().expect("Failed to get filename").to_owned();
            out_fname.push("_out");
            let out_path = path.with_file_name(out_fname).with_extension("wav");
            println!("{:?}", out_path.as_os_str());

            let audio = read_audio(path, None).expect("Failed to read file");
            write_audio(out_path, audio).expect("Failed to write audio");
        }
    }
}
