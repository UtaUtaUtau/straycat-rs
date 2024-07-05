use std::f64::consts;

pub fn lanczos_window(x: f64, a: isize) -> f64 {
    if x == 0. {
        1.
    } else if x.abs() > a as f64 {
        0.
    } else {
        let x = consts::PI * x;
        let a = a as f64;
        a * x.sin() * (x / a).sin() / (x * x)
    }
}
