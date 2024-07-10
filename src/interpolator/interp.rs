use crate::util;
pub trait Interpolator {
    fn sample(&self, x: f64) -> f64;

    fn sample_with_vec(&self, x: &Vec<f64>) -> Vec<f64> {
        x.into_iter().map(|p| self.sample(*p)).collect()
    }
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

pub enum InterpolatorType {
    Akima,
    CatmullRom,
    Lanczos(Option<f64>),
}

pub fn interpolate_first_axis(
    vec_2d: Vec<Vec<f64>>,
    points: &Vec<f64>,
    interpolator_type: InterpolatorType,
) -> Vec<Vec<f64>> {
    let mut interpolated = Vec::with_capacity(points.len());
    let axis0_len = vec_2d.len();
    let axis1_len = vec_2d[0].len();
    for j in 0..axis1_len {
        let mut axis0_vec = Vec::with_capacity(axis0_len);
        for i in 0..axis0_len {
            axis0_vec.push(vec_2d[i][j]);
        }

        let axis0_interpolator: Box<dyn Interpolator> = match interpolator_type {
            InterpolatorType::Akima => Box::new(Akima::new(axis0_vec)),
            InterpolatorType::CatmullRom => Box::new(CatmullRom::new(axis0_vec)),
            InterpolatorType::Lanczos(q) => Box::new(Lanczos::new(axis0_vec, q)),
        };

        interpolated.push(axis0_interpolator.sample_with_vec(points))
    }
    util::transpose(interpolated)
}

pub fn interpolate_second_axis(
    vec_2d: Vec<Vec<f64>>,
    points: &Vec<f64>,
    interpolator_type: InterpolatorType,
) -> Vec<Vec<f64>> {
    let mut interpolated = Vec::with_capacity(vec_2d.len());
    for axis1_vec in vec_2d {
        let axis1_interpolator: Box<dyn Interpolator> = match interpolator_type {
            InterpolatorType::Akima => Box::new(Akima::new(axis1_vec)),
            InterpolatorType::CatmullRom => Box::new(CatmullRom::new(axis1_vec)),
            InterpolatorType::Lanczos(q) => Box::new(Lanczos::new(axis1_vec, q)),
        };
        interpolated.push(axis1_interpolator.sample_with_vec(points))
    }
    interpolated
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Write};

    use crate::{
        interpolator::interp,
        util::{self, transpose},
    };

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

    #[test]
    fn test_2d_interp() {
        let mut test_2d = Vec::with_capacity(16);
        for i in 0..16 {
            test_2d.push(vec![
                i as f64,
                (i + 1) as f64,
                (i + 2) as f64,
                (i + 3) as f64,
            ]);
        }
        println!("test shape: ({}, {})", test_2d.len(), test_2d[0].len());

        let mut interpolated = Vec::with_capacity(32);
        for j in 0..4 {
            let mut axis_vec = Vec::with_capacity(16);
            for i in 0..15 {
                axis_vec.push(test_2d[i][j]);
            }

            let axis_interp = CatmullRom::new(axis_vec);
            let mut axis_interpolated = Vec::with_capacity(32);
            for i in 0..32 {
                axis_interpolated.push(axis_interp.sample(i as f64 / 2.));
            }
            interpolated.push(axis_interpolated);
        }
        let interpolated = transpose(interpolated);
        println!(
            "interpolated shape: ({}, {})",
            interpolated.len(),
            interpolated[0].len()
        );

        for i in 0..16 {
            let line: Vec<String> = test_2d[i].iter().map(|x| format!("{}", x)).collect();
            let line = line.join(", ");
            println!("{}", line);
        }

        for i in 0..32 {
            let line: Vec<String> = interpolated[i].iter().map(|x| format!("{}", x)).collect();
            let line = line.join(", ");
            println!("{}", line);
        }
    }
}
