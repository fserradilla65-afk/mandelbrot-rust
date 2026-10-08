use crate::fractal::FractalKind;
use crate::viewport::Viewport;
use num_complex::Complex;

pub struct CliArgs {
    pub viewport: Viewport,
    pub kind: FractalKind,
}

pub fn parse_args(aspect: f64) -> Result<CliArgs, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        return Ok(CliArgs {
            viewport: Viewport::default_mandelbrot(aspect),
            kind: FractalKind::Mandelbrot,
        });
    }

    if args[0].eq_ignore_ascii_case("julia") {
        if args.len() < 3 || args.len() > 4 {
            return Err("Uso: mandelbrot-rust julia <cr> <ci> [zoom]".into());
        }
        let cr: f64 = args[1].parse()
            .map_err(|_| format!("'{}' no es un número válido para cr", args[1]))?;
        let ci: f64 = args[2].parse()
            .map_err(|_| format!("'{}' no es un número válido para ci", args[2]))?;
        let zoom: f64 = if args.len() == 4 {
            args[3].parse()
                .map_err(|_| format!("'{}' no es un número válido para zoom", args[3]))?
        } else {
            1.0
        };
        if zoom <= 0.0 {
            return Err("el zoom debe ser mayor que 0".into());
        }
        return Ok(CliArgs {
            viewport: Viewport::new_centered(0.0, 0.0, 2.0 / zoom, aspect),
            kind: FractalKind::Julia { c: Complex::new(cr, ci) },
        });
    }

    match args.len() {
        2 | 3 => {
            let cx: f64 = args[0].parse()
                .map_err(|_| format!("'{}' no es un número válido para cx", args[0]))?;
            let cy: f64 = args[1].parse()
                .map_err(|_| format!("'{}' no es un número válido para cy", args[1]))?;
            let zoom: f64 = if args.len() == 3 {
                args[2].parse()
                    .map_err(|_| format!("'{}' no es un número válido para zoom", args[2]))?
            } else {
                1.0
            };
            if zoom <= 0.0 {
                return Err("el zoom debe ser mayor que 0".into());
            }
            Ok(CliArgs {
                viewport: Viewport::from_center_zoom(cx, cy, zoom, aspect),
                kind: FractalKind::Mandelbrot,
            })
        }
        _ => Err(
            "Uso: mandelbrot-rust [cx cy [zoom]] | mandelbrot-rust julia <cr> <ci> [zoom]"
                .into(),
        ),
    }
}
