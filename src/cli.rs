use crate::fractal::FractalKind;
use crate::viewport::Viewport;
use num_complex::Complex;

pub struct CliArgs {
    pub viewport: Viewport,
    pub kind: FractalKind,
    pub exponent: u32, // <-- NUEVO
}

pub fn parse_args(aspect: f64) -> Result<CliArgs, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let default_exponent = 2;

    if args.is_empty() {
        return Ok(CliArgs {
            viewport: Viewport::default_mandelbrot(aspect),
            kind: FractalKind::Mandelbrot,
            exponent: default_exponent,
        });
    }

    if args[0].eq_ignore_ascii_case("julia") {
        if args.len() < 3 || args.len() > 5 { // <-- Límite ampliado a 5
            return Err("Uso: mandelbrot-rust julia <cr> <ci> [zoom] [n]".into());
        }
        let cr: f64 = args[1].parse()
            .map_err(|_| format!("'{}' no es un número válido para cr", args[1]))?;
        let ci: f64 = args[2].parse()
            .map_err(|_| format!("'{}' no es un número válido para ci", args[2]))?;
        let zoom: f64 = if args.len() >= 4 {
            args[3].parse()
                .map_err(|_| format!("'{}' no es un número válido para zoom", args[3]))?
        } else {
            1.0
        };
        let exponent: u32 = if args.len() == 5 {
            args[4].parse()
                .map_err(|_| format!("'{}' no es un número válido para n", args[4]))?
        } else {
            default_exponent
        };
        
        if zoom <= 0.0 {
            return Err("el zoom debe ser mayor que 0".into());
        }
        if exponent < 2 || exponent > 8 {
            return Err("el exponente n debe estar entre 2 y 8".into());
        }
        
        return Ok(CliArgs {
            viewport: Viewport::new_centered(0.0, 0.0, 2.0 / zoom, aspect),
            kind: FractalKind::Julia { c: Complex::new(cr, ci) },
            exponent,
        });
    }

    match args.len() {
        2 | 3 | 4 => { // <-- Límite ampliado a 4
            let cx: f64 = args[0].parse()
                .map_err(|_| format!("'{}' no es un número válido para cx", args[0]))?;
            let cy: f64 = args[1].parse()
                .map_err(|_| format!("'{}' no es un número válido para cy", args[1]))?;
            let zoom: f64 = if args.len() >= 3 {
                args[2].parse()
                    .map_err(|_| format!("'{}' no es un número válido para zoom", args[2]))?
            } else {
                1.0
            };
            let exponent: u32 = if args.len() == 4 {
                args[3].parse()
                    .map_err(|_| format!("'{}' no es un número válido para n", args[3]))?
            } else {
                default_exponent
            };

            if zoom <= 0.0 {
                return Err("el zoom debe ser mayor que 0".into());
            }
            if exponent < 2 || exponent > 8 {
                return Err("el exponente n debe estar entre 2 y 8".into());
            }

            Ok(CliArgs {
                viewport: Viewport::from_center_zoom(cx, cy, zoom, aspect),
                kind: FractalKind::Mandelbrot,
                exponent,
            })
        }
        _ => Err(
            "Uso: mandelbrot-rust [cx cy [zoom [n]]] | mandelbrot-rust julia <cr> <ci> [zoom [n]]"
                .into(),
        ),
    }
}