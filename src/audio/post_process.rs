use crate::{
    consts, filter,
    interpolator::interp::{self, Interpolator},
};
use anyhow::Result;
use biquad::{DirectForm2Transposed, Type, Q_BUTTERWORTH_F64};

pub fn peak_compression(signal: &mut Vec<f64>, peak: f64) -> Result<()> {
    // peak compression
    if signal.len() < consts::FFT_SIZE as usize {
        println!("Render too short. Not compressing.");
        return Ok(());
    }
    // get rms size
    let hop_size = consts::SAMPLE_RATE as f64 * consts::FRAME_PERIOD / 1000.;
    let env_fs = 1000. / consts::FRAME_PERIOD; // sampling frequency of the envelope. i know, weird.
    let frame_size = consts::FFT_SIZE as f64;
    let hops = (1. + (signal.len() as f64 - frame_size) / hop_size) as usize;

    // calculate rms and max
    let mut comp = Vec::with_capacity(hops);
    let mut comp_max: f64 = -1.;
    for h in 0..hops {
        let i = (h as f64 * hop_size) as usize;
        let frame = &signal[i..i + frame_size as usize];
        let curr_rms = frame
            .iter()
            .enumerate()
            .fold(0., |acc, (i, x)| acc + (x * x - acc) / (i + 1) as f64) //recursive stable mean
            .sqrt();
        comp_max = comp_max.max(curr_rms);
        comp.push(curr_rms);
    }

    // turn rms to compression envelope
    let env_max = 1. / peak - 1.;
    comp.iter_mut().for_each(|x| {
        *x = *x / (peak * comp_max);
        *x = if *x >= 1. {
            1. - (1. - peak) * (*x - 1.) / env_max
        } else {
            1.
        }
    });

    // "blur" the compression envelope
    let blur_coeffs =
        filter::make_coefficients(Type::LowPass, env_fs, env_fs / 10., Q_BUTTERWORTH_F64)?;
    let mut blur_biquad = DirectForm2Transposed::<f64>::new(blur_coeffs);
    filter::forward_backward_filter(&mut comp, &mut blur_biquad, 1);

    // setup compression interpolator
    let comp_interp = interp::Akima::new(&comp);

    // compress signal
    signal
        .iter_mut()
        .enumerate()
        .for_each(|(i, x)| *x *= comp_interp.sample(i as f64 / hop_size));

    Ok(())
}

pub fn peak_normalization(signal: &mut Vec<f64>, db_norm: f64) {
    // normalize
    let norm = (-std::f64::consts::LN_10 * db_norm / 20.).exp();
    let peak: f64 = signal.iter().fold(-1., |acc, x| acc.max(x.abs()));
    signal.iter_mut().for_each(|x| *x = norm * *x / peak);
}
