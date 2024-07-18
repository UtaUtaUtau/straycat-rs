use std::path::Path;

use crate::audio::read_write::{read_audio, write_audio};
use crate::flags::parser::Flags;
use crate::interpolator::interp::{self, Interpolator};
use crate::parser::ResamplerArgs;
use crate::util::{self, smoothstep};
use crate::world::features::{generate_features, read_features, to_feature_path};
use crate::world::synthesis::synthesize;
use crate::{consts, pitchbend};
use anyhow::Result;

pub fn run(args: ResamplerArgs) -> Result<()> {
    let null_out = &args.out_file == "nul"; // null file from Initialize freq. map args
    let flags: Flags = args.flags.replace("/", "").parse()?; // parse flags

    // input file and feature file
    let in_file = Path::new(&args.in_file);
    let feature_path = to_feature_path(in_file);

    // force generate feature file if enabled
    if let Some(threshold) = flags.generate_features {
        let threshold = threshold / 100.;
        println!(
            "Forcing feature generation with D4C threshold {}.",
            threshold
        );
        let audio = read_audio(&args.in_file)?;
        generate_features(&args.in_file, audio, Some(threshold))?;
    }

    // generate feature file if it doesn't exist
    let features = if !feature_path.exists() {
        println!("Generating features.");
        let audio = read_audio(&args.in_file)?;
        generate_features(&args.in_file, audio, None)?
    } else {
        println!("Reading features.");
        read_features(&feature_path)?
    };

    // skip null output
    if null_out {
        println!("Null output file. Skipping.");
        return Ok(());
    }

    let out_file = Path::new(&args.out_file); // output file
    let velocity = (1. - args.velocity / 100.).exp2(); // velocity as stretch
    let volume = args.volume / 100.; // volume
    let modulation = args.modulation / 100.; // mod

    println!("Decoding WORLD features.");

    let feature_length = features.f0.len();
    let feature_dim = consts::FFT_SIZE / 2 + 1;
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
    let fps = 1000. / consts::FRAME_PERIOD; // WORLD frames per second
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
        util::linspace(consonant, end, (length_req * fps) as usize, true)
    };
    let consonant = velocity * args.consonant / 1000.; // timestamp of consonant in the render

    let t_render: Vec<f64> = t_consonant
        .into_iter()
        .chain(t_stretch.into_iter())
        .map(|x| x * fps)
        .collect();
    let render_length = t_render.len();

    println!("Interpolating WORLD features.");
    let f0_off_interp = interp::Akima::new(f0_off);

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
    println!("Checking flags.");
    if flags.pitch_offset != 0. {
        println!("Applying pitch offset.");
    }
    let pitch = pitchbend::parser::pitch_string_to_midi(
        args.pitchbend,
        args.pitch as f64,
        flags.pitch_offset / 100.,
    )?;
    let pps = 8. * args.tempo / 5.; // pitchbend points per second
    let pitch_interp = interp::Akima::new(pitch);
    let t_pitch: Vec<f64> = t_sec.iter().map(|x| x * pps).collect();
    let pitch_render = pitch_interp.sample_with_vec(&t_pitch);

    let mut f0_render = Vec::with_capacity(render_length);
    for i in 0..render_length {
        f0_render.push(util::midi_to_hz(pitch_render[i]) + f0_off_render[i] * modulation);
    }

    if flags.fry_enable != 0. {
        println!("Applying fry.");
        let fry_length = flags.fry_enable / 1000.;
        let fry_transition = 0.5 * flags.fry_transition / 1000.;
        let fry_offset = flags.fry_offset / 1000.;
        let fry_volume = flags.fry_volume / 100.;

        for i in 0..render_length {
            let t = t_sec[i] - consonant - fry_offset;
            let amt = smoothstep(
                -fry_length - fry_transition,
                -fry_length + fry_transition,
                t,
            ) * smoothstep(fry_transition, -fry_transition, t);
            f0_render[i] = util::lerp(f0_render[i], flags.fry_pitch, amt);
            let sp_frame = &mut sp_render[i];
            let ap_frame = &ap_render[i];
            for j in 0..feature_dim as usize {
                sp_frame[j] *= util::lerp(1., 1. - ap_frame[j] * ap_frame[j], amt);
            }
            sp_render[i]
                .iter_mut()
                .chain(ap_render[i].iter_mut())
                .for_each(|x| *x *= util::lerp(1., fry_volume, amt));
        }
    }

    let syn: Vec<f64> = synthesize(&f0_render, &mut sp_render, &mut ap_render)
        .iter()
        .map(|x| x * volume)
        .collect();
    write_audio(out_file, &syn)?;
    Ok(())
}
