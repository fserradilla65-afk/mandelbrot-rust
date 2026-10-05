use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Window, WindowOptions};
use num_complex::Complex;
use rayon::prelude::*;
use std::time::{Duration, Instant};

const WIDTH: usize = 1000;
const HEIGHT: usize = 1000;
const MAX_ITER_DEFAULT: u32 = 1000;
const MAX_ITER_STEP: u32 = 1000;
const MAX_ITER_MIN: u32 = 100;
const ZOOM_FACTOR: f64 = 2.0;   // cada click reduce a la mitad el ancho visible

// ============================================================
//  Paletas de color (interpolación lineal sobre anclas)
// ============================================================

#[derive(Clone, Copy, Debug)]
enum Palette {
    Grayscale,
    Viridis,
    Inferno,
    Plasma,
    Turbo,
    Hot,
    Flag,
    Twilight,
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

    /// t ∈ [0,1] → 0x00RRGGBB
    fn color(self, t: f64) -> u32 {
        let t = t.clamp(0.0, 1.0);
        let (r, g, b): (u8, u8, u8) = match self {
            Palette::Grayscale => {
                let v = (t * 255.0) as u8;
                (v, v, v)
            }
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
// Flag: 8 colores cíclicos (rojo, blanco, azul, negro, amarillo, magenta, cian, verde)
const FLAG: [(f64, (u8, u8, u8)); 8] = [
    (0.000, (255,   0,   0)), // rojo
    (0.143, (255, 255, 255)), // blanco
    (0.286, (  0,   0, 255)), // azul
    (0.429, (  0,   0,   0)), // negro
    (0.571, (255, 255,   0)), // amarillo
    (0.714, (255,   0, 255)), // magenta
    (0.857, (  0, 255, 255)), // cian
    (1.000, (  0, 255,   0)), // verde
];
// Twilight: paleta divergente azul → púrpura → rojo → naranja → verde claro
const TWILIGHT: [(f64, (u8, u8, u8)); 7] = [
    (0.00, (226, 217, 226)), // gris claro rosado
    (0.20, ( 76,  88, 130)), // azul medio
    (0.40, ( 45,  36,  66)), // púrpura oscuro
    (0.55, (110,  20,  19)), // rojo profundo
    (0.70, (208,  98,  70)), // naranja
    (0.85, (157, 186, 119)), // verde oliva
    (1.00, ( 10, 249, 194)), // cian brillante
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
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
}

impl Viewport {
    fn initial() -> Self {
        Viewport { x_min: -2.0, x_max: 1.0, y_min: -1.5, y_max: 1.5 }
    }
    fn center(&self) -> (f64, f64) {
        ((self.x_min + self.x_max) * 0.5, (self.y_min + self.y_max) * 0.5)
    }
    fn zoom(&self) -> f64 {
        let span = (self.x_max - self.x_min).min(self.y_max - self.y_min);
        3.0 / span
    }
}
impl Viewport {
    /// Crea un viewport centrado en (cx, cy) con el zoom indicado (1.0 = vista completa).
    fn from_center(cx: f64, cy: f64, zoom: f64) -> Self {
        let half_w = 1.5 / zoom;
        let half_h = half_w * (HEIGHT as f64 / WIDTH as f64);
        Viewport {
            x_min: cx - half_w, x_max: cx + half_w,
            y_min: cy - half_h, y_max: cy + half_h,
        }
    }
}
// ============================================================
//  Render con Rayon + smooth coloring + paleta
// ============================================================

fn render_mandelbrot(
    buffer: &mut [u32],
    viewport: &Viewport,
    max_iter: u32,
    palette: Palette,
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

            let c = Complex::new(cx, cy);
            let mut z = Complex::new(0.0, 0.0);
            let mut iter: u32 = 0;

            while z.norm_sqr() <= 4.0 && iter < max_iter {
                z = z * z + c;
                iter += 1;
            }

            row[x] = if iter == max_iter {
                0x00000000 // interior
            } else {
                // Coloración suavizada: elimina las bandas
                let modulus = z.norm();
                let smooth = if modulus > 1.0 {
                    iter as f64 + 1.0
                    - (modulus.ln() / std::f64::consts::LN_2).ln()
                    / std::f64::consts::LN_2
                } else {
                    iter as f64
                };
                // Ciclamos por la paleta cada ~50 iteraciones
                let t = (smooth * 0.02).rem_euclid(1.0);
                palette.color(t)
            };
        }
    });
}

/// Hace zoom sobre el punto de pantalla (mx, my) reduciendo el ancho visible
/// por un factor `factor` (>1). Mantiene el punto bajo el cursor en el centro
/// de la nueva vista. Devuelve `true` si el viewport cambió.
fn zoom_at(viewport: &mut Viewport, mx: f64, my: f64, factor: f64) -> bool {
    if factor <= 1.0 { return false; }

    // Convertir el píxel del cursor a coordenadas del plano complejo
    let scale_x = (viewport.x_max - viewport.x_min) / (WIDTH  as f64 - 1.0);
    let scale_y = (viewport.y_max - viewport.y_min) / (HEIGHT as f64 - 1.0);
    let cx = viewport.x_min + mx * scale_x;
    let cy = viewport.y_min + my * scale_y;

    // Nuevo tamaño manteniendo la relación de aspecto
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

/// Sombra desplazada 1px para que el texto sea legible sobre cualquier fondo
fn draw_text_shadowed(buffer: &mut [u32], x: usize, y: usize, text: &str, fg: u32) {
    draw_text(buffer, x + 1, y + 1, text, 0x00000000);
    draw_text(buffer, x,     y,     text, fg);
}

// ============================================================
//  Formato de tiempo mm:ss:cc
// ============================================================

/// Formatea una Duration como hh:mm:ss:cs (horas:minutos:segundos:centésimas)
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
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

/// Guarda el buffer (formato 0x00RRGGBB) como PNG en ~/Imágenes/Capturas/mandelbrot_<ts>.png
fn save_png(buffer: &[u32]) -> Result<String, String> {
    let home = std::env::var("HOME")
    .map_err(|_| "No se pudo leer la variable de entorno HOME".to_string())?;
    let dir = format!("{}/Imágenes/Capturas", home);

    fs::create_dir_all(&dir).map_err(|e| format!("No se pudo crear '{}': {}", dir, e))?;

    let ts = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map_err(|e| e.to_string())?
    .as_secs();

    let filename = format!("{}/mandelbrot_{}.png", dir, ts);

    let mut bytes = Vec::with_capacity(WIDTH * HEIGHT * 3);
    for &p in buffer {
        bytes.push(((p >> 16) & 0xFF) as u8);
        bytes.push(((p >>  8) & 0xFF) as u8);
        bytes.push(( p        & 0xFF) as u8);
    }

    image::save_buffer(
        &filename,
        &bytes,
        WIDTH as u32,
        HEIGHT as u32,
        image::ColorType::Rgb8,
    )
    .map_err(|e| format!("No se pudo guardar '{}': {}", filename, e))?;

    Ok(filename)
}
/// Parsea argumentos de línea de comandos.
/// Formatos aceptados:
///   programa                          → vista por defecto
///   programa <cx> <cy>                → centra en (cx, cy) con zoom por defecto
///   programa <cx> <cy> <zoom>         → centra en (cx, cy) con el zoom indicado
fn parse_args() -> Result<(Option<(f64, f64, f64)>), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    match args.len() {
        0 => Ok(None),
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
                return Err("El zoom debe ser mayor que 0".into());
            }
            Ok(Some((cx, cy, zoom)))
        }
        _ => Err("Uso: mandelbrot-rust [cx cy [zoom]]".into()),
    }
}
// ============================================================
//  main
// ============================================================
fn main() {
    let mut viewport = match parse_args() {
        Ok(Some((cx, cy, zoom))) => Viewport::from_center(cx, cy, zoom),
        Ok(None) => Viewport::initial(),
        Err(e) => {
            eprintln!("Error en los argumentos: {}", e);
            eprintln!("Uso: mandelbrot-rust [cx cy [zoom]]");
            std::process::exit(1);
        }
    };
    let initial_viewport = viewport;
    let mut max_iter  = MAX_ITER_DEFAULT;
    let mut palette   = Palette::Viridis;

    let mut fractal_buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut display_buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    use minifb::{Window, WindowOptions, Scale};

    let mut window = Window::new(
        "Mandelbrot Explorer (Rust) — Esc para salir",
                                 WIDTH,
                                 HEIGHT,
                                 WindowOptions {
                                     scale: Scale::FitScreen,   // ← ajusta al tamaño de pantalla
                                     resize: false,
                                     ..WindowOptions::default()
                                 },
    )
    .expect("No se pudo crear la ventana");

    window.set_target_fps(60);

    let mut history: Vec<Viewport> = Vec::new();
    let mut left_was_down  = false;
    let mut right_was_down = false;

    // Render inicial
    let mut last_render_time = {
        let t0 = Instant::now();
        render_mandelbrot(&mut fractal_buffer, &viewport, max_iter, palette);
        t0.elapsed()
    };

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mut needs_render = false;

        // ---------- Ratón: zoom y deshacer con click ----------
        let left_down  = window.get_mouse_down(MouseButton::Left);
        let right_down = window.get_mouse_down(MouseButton::Right);

        // Click izquierdo: zoom centrado en el cursor
        if left_down && !left_was_down {
            if let Some((mx, my)) = window.get_mouse_pos(MouseMode::Clamp) {
                let before = viewport;
                if zoom_at(&mut viewport, mx as f64, my as f64, ZOOM_FACTOR) {
                    if history.len() >= 500 {
                        history.remove(0);
                    }
                    history.push(before);
                    needs_render = true;
                }
            }
        }
        // Click derecho: deshacer
        if right_down && !right_was_down {
            if let Some(prev) = history.pop() {
                viewport = prev;
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
            max_iter = MAX_ITER_DEFAULT;
            history.clear();
            needs_render = true;
        }
        if window.is_key_pressed(Key::U, KeyRepeat::No) {
            if let Some(prev) = history.pop() {
                viewport = prev;
                needs_render = true;
            }
        }
        // S: captura sin leyenda
        if window.is_key_pressed(Key::S, KeyRepeat::No) && window.is_key_down(Key::LeftShift) {
            match save_png(&display_buffer) {
                Ok(path) => eprintln!("[OK] Captura con leyenda: {}", path),
                Err(e)   => eprintln!("[ERROR] {}", e),
            }
        }
        // Shift+S: captura con leyenda
        if window.is_key_pressed(Key::S, KeyRepeat::No) && !window.is_key_down(Key::LeftShift) {
            match save_png(&fractal_buffer) {
                Ok(path) => eprintln!("[OK] Captura limpia: {}", path),
                Err(e)   => eprintln!("[ERROR] {}", e),
            }
        }
        // ---------- Render si hace falta ----------
        if needs_render {
            let t0 = Instant::now();
            render_mandelbrot(&mut fractal_buffer, &viewport, max_iter, palette);
            last_render_time = t0.elapsed();
        }

        // ---------- Composición de la imagen a mostrar ----------
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
            "Centro: ({:+.8}, {:+.8})   Zoom: {:.3e}   Iter: {}   Paleta: {}   Tiempo: {}\n\
Click izquierdo: zoom (2x)  Click derecho: deshacer zoom\n\
i/o: iteraciones +/-{}   p: paleta   u: undo ({})   r: reset   s: guardar",

cx, cy,
viewport.zoom(), max_iter, palette.name(),
                           format_time(last_render_time), MAX_ITER_STEP, history.len(),
        );

        // Fondo translúcido para el bloque de texto
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
