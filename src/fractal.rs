use num_complex::Complex;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum FractalKind {
    Mandelbrot,
    Julia { c: Complex<f64> },
}

impl FractalKind {
    pub fn name(self) -> String {
        match self {
            FractalKind::Mandelbrot => "Mandelbrot".into(),
            FractalKind::Julia { c } => format!("Julia (c = {:+.5} {:+.5}i)", c.re, c.im),
        }
    }
}

#[inline(always)]
pub fn in_mandelbrot_interior(cx: f64, cy: f64) -> bool {
    let y2 = cy * cy;
    let xm = cx - 0.25;
    let q = xm * xm + y2;
    if q * (q + xm) <= 0.25 * y2 {
        return true;
    }
    let xp = cx + 1.0;
    if xp * xp + y2 <= 0.0625 {
        return true;
    }
    false
}

#[inline(always)]
pub fn iterate(
    mut zr: f64,
    mut zi: f64,
    cr: f64,
    ci: f64,
    max_iter: u32,
    escape_sq: f64,
    check_period: bool,
    exponent: u32,
) -> (u32, f64, f64) {
    let mut zr2 = zr * zr;
    let mut zi2 = zi * zi;
    let mut iter: u32 = 0;

    let mut old_zr = 0.0f64;
    let mut old_zi = 0.0f64;
    let mut next_check: u32 = 20;
    let mut have_old = false;

    const EPS2: f64 = 1e-16;

    while zr2 + zi2 <= escape_sq && iter < max_iter {
        // Bifurcación: n == 2 usa el camino rápido original.
        // n > 2 usa el bucle genérico de potenciación.
        let (nzr, nzi) = if exponent == 2 {
            (zr2 - zi2, 2.0 * zr * zi)
        } else {
            // z^n por multiplicación repetida
            let (mut ar, mut ai) = (zr, zi);
            for _ in 1..exponent {
                let tr = ar * zr - ai * zi;
                ai = ar * zi + ai * zr;
                ar = tr;
            }
            (ar, ai)
        };

        zr = nzr + cr;
        zi = nzi + ci;
        zr2 = zr * zr;
        zi2 = zi * zi;
        iter += 1;

        if check_period {
            if have_old && iter == next_check {
                let dr = zr - old_zr;
                let di = zi - old_zi;
                if dr * dr + di * di < EPS2 {
                    return (max_iter, zr, zi);
                }
                old_zr = zr;
                old_zi = zi;
                next_check = next_check.saturating_add(20);
            } else if !have_old {
                old_zr = zr;
                old_zi = zi;
                have_old = true;
            }
        }
    }

    (iter, zr, zi)
}