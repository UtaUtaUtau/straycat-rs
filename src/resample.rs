use std::path::Path;

use crate::audio::read_write::{read_audio, write_audio};
use crate::interpolator::interp::{self, Interpolator};
use crate::parser::ResamplerArgs;
use crate::util;
use crate::world::features::{generate_features, read_features, to_feature_path};
use crate::world::synthesis::synthesize;
use crate::{consts, pitchbend};
use anyhow::Result;

pub fn run(args: ResamplerArgs) -> Result<()> {
    let generate_only = &args.out_file == "nul";

    let in_file = Path::new(&args.in_file);
    let feature_path = to_feature_path(in_file);
    let features = if !feature_path.exists() || generate_only {
        println!("Generating features.");
        let audio = read_audio(&args.in_file)?;
        generate_features(&args.in_file, audio)?
    } else {
        println!("Reading features.");
        read_features(&feature_path)?
    };

    if generate_only {
        println!("Null output file. Skipping.");
        return Ok(());
    }

    let out_file = Path::new(&args.out_file);
    let velocity = (1. - args.velocity / 100.).exp2();
    let volume = args.volume / 100.;
    let modulation = args.modulation / 100.;

    println!("Decoding WORLD features.");

    let feature_length = features.f0.len();
    let sp = rsworld::decode_spectral_envelope(
        &features.mgc,
        feature_length as i32,
        consts::SAMPLE_RATE as i32,
        consts::FFT_SIZE,
    );
    let ap = rsworld::decode_aperiodicity(
        &features.bap,
        feature_length as i32,
        consts::SAMPLE_RATE as i32,
    );
    let f0_off: Vec<f64> = features
        .f0
        .iter()
        .map(|f0| {
            if *f0 == 0. {
                features.base_f0
            } else {
                f0 - features.base_f0
            }
        })
        .collect();

    println!("Calculating timing.");
    let fps = 1000. / consts::FRAME_PERIOD;
    let t_features: Vec<f64> = util::arange(feature_length as i32)
        .iter()
        .map(|x| x / fps)
        .collect();
    let feature_length_sec = feature_length as f64 / fps;
    let start = args.offset / 1000.;
    let end = args.cutoff / 1000.;
    let end = if end < 0. {
        start - end
    } else {
        feature_length_sec - end
    };
    let consonant = start + args.consonant / 1000.;

    println!("Preparing interpolation.");

    let t_consonant = util::linspace(
        start,
        consonant,
        (velocity * args.consonant / consts::FRAME_PERIOD) as usize,
        false,
    );

    let length_req = args.length / 1000.;
    let stretch_length = end - consonant;
    let t_stretch = if stretch_length > length_req {
        let con_idx = (consonant * fps) as usize;
        let len_idx = (length_req * fps) as usize;
        t_features[con_idx..con_idx + len_idx].to_vec()
    } else {
        crate::util::linspace(consonant, end, (length_req * fps) as usize, true)
    };
    let consonant_index = t_consonant.len();

    let t_render: Vec<f64> = t_consonant
        .into_iter()
        .chain(t_stretch.into_iter())
        .map(|x| x * fps)
        .collect();
    let render_length = t_render.len();

    println!("Interpolating WORLD features.");
    let f0_off_interp = interp::CatmullRom::new(f0_off);

    let f0_off_render = f0_off_interp.sample_with_vec(&t_render);
    let mut sp_render =
        interp::interpolate_first_axis(sp, &t_render, interp::InterpolatorType::Akima);
    let mut ap_render =
        interp::interpolate_first_axis(ap, &t_render, interp::InterpolatorType::Akima);
    let t_sec: Vec<f64> = util::arange(render_length as i32)
        .iter()
        .map(|x| x / fps)
        .collect();

    println!("Interpreting pitchbend.");
    let pitch = pitchbend::parser::pitch_string_to_cents(args.pitchbend, args.pitch as f64)?;
    let pps = 96. * args.tempo / 60.;
    let pitch_interp = interp::CatmullRom::new(pitch);
    let t_pitch: Vec<f64> = t_sec.iter().map(|x| x * pps).collect();
    let pitch_render = pitch_interp.sample_with_vec(&t_pitch);

    let mut f0_render = Vec::with_capacity(render_length);
    for i in 0..render_length {
        f0_render.push(util::midi_to_hz(pitch_render[i]) + f0_off_render[i] * modulation);
    }

    let syn: Vec<f64> = synthesize(&f0_render, &mut sp_render, &mut ap_render)
        .iter()
        .map(|x| x * volume)
        .collect();
    write_audio(out_file, &syn)?;
    Ok(())
}
