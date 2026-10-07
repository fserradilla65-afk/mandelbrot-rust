use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Scale, Window, WindowOptions};
use num_complex::Complex;
use rayon::prelude::*;
use std::time::{Duration, Instant};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

const WIDTH: usize = 1000;
const HEIGHT: usize = 1000;
const MAX_ITER_DEFAULT: u32 = 1000;
const MAX_ITER_STEP: u32 = 1000;
const MAX_ITER_MIN: u32 = 100;
const ZOOM_FACTOR: f64 = 2.0;   // cada click reduce a la mitad el ancho visible

// ============================================================
//  Modo de fractal
// ============================================================

#[derive(Clone, Copy, Debug)]
enum FractalKind {
    Mandelbrot,
    /// Conjunto de Julia para un parámetro c fijo.
    Julia { c: Complex<f64> },
}

impl FractalKind {
    fn name(self) -> String {
        match self {
            FractalKind::Mandelbrot => "Mandelbrot".to_string(),
            FractalKind::Julia { c } => format!("Julia (c = {:+.5} {:+.5}i)", c.re, c.im),
        }
    }
}

// ============================================================
//  Paletas de color (interpolación lineal sobre anclas)
// ============================================================

#[derive(Clone, Copy, Debug)]
enum Palette {
    Grayscale, Viridis, Inferno, Plasma, Turbo, Hot, Flag, Twilight,
}

impl Palette {
    fn next(self) -> Self {
        match self {
            Palette::Grayscale => Palette::Viridis,
            Palette::Viridis   => Palette::Inferno,
            Palette::Inferno   => Palette::Plasma,
            Palette::Plasma    => Palette::Turbo,
            Palette::Turbo     => Palette::Hot,
            Palette::Hot       => Palette::Flag,
            Palette::Flag      => Palette::Twilight,
            Palette::Twilight  => Palette::Grayscale,
        }
    }
    fn name(self) -> &'static str {
        match self {
            Palette::Grayscale => "Grayscale",
            Palette::Viridis   => "Viridis",
            Palette::Inferno   => "Inferno",
            Palette::Plasma    => "Plasma",
            Palette::Turbo     => "Turbo",
            Palette::Hot       => "Hot",
            Palette::Flag      => "Flag",
            Palette::Twilight  => "Twilight",
        }
    }
    fn color(self, t: f64) -> u32 {
        let t = t.clamp(0.0, 1.0);
        let (r, g, b): (u8, u8, u8) = match self {
            Palette::Grayscale => { let v = (t * 255.0) as u8; (v, v, v) }
            Palette::Viridis  => interpolate(&VIRIDIS,  t),
            Palette::Inferno  => interpolate(&INFERNO,  t),
            Palette::Plasma   => interpolate(&PLASMA,   t),
            Palette::Turbo    => interpolate(&TURBO,    t),
            Palette::Hot      => interpolate(&HOT,      t),
            Palette::Flag     => interpolate(&FLAG,     t),
            Palette::Twilight => interpolate(&TWILIGHT, t),
        };
        ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
    }
}

// Anclas aproximadas de las paletas de Matplotlib
const VIRIDIS: [(f64, (u8, u8, u8)); 5] = [
    (0.00, ( 68,   1,  84)),
    (0.25, ( 59,  82, 139)),
    (0.50, ( 33, 145, 140)),
    (0.75, ( 94, 201,  98)),
    (1.00, (253, 231,  37)),
];
const INFERNO: [(f64, (u8, u8, u8)); 5] = [
    (0.00, (  0,   0,   4)),
    (0.25, ( 87,  15, 109)),
    (0.50, (187,  55,  84)),
    (0.75, (249, 142,   8)),
    (1.00, (252, 255, 164)),
];
const PLASMA: [(f64, (u8, u8, u8)); 5] = [
    (0.00, ( 13,   8, 135)),
    (0.25, (126,   3, 168)),
    (0.50, (204,  71, 120)),
    (0.75, (248, 149,  64)),
    (1.00, (240, 249,  33)),
];
const TURBO: [(f64, (u8, u8, u8)); 5] = [
    (0.00, ( 48,  18,  59)),
    (0.25, ( 70, 134, 251)),
    (0.50, ( 27, 229, 181)),
    (0.75, (250, 145,  20)),
    (1.00, (122,   4,   3)),
];
const HOT: [(f64, (u8, u8, u8)); 4] = [
    (0.00, (  0,   0,   0)),
    (0.33, (255,   0,   0)),
    (0.66, (255, 255,   0)),
    (1.00, (255, 255, 255)),
];
const FLAG: [(f64, (u8, u8, u8)); 8] = [
    (0.000, (255,   0,   0)),
    (0.143, (255, 255, 255)),
    (0.286, (  0,   0, 255)),
    (0.429, (  0,   0,   0)),
    (0.571, (255, 255,   0)),
    (0.714, (255,   0, 255)),
    (0.857, (  0, 255, 255)),
    (1.000, (  0, 255,   0)),
];
const TWILIGHT: [(f64, (u8, u8, u8)); 7] = [
    (0.00, (226, 217, 226)),
    (0.20, ( 76,  88, 130)),
    (0.40, ( 45,  36,  66)),
    (0.55, (110,  20,  19)),
    (0.70, (208,  98,  70)),
    (0.85, (157, 186, 119)),
    (1.00, ( 10, 249, 194)),
];
fn interpolate(anchors: &[(f64, (u8, u8, u8))], t: f64) -> (u8, u8, u8) {
    if t <= anchors[0].0 { return anchors[0].1; }
    let last = anchors[anchors.len() - 1];
    if t >= last.0 { return last.1; }
    for w in anchors.windows(2) {
        let (t0, c0) = w[0];
        let (t1, c1) = w[1];
        if t >= t0 && t <= t1 {
            let f = (t - t0) / (t1 - t0);
            let r = (c0.0 as f64 + f * (c1.0 as f64 - c0.0 as f64)) as u8;
            let g = (c0.1 as f64 + f * (c1.1 as f64 - c0.1 as f64)) as u8;
            let b = (c0.2 as f64 + f * (c1.2 as f64 - c0.2 as f64)) as u8;
            return (r, g, b);
        }
    }
    last.1
}

// ============================================================
//  Viewport
// ============================================================

#[derive(Clone, Copy)]
struct Viewport {
    x_min: f64, x_max: f64,
    y_min: f64, y_max: f64,
}

impl Viewport {
    fn new_centered(cx: f64, cy: f64, half_width: f64) -> Self {
        let half_height = half_width * (HEIGHT as f64 / WIDTH as f64);
        Viewport {
            x_min: cx - half_width,  x_max: cx + half_width,
            y_min: cy - half_height, y_max: cy + half_height,
        }
    }
    /// Vista inicial del conjunto de Mandelbrot (span horizontal = 3).
    fn default_mandelbrot() -> Self {
        Viewport::new_centered(-0.5, 0.0, 1.5)
    }
    /// Vista inicial del conjunto de Julia (span horizontal = 4).
    fn default_julia() -> Self {
        Viewport::new_centered(0.0, 0.0, 2.0)
    }
    fn from_center(cx: f64, cy: f64, zoom: f64) -> Self {
        Viewport::new_centered(cx, cy, 1.5 / zoom)
    }
    fn center(&self) -> (f64, f64) {
        ((self.x_min + self.x_max) * 0.5, (self.y_min + self.y_max) * 0.5)
    }
    fn zoom(&self) -> f64 {
        let span = (self.x_max - self.x_min).min(self.y_max - self.y_min);
        3.0 / span
    }
}

// ============================================================
//  Render con Rayon + smooth coloring + paleta
// ============================================================

fn render_fractal(
    buffer: &mut [u32],
    viewport: &Viewport,
    max_iter: u32,
    palette: Palette,
    kind: FractalKind,
) {
    let scale_x = (viewport.x_max - viewport.x_min) / (WIDTH  as f64 - 1.0);
    let scale_y = (viewport.y_max - viewport.y_min) / (HEIGHT as f64 - 1.0);

    buffer
        .par_chunks_mut(WIDTH)
        .enumerate()
        .for_each(|(y, row)| {
            let cy = viewport.y_min + y as f64 * scale_y;
            for x in 0..WIDTH {
                let cx = viewport.x_min + x as f64 * scale_x;

                // Mandelbrot: z0 = 0, c = píxel
                // Julia:      z0 = píxel, c = parámetro
                let (mut z, c) = match kind {
                    FractalKind::Mandelbrot => (Complex::new(0.0, 0.0), Complex::new(cx, cy)),
                    FractalKind::Julia { c } => (Complex::new(cx, cy), c),
                };

                let mut iter: u32 = 0;
                while z.norm_sqr() <= 4.0 && iter < max_iter {
                    z = z * z + c;
                    iter += 1;
                }

                row[x] = if iter == max_iter {
                    0x00000000 // interior
                } else {
                    let modulus = z.norm();
                    let smooth = if modulus > 1.0 {
                        iter as f64 + 1.0
                            - (modulus.ln() / std::f64::consts::LN_2).ln()
                                / std::f64::consts::LN_2
                    } else {
                        iter as f64
                    };
                    let t = (smooth * 0.02).rem_euclid(1.0);
                    palette.color(t)
                };
            }
        });
}

/// Hace zoom sobre el punto de pantalla (mx, my).
fn zoom_at(viewport: &mut Viewport, mx: f64, my: f64, factor: f64) -> bool {
    if factor <= 1.0 { return false; }

    let scale_x = (viewport.x_max - viewport.x_min) / (WIDTH  as f64 - 1.0);
    let scale_y = (viewport.y_max - viewport.y_min) / (HEIGHT as f64 - 1.0);
    let cx = viewport.x_min + mx * scale_x;
    let cy = viewport.y_min + my * scale_y;

    let new_half_w = (viewport.x_max - viewport.x_min) * 0.5 / factor;
    let new_half_h = (viewport.y_max - viewport.y_min) * 0.5 / factor;

    viewport.x_min = cx - new_half_w;
    viewport.x_max = cx + new_half_w;
    viewport.y_min = cy - new_half_h;
    viewport.y_max = cy + new_half_h;

    true
}

// ============================================================
//  Texto con font8x8
// ============================================================

fn draw_char(buffer: &mut [u32], x: usize, y: usize, ch: char, color: u32) {
    use font8x8::UnicodeFonts;
    if let Some(glyph) = font8x8::BASIC_FONTS.get(ch) {
        for (row, byte) in glyph.iter().enumerate() {
            for col in 0..8 {
                if (byte >> col) & 1 == 1 {
                    let px = x + col;
                    let py = y + row;
                    if px < WIDTH && py < HEIGHT {
                        buffer[py * WIDTH + px] = color;
                    }
                }
            }
        }
    }
}

fn draw_text(buffer: &mut [u32], x: usize, y: usize, text: &str, color: u32) {
    let mut cx = x;
    let mut cy = y;
    for ch in text.chars() {
        if ch == '\n' {
            cx = x;
            cy += 10;
        } else {
            draw_char(buffer, cx, cy, ch, color);
            cx += 8;
        }
    }
}

fn draw_text_shadowed(buffer: &mut [u32], x: usize, y: usize, text: &str, fg: u32) {
    draw_text(buffer, x + 1, y + 1, text, 0x00000000);
    draw_text(buffer, x,     y,     text, fg);
}

// ============================================================
//  Formato de tiempo hh:mm:ss:cs
// ============================================================

fn format_time(d: Duration) -> String {
    let total_cs = d.as_millis() / 10;
    let cs = total_cs % 100;
    let total_s = total_cs / 100;
    let s = total_s % 60;
    let total_m = total_s / 60;
    let m = total_m % 60;
    let h = total_m / 60;
    format!("{:02}:{:02}:{:02}:{:02}", h, m, s, cs)
}

// ============================================================
//  Captura PNG
// ============================================================

fn save_png(buffer: &[u32]) -> Result<String, String> {
    let base = dirs::picture_dir()
        .or_else(dirs::home_dir)
        .ok_or_else(|| "No se pudo determinar la carpeta de imágenes del usuario".to_string())?;

    let dir = base.join("Capturas");
    fs::create_dir_all(&dir)
        .map_err(|e| format!("No se pudo crear '{}': {}", dir.display(), e))?;

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();

    let filename = dir.join(format!("mandelbrot_{}.png", ts));

    let mut bytes = Vec::with_capacity(WIDTH * HEIGHT * 3);
    for &p in buffer {
        bytes.push(((p >> 16) & 0xFF) as u8);
        bytes.push(((p >>  8) & 0xFF) as u8);
        bytes.push(( p        & 0xFF) as u8);
    }

    image::save_buffer(
        &filename, &bytes, WIDTH as u32, HEIGHT as u32,
        image::ColorType::Rgb8,
    )
    .map_err(|e| format!("No se pudo guardar '{}': {}", filename.display(), e))?;

    Ok(filename.to_string_lossy().into_owned())
}

// ============================================================
//  Argumentos de línea de comandos
// ============================================================
//
//  Uso:
//    mandelbrot-rust                        → Mandelbrot por defecto
//    mandelbrot-rust <cx> <cy> [zoom]       → Mandelbrot centrado
//    mandelbrot-rust julia <cr> <ci> [zoom] → Conjunto de Julia para c = cr + ci·i

fn parse_args() -> Result<(Viewport, FractalKind), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        return Ok((Viewport::default_mandelbrot(), FractalKind::Mandelbrot));
    }

    // Subcomando julia
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
        } else { 1.0 };
        if zoom <= 0.0 {
            return Err("El zoom debe ser mayor que 0".into());
        }
        let vp = Viewport::new_centered(0.0, 0.0, 2.0 / zoom);
        return Ok((vp, FractalKind::Julia { c: Complex::new(cr, ci) }));
    }

    // Mandelbrot
    match args.len() {
        2 | 3 => {
            let cx: f64 = args[0].parse()
                .map_err(|_| format!("'{}' no es un número válido para cx", args[0]))?;
            let cy: f64 = args[1].parse()
                .map_err(|_| format!("'{}' no es un número válido para cy", args[1]))?;
            let zoom: f64 = if args.len() == 3 {
                args[2].parse()
                    .map_err(|_| format!("'{}' no es un número válido para zoom", args[2]))?
            } else { 1.0 };
            if zoom <= 0.0 {
                return Err("El zoom debe ser mayor que 0".into());
            }
            Ok((Viewport::from_center(cx, cy, zoom), FractalKind::Mandelbrot))
        }
        _ => Err(
            "Uso: mandelbrot-rust [cx cy [zoom]] | mandelbrot-rust julia <cr> <ci> [zoom]".into()
        ),
    }
}

// ============================================================
//  main
// ============================================================

fn main() {
    let (initial_viewport, initial_kind) = match parse_args() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error en los argumentos: {}", e);
            eprintln!("Uso: mandelbrot-rust [cx cy [zoom]]");
            eprintln!("     mandelbrot-rust julia <cr> <ci> [zoom]");
            std::process::exit(1);
        }
    };

    let mut viewport = initial_viewport;
    let mut kind     = initial_kind;
    let mut max_iter = MAX_ITER_DEFAULT;
    let mut palette  = Palette::Viridis;

    let mut fractal_buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut display_buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "Mandelbrot / Julia Explorer (Rust) — Esc para salir",
        WIDTH,
        HEIGHT,
        WindowOptions {
            scale: Scale::FitScreen,
            resize: false,
            ..WindowOptions::default()
        },
    )
    .expect("No se pudo crear la ventana");

    window.set_target_fps(60);

    // Historial: guarda (viewport, modo) para que undo funcione
    // a través de cambios Mandelbrot ↔ Julia.
    let mut history: Vec<(Viewport, FractalKind)> = Vec::new();
    let mut left_was_down  = false;
    let mut right_was_down = false;

    // Último encuadre visto en modo Mandelbrot. Permite que M devuelva al
    // usuario exactamente donde estaba en lugar de resetear al encuadre inicial.
    // Si arrancamos directamente en Julia (vía CLI), no hay ninguno todavía.
    let mut last_mandelbrot: Option<Viewport> = match initial_kind {
        FractalKind::Mandelbrot => Some(initial_viewport),
        FractalKind::Julia { .. } => None,
    };

    // Render inicial
    let mut last_render_time = {
        let t0 = Instant::now();
        render_fractal(&mut fractal_buffer, &viewport, max_iter, palette, kind);
        t0.elapsed()
    };

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mut needs_render = false;

        // ---------- Ratón: zoom y deshacer ----------
        let left_down  = window.get_mouse_down(MouseButton::Left);
        let right_down = window.get_mouse_down(MouseButton::Right);

        if left_down && !left_was_down {
            if let Some((mx, my)) = window.get_mouse_pos(MouseMode::Clamp) {
                let before = (viewport, kind);
                if zoom_at(&mut viewport, mx as f64, my as f64, ZOOM_FACTOR) {
                    if history.len() >= 500 { history.remove(0); }
                    history.push(before);
                    needs_render = true;
                }
            }
        }
        if right_down && !right_was_down {
            if let Some((prev_vp, prev_kind)) = history.pop() {
                viewport = prev_vp;
                kind = prev_kind;
                needs_render = true;
            }
        }
        left_was_down  = left_down;
        right_was_down = right_down;

        // ---------- Teclado ----------
        if window.is_key_pressed(Key::I, KeyRepeat::No) {
            max_iter = max_iter.saturating_add(MAX_ITER_STEP);
            needs_render = true;
        }
        if window.is_key_pressed(Key::O, KeyRepeat::No) {
            max_iter = max_iter.saturating_sub(MAX_ITER_STEP).max(MAX_ITER_MIN);
            needs_render = true;
        }
        if window.is_key_pressed(Key::P, KeyRepeat::No) {
            palette = palette.next();
            needs_render = true;
        }
        if window.is_key_pressed(Key::R, KeyRepeat::No) {
            viewport = initial_viewport;
            kind = initial_kind;
            max_iter = MAX_ITER_DEFAULT;
            history.clear();
            last_mandelbrot = match initial_kind {         // ← NUEVO
                FractalKind::Mandelbrot => Some(initial_viewport),
                FractalKind::Julia { .. } => None,
            };
            needs_render = true;
        }
        if window.is_key_pressed(Key::U, KeyRepeat::No) {
            if let Some((prev_vp, prev_kind)) = history.pop() {
                viewport = prev_vp;
                kind = prev_kind;
                needs_render = true;
            }
        }

        // J: en Mandelbrot, el píxel bajo el cursor (o el centro)
        //    pasa a ser c y saltamos a la vista Julia.
        if window.is_key_pressed(Key::J, KeyRepeat::No)
            && matches!(kind, FractalKind::Mandelbrot)
        {
            let (px, py) = window
                .get_mouse_pos(MouseMode::Clamp)
                .map(|(x, y)| (x as f64, y as f64))
                .unwrap_or(((WIDTH / 2) as f64, (HEIGHT / 2) as f64));

            let scale_x = (viewport.x_max - viewport.x_min) / (WIDTH  as f64 - 1.0);
            let scale_y = (viewport.y_max - viewport.y_min) / (HEIGHT as f64 - 1.0);
            let cr = viewport.x_min + px * scale_x;
            let ci = viewport.y_min + py * scale_y;

            history.push((viewport, kind));
            last_mandelbrot = Some(viewport);
            kind = FractalKind::Julia { c: Complex::new(cr, ci) };
            viewport = Viewport::default_julia();
            needs_render = true;
        }

        // M: vuelve al Mandelbrot inicial desde cualquier modo
        if window.is_key_pressed(Key::M, KeyRepeat::No)
            && !matches!(kind, FractalKind::Mandelbrot)
        {
            history.push((viewport, kind));
            kind = FractalKind::Mandelbrot;
            viewport = last_mandelbrot.unwrap_or(initial_viewport);
            needs_render = true;
        }

        // ---------- Capturas ----------
        // Shift+S: con leyenda y barra de paleta
        if window.is_key_pressed(Key::S, KeyRepeat::No) && window.is_key_down(Key::LeftShift) {
            match save_png(&display_buffer) {
                Ok(path) => eprintln!("[OK] Captura con leyenda: {}", path),
                Err(e)   => eprintln!("[ERROR] {}", e),
            }
        }
        // S: limpia, sin leyenda
        if window.is_key_pressed(Key::S, KeyRepeat::No) && !window.is_key_down(Key::LeftShift) {
            match save_png(&fractal_buffer) {
                Ok(path) => eprintln!("[OK] Captura limpia: {}", path),
                Err(e)   => eprintln!("[ERROR] {}", e),
            }
        }

        // ---------- Render si hace falta ----------
        if needs_render {
            let t0 = Instant::now();
            render_fractal(&mut fractal_buffer, &viewport, max_iter, palette, kind);
            last_render_time = t0.elapsed();
        }

        // ---------- Composición del frame ----------
        display_buffer.copy_from_slice(&fractal_buffer);

        // Barra de la paleta actual
        let bar_x = 10usize;
        let bar_y = HEIGHT - 26;
        let bar_w = 260usize;
        let bar_h = 14usize;
        for i in 0..bar_w {
            let t = i as f64 / (bar_w - 1) as f64;
            let c = palette.color(t);
            for j in 0..bar_h {
                display_buffer[(bar_y + j) * WIDTH + bar_x + i] = c;
            }
        }

        // Leyenda textual
        let (cx, cy) = viewport.center();
        let info = format!(
            "Modo: {}   Centro: ({:+.8}, {:+.8})   Zoom: {:.3e}\n\
Iter: {}   Paleta: {}   Tiempo: {}   Historial: {}",
            kind.name(), cx, cy, viewport.zoom(),
            max_iter, palette.name(), format_time(last_render_time), history.len(),
        );

        // Fondo translúcido
        let lines = info.lines().count();
        let max_line = info.lines().map(|l| l.len()).max().unwrap_or(0);
        let box_w = (max_line * 8 + 12).min(WIDTH - 20);
        let box_h = lines * 10 + 8;
        for y in 0..box_h {
            for x in 0..box_w {
                let px = 6 + x;
                let py = 6 + y;
                if px < WIDTH && py < HEIGHT {
                    let idx = py * WIDTH + px;
                    let bg = display_buffer[idx];
                    let r = 3 * ((bg >> 16) & 0xFF) / 4;
                    let g = 3 * ((bg >>  8) & 0xFF) / 4;
                    let b = 3 * ( bg        & 0xFF) / 4;
                    display_buffer[idx] = (r << 16) | (g << 8) | b;
                }
            }
        }

        draw_text_shadowed(&mut display_buffer, 10, 10, &info, 0x00FF_FFFF);

        window
            .update_with_buffer(&display_buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}
