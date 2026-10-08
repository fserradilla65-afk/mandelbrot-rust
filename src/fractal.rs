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

/// Devuelve `true` si (cx, cy) cae dentro del cardioide principal
/// o del bulbo de período 2. Solo válido para Mandelbrot.
#[inline(always)]
pub fn in_mandelbrot_interior(cx: f64, cy: f64) -> bool {
    let y2 = cy * cy;
    // Cardioide: q(q + (x − 1/4)) ≤ y²/4
    let xm = cx - 0.25;
    let q = xm * xm + y2;
    if q * (q + xm) <= 0.25 * y2 {
        return true;
    }
    // Bulbo período 2: (x + 1)² + y² ≤ 1/16
    let xp = cx + 1.0;
    if xp * xp + y2 <= 0.0625 {
        return true;
    }
    false
}

/// Itera `z ← z² + c` desde (zr0, zi0) hasta que |z|² > escape_sq o
/// se alcancen max_iter iteraciones. Devuelve (iter, zr, zi).
///
/// Si `check_period`, cada 20 iteraciones compara con el valor de 20
/// iteraciones atrás; si coinciden (a menos de ε) el punto está en un
/// ciclo atractor y se considera interior (retorna max_iter).
#[inline(always)]
pub fn iterate(
    mut zr: f64,
    mut zi: f64,
    cr: f64,
    ci: f64,
    max_iter: u32,
    escape_sq: f64,
    check_period: bool,
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
        zi = 2.0 * zr * zi + ci;
        zr = zr2 - zi2 + cr;
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
