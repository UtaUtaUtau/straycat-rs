use std::path::Path;

use crate::audio::post_process::{peak_compression, peak_normalization};
use crate::audio::read_write::{read_audio, write_audio};
use crate::flags::parser::Flags;
use crate::interpolator::interp::{self, Interpolator};
use crate::parser::ResamplerArgs;
use crate::util::{self, smoothstep};
use crate::world::features::{generate_features, read_features, to_feature_path};
use crate::world::synthesis::{synthesize_aperiodic, synthesize_harmonic};
use crate::{consts, pitchbend};
use anyhow::Result;

fn fry(
    f0: &mut Vec<f64>,
    sp: &mut Vec<Vec<f64>>,
    vuv: &Vec<bool>,
    t: &Vec<f64>,
    consonant: f64,
    flags: &Flags,
) {
    // fake fry with a low pitchbend
    let fry_length = flags.fry_enable / 1000.;
    let fry_transition = 0.5 * flags.fry_transition.copysign(flags.fry_enable) / 1000.;
    let fry_offset = flags.fry_offset / 1000.;
    let fry_volume = flags.fry_volume / 100.;
    sp.iter_mut()
        .zip(f0.iter_mut())
        .zip(t.iter().zip(vuv.iter()))
        .for_each(|((sp_frame, f0), (t, vuv))| {
            let t = t - consonant - fry_offset;
            let amt = smoothstep(
                -fry_length - fry_transition,
                -fry_length + fry_transition,
                t,
            ) * smoothstep(fry_transition, -fry_transition, t);
            if *vuv {
                *f0 = util::lerp(*f0, flags.fry_pitch, amt);
            }
            sp_frame
                .iter_mut()
                .for_each(|x| *x *= util::lerp(1., fry_volume * fry_volume, amt));
        });
}

fn formant_shift(sp: &mut Vec<Vec<f64>>, ap: &mut Vec<Vec<f64>>, feature_dim: i32, shift: f64) {
    // shift formants by stretching in the frequency domain
    let freq_t: Vec<f64> = util::arange(feature_dim)
        .iter()
        .map(|x| x * shift)
        .collect();
    let mask: Vec<f64> = freq_t
        .iter()
        .map(|x| smoothstep((feature_dim - 1) as f64, (feature_dim - 2) as f64, *x))
        .collect();

    sp.iter_mut().for_each(|sp_frame| {
        let freq_interp = interp::Akima::new(sp_frame);
        *sp_frame = freq_interp.sample_with_vec(&freq_t);
        sp_frame
            .iter_mut()
            .zip(mask.iter())
            .for_each(|(s, m)| *s *= *m);
    });

    ap.iter_mut().for_each(|ap_frame| {
        let freq_interp = interp::Akima::new(ap_frame);
        *ap_frame = freq_interp.sample_with_vec(&freq_t);
        ap_frame
            .iter_mut()
            .zip(mask.iter())
            .for_each(|(a, m)| *a *= *m);
    });
}

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
    let feature_dim = (consts::FFT_SIZE / 2 + 1) as usize;
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
    let vuv: Vec<bool> = features.f0.iter().map(|f0| *f0 != 0.).collect();
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
        .into_iter()
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
    let f0_off_interp = interp::Akima::new(&f0_off);

    let f0_off_render = f0_off_interp.sample_with_vec(&t_render);
    let vuv_render: Vec<bool> = t_render
        .iter()
        .map(|i| vuv[(*i as usize).clamp(0, feature_length - 1)])
        .collect();
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
    let pitch_interp = interp::Akima::new(&pitch);
    let t_pitch: Vec<f64> = t_sec.iter().map(|x| x * pps).collect();
    let mut pitch_render = pitch_interp.sample_with_vec(&t_pitch);

    let mut f0_render: Vec<f64> = pitch_render
        .iter()
        .zip(f0_off_render.into_iter().zip(vuv_render.iter()))
        .map(|(pitch, (f0_off, vuv))| {
            if *vuv {
                util::midi_to_hz(*pitch) + f0_off * modulation
            } else {
                0.
            }
        })
        .collect();

    if flags.gender != 0. {
        println!("Shifting formants.");
        let shift = (flags.gender / 120.).exp2();
        formant_shift(&mut sp_render, &mut ap_render, feature_dim as i32, shift);
    }

    if flags.fry_enable != 0. {
        println!("Applying fry.");
        fry(
            &mut f0_render,
            &mut sp_render,
            &vuv,
            &t_sec,
            consonant,
            &flags,
        );
    }

    // render harmonic and aperiodic signals
    let syn_harmonic: Vec<f64> = synthesize_harmonic(&f0_render, &sp_render, &ap_render);
    let syn_aperiodic: Vec<f64> =
        synthesize_aperiodic(&f0_render, &mut sp_render, &ap_render, true);
    let t_syn: Vec<f64> = util::arange(syn_harmonic.len() as i32)
        .iter()
        .map(|x| x / consts::SAMPLE_RATE as f64)
        .collect();

    let harmonic_mix = 1. - 2. * (flags.breathiness / 100. - 0.5);
    if flags.breathiness != 50. {
        println!("Adjusting breathiness.");
    }

    // combined logic for all flags related to controlling voicing
    let mut syn: Vec<f64> = if flags.devoice_enable != 0. {
        let devoice_length = flags.devoice_enable / 1000.;
        let devoice_transition =
            0.5 * flags.devoice_transition.copysign(flags.devoice_enable) / 1000.;
        let devoice_offset = flags.devoice_offset / 1000.;
        syn_harmonic
            .iter()
            .zip(syn_aperiodic.iter())
            .zip(t_syn.iter())
            .map(|((hm, wh), t)| {
                let t = t - consonant - devoice_offset;
                let amt = smoothstep(
                    -devoice_length - devoice_transition,
                    -devoice_length + devoice_transition,
                    t,
                ) * smoothstep(devoice_transition, -devoice_transition, t);
                (hm * (1. - amt) * harmonic_mix + wh) * volume
            })
            .collect()
    } else {
        syn_harmonic
            .iter()
            .zip(syn_aperiodic.iter())
            .map(|(hm, wh)| (hm * harmonic_mix + wh) * volume)
            .collect()
    };

    if flags.peak_compression != 0. {
        println!("Compressing render.");
        peak_compression(&mut syn, flags.peak_compression / 100.)?;
    }

    if flags.peak_normalization >= 0. {
        println!("Normalizing render.");
        peak_normalization(&mut syn, flags.peak_normalization);
    }

    write_audio(out_file, &syn)?;
    Ok(())
}
