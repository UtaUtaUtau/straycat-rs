use crate::consts;
use rsworld::synthesis;

pub fn synthesize(f0: &Vec<f64>, sp: &mut Vec<Vec<f64>>, ap: &mut Vec<Vec<f64>>) -> Vec<f64> {
    for sp_frame in &mut *sp {
        for s in sp_frame {
            *s = s.max(1e-8);
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
