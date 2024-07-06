pub trait Interpolator {
    fn sample(&self, x: f64) -> f64;
}

struct CubicCoefficients {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
}

pub struct Akima {
    curve: Vec<f64>,
    coeffs: Vec<CubicCoefficients>,
}

impl Akima {
    pub fn new(curve: Vec<f64>) -> Self {
        let n = curve.len() - 1;
        let mut coeffs: Vec<CubicCoefficients> = Vec::with_capacity(n);
        let mut m: Vec<f64> = Vec::with_capacity(n + 4);
        let mut s: Vec<f64> = Vec::with_capacity(n + 1);

        for i in 0..n {
            m.push(curve[i + 1] - curve[i]);
        }

        m.insert(0, 0.);
        m.insert(0, 0.);
        m.push(0.);
        m.push(0.);

        for i in 0..n + 1 {
            let i = i + 2;
            let weight_1 = (m[i + 1] - m[i]).abs();
            let weight_2 = (m[i - 1] - m[i - 2]).abs();
            let tally = weight_1 + weight_2;
            s.push(if tally == 0. {
                (m[i - 1] + m[i]) / 2.
            } else {
                (weight_1 * m[i - 1] + weight_2 * m[i]) / tally
            });
        }

        for i in 0..n {
            coeffs.push(CubicCoefficients {
                a: curve[i],
                b: s[i],
                c: 3. * m[i + 2] - 2. * s[i] - s[i + 1],
                d: s[i] + s[i + 1] - 2. * m[i + 2],
            })
        }

        Self { curve, coeffs }
    }
}

impl Interpolator for Akima {
    fn sample(&self, x: f64) -> f64 {
        let x = x.clamp(0., self.curve.len() as f64 - 1.);
        if x == self.curve.len() as f64 - 1. {
            return self.curve[self.curve.len() - 1];
        }
        if x == 0. {
            return self.curve[0];
        }
        let index = x.floor() as usize;
        let ratio = x.fract();
        let coeff = &self.coeffs[index];

        coeff.a + coeff.b * ratio + coeff.c * ratio * ratio + coeff.d * ratio * ratio * ratio
    }
}

pub struct Lanczos {
    curve: Vec<f64>,
    q: f64,
}

impl Lanczos {
    pub fn new(curve: Vec<f64>, q: Option<f64>) -> Self {
        Self {
            curve,
            q: q.unwrap_or(3.),
        }
    }

    fn lanczos_window(&self, x: f64) -> f64 {
        let q = self.q;
        if x == 0. {
            1.
        } else if x.abs() > q {
            0.
        } else {
            let x = std::f64::consts::PI * x;
            q * x.sin() * (x / q).sin() / (x * x)
        }
    }
}

impl Interpolator for Lanczos {
    fn sample(&self, x: f64) -> f64 {
        let x = x.clamp(0., self.curve.len() as f64 - 1.);
        let index = x.floor() as isize;
        let mut y = 0.;
        let a = self.q as isize;
        for i in -a..=a {
            let k = (index + i).clamp(0, self.curve.len() as isize - 1) as usize;
            y += self.curve[k] * self.lanczos_window(x - k as f64);
        }
        y
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Write};

    use super::{Akima, Interpolator};

    #[test]
    fn test_akima() {
        let x = vec![0., 0., 0., 0., 0.5, 4., 5., 7.5];
        let n = x.len() as i32;
        let interp = Akima::new(x);

        let mut csv = File::create("test/akima.csv").expect("Cannot create file");
        csv.write_all("x,y\n".as_bytes())
            .expect("Cannot write to file");
        for i in -1..=n + 1 {
            for j in 0..8 {
                let i = i as f64 + j as f64 / 8.;
                let y = interp.sample(i);
                csv.write_all(format!("{},{}\n", i, y).as_bytes())
                    .expect("Cannot write to file");
            }
        }
    }
}
