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
        let r = x.fract();
        let coeff = &self.coeffs[index];

        coeff.a + coeff.b * r + coeff.c * r * r + coeff.d * r * r * r
    }
}

pub struct CatmullRom {
    curve: Vec<f64>,
    coeffs: Vec<CubicCoefficients>,
}

impl CatmullRom {
    pub fn new(curve: Vec<f64>) -> Self {
        let n = curve.len() - 1;
        let mut coeffs = Vec::with_capacity(n);
        let mut p = Vec::with_capacity(n + 2);

        for x in &curve {
            p.push(*x);
        }

        p.insert(0, curve[0]);
        p.push(curve[curve.len() - 1]);

        for i in 0..n {
            let i = i + 1;
            coeffs.push(CubicCoefficients {
                a: -0.5 * p[i - 1] + 1.5 * p[i] - 1.5 * p[i + 1] + 0.5 * p[i + 2],
                b: p[i - 1] - 2.5 * p[i] + 2. * p[i + 1] - 0.5 * p[i + 2],
                c: -0.5 * p[i - 1] + 0.5 * p[i + 1],
                d: p[i],
            });
        }

        Self { curve, coeffs }
    }
}

impl Interpolator for CatmullRom {
    fn sample(&self, x: f64) -> f64 {
        let x = x.clamp(0., self.curve.len() as f64 - 1.);
        if x == self.curve.len() as f64 - 1. {
            return self.curve[self.curve.len() - 1];
        }
        if x == 0. {
            return self.curve[0];
        }
        let index = x.floor() as usize;
        let r = x.fract();
        let coeff = &self.coeffs[index];

        coeff.a * r * r * r + coeff.b * r * r + coeff.c * r + coeff.d
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
            let k = index + i;
            let s = if k < 0 {
                0.
            } else if k as usize > self.curve.len() - 1 {
                0.
            } else {
                self.curve[k as usize]
            };
            y += s * self.lanczos_window(x - k as f64);
        }
        y
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Write};

    use super::{Akima, CatmullRom, Interpolator, Lanczos};
    const X: [f64; 6] = [1., 2., 4., 2., 3., 2.]; // [0., 0., 0., 0., 0.5, 4., 5., 7.5];

    #[test]
    fn test_akima() {
        let n = X.len() as i32;
        let interp = Akima::new(X.to_vec());

        let mut csv = File::create("test/akima.csv").expect("Cannot create file");
        csv.write_all("x,y\n".as_bytes())
            .expect("Cannot write to file");
        for i in 0..n - 1 {
            for j in 0..256 {
                let i = i as f64 + j as f64 / 256.;
                let y = interp.sample(i);
                csv.write_all(format!("{},{}\n", i, y).as_bytes())
                    .expect("Cannot write to file");
            }
        }
    }

    #[test]
    fn test_catmull_rom() {
        let n = X.len() as i32;
        let interp = CatmullRom::new(X.to_vec());

        let mut csv = File::create("test/catmull_rom.csv").expect("Cannot create file");
        csv.write_all("x,y\n".as_bytes())
            .expect("Cannot write to file");
        for i in 0..n - 1 {
            for j in 0..256 {
                let i = i as f64 + j as f64 / 256.;
                let y = interp.sample(i);
                csv.write_all(format!("{},{}\n", i, y).as_bytes())
                    .expect("Cannot write to file");
            }
        }
    }

    #[test]
    fn test_lanczos() {
        let n = X.len() as i32;
        let interp = Lanczos::new(X.to_vec(), None);

        let mut csv = File::create("test/lanczos.csv").expect("Cannot create file");
        csv.write_all("x,y\n".as_bytes())
            .expect("Cannot write to file");
        for i in 0..n - 1 {
            for j in 0..256 {
                let i = i as f64 + j as f64 / 256.;
                let y = interp.sample(i);
                csv.write_all(format!("{},{}\n", i, y).as_bytes())
                    .expect("Cannot write to file");
            }
        }
    }
}
