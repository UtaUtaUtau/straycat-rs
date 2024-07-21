use anyhow::{anyhow, Result};
use biquad::{Biquad, Coefficients, ToHertz, Type};

pub fn forward_backward_filter<F: Biquad<f64>>(
    signal: &mut Vec<f64>,
    filter: &mut F,
    repeats: usize,
) {
    // do forward-backward filtering on any biquad filter
    for _ in 0..repeats {
        signal.iter_mut().for_each(|x| *x = filter.run(*x)); // forward pass
        filter.reset_state(); // filter reset
        signal.iter_mut().rev().for_each(|x| *x = filter.run(*x)); // backward pass
        filter.reset_state(); // filter reset
    }
}

pub fn make_coefficients(
    filter: Type<f64>,
    fs: f64,
    f0: f64,
    q_value: f64,
) -> Result<Coefficients<f64>> {
    // make biquad coeffs with errors "handled"
    match Coefficients::<f64>::from_params(filter, fs.hz(), f0.hz(), q_value) {
        Ok(coeffs) => Ok(coeffs),
        Err(_) => Err(anyhow!("Can't make filter coefficients.")),
    }
}
