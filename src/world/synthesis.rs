use crate::consts;
use rsworld::synthesis;

pub fn synthesize(f0: &Vec<f64>, sp: &mut Vec<Vec<f64>>, ap: &mut Vec<Vec<f64>>) -> Vec<f64> {
    // Synthesize from WORLD features, ensuring features are within WORLD's restrictions
    for sp_frame in &mut *sp {
        for s in sp_frame {
            *s = s.max(1e-16);
        }
    }

    for ap_frame in &mut *ap {
        for a in ap_frame {
            *a = a.clamp(0., 1.);
        }
    }

    synthesis(
        &f0,
        &sp,
        &ap,
        consts::FRAME_PERIOD,
        consts::SAMPLE_RATE as i32,
    )
}

pub fn synthesize_harmonic(f0: &Vec<f64>, sp: &Vec<Vec<f64>>, ap: &Vec<Vec<f64>>) -> Vec<f64> {
    let mut sp_harmonic: Vec<Vec<f64>> = sp
        .iter()
        .zip(ap.iter())
        .map(|(sp_frame, ap_frame)| {
            sp_frame
                .iter()
                .zip(ap_frame.iter())
                .map(|(sp_v, ap_v)| sp_v * (1. - ap_v * ap_v))
                .collect()
        })
        .collect();
    let mut ap_harmonic: Vec<Vec<f64>> = ap
        .iter()
        .map(|frame| frame.iter().map(|_| 0.).collect())
        .collect();
    synthesize(f0, &mut sp_harmonic, &mut ap_harmonic)
}

pub fn synthesize_aperiodic(f0: &Vec<f64>, sp: &Vec<Vec<f64>>, ap: &Vec<Vec<f64>>) -> Vec<f64> {
    let mut sp_aperiodic: Vec<Vec<f64>> = sp
        .iter()
        .zip(ap.iter())
        .map(|(sp_frame, ap_frame)| {
            sp_frame
                .iter()
                .zip(ap_frame.iter())
                .map(|(sp_v, ap_v)| sp_v * ap_v * ap_v)
                .collect()
        })
        .collect();

    let mut ap_aperiodic: Vec<Vec<f64>> = ap
        .iter()
        .map(|frame| frame.iter().map(|_| 1.).collect())
        .collect();
    synthesize(f0, &mut sp_aperiodic, &mut ap_aperiodic)
}
