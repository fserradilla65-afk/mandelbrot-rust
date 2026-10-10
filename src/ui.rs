use crate::palette::Palette;
use crate::state::State;
use crate::{HEIGHT, WIDTH};
use noto_sans_mono_bitmap::{get_raster, get_raster_width, FontWeight, RasterHeight};
use std::time::Duration;

// ============================================================
//  Utilidades de formato
// ============================================================

pub fn format_time(d: Duration) -> String {
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
//  Render de texto — implementación parametrizada (w, h)
//  Sirve tanto para la ventana principal como para la de ayuda.
// ============================================================

/// Dibuja un carácter con Noto Sans Mono (antialiasing real).
/// `w` y `h` son las dimensiones del buffer destino.
fn draw_char_in(
    buffer: &mut [u32],
    w: usize,
    h: usize,
    x: usize,
    y: usize,
    ch: char,
    fg: u32,
) {
    let raster = match get_raster(ch, FontWeight::Regular, RasterHeight::Size16) {
        Some(r) => r,
        None => return,
    };

    let fg_r = (fg >> 16) & 0xFF;
    let fg_g = (fg >> 8) & 0xFF;
    let fg_b = fg & 0xFF;

    for (row, row_data) in raster.raster().iter().enumerate() {
        for (col, &alpha) in row_data.iter().enumerate() {
            if alpha == 0 {
                continue;
            }
            let px = x + col;
            let py = y + row;
            if px >= w || py >= h {
                continue;
            }

            let idx = py * w + px;
            let bg = buffer[idx];
            let bg_r = (bg >> 16) & 0xFF;
            let bg_g = (bg >> 8) & 0xFF;
            let bg_b = bg & 0xFF;

            let a = alpha as u32;
            let inv = 255 - a;

            let r = (fg_r * a + bg_r * inv) / 255;
            let g = (fg_g * a + bg_g * inv) / 255;
            let b = (fg_b * a + bg_b * inv) / 255;

            buffer[idx] = (r << 16) | (g << 8) | b;
        }
    }
}

/// Texto multilínea (`\n` soportado). Avance horizontal según la
/// fuente (≈ 9-10 px con Size16), salto de línea 20 px.
fn draw_text_in(
    buffer: &mut [u32],
    w: usize,
    h: usize,
    x: usize,
    y: usize,
    text: &str,
    color: u32,
) {
    let char_w = get_raster_width(FontWeight::Regular, RasterHeight::Size16) as usize;
    let line_h = 20;

    let mut cx = x;
    let mut cy = y;
    for ch in text.chars() {
        if ch == '\n' {
            cx = x;
            cy += line_h;
        } else {
            draw_char_in(buffer, w, h, cx, cy, ch, color);
            cx += char_w;
        }
    }
}

/// Texto con sombra de 1 px (esquina inferior derecha, negro).
fn draw_text_shadowed_in(
    buffer: &mut [u32],
    w: usize,
    h: usize,
    x: usize,
    y: usize,
    text: &str,
    fg: u32,
) {
    draw_text_in(buffer, w, h, x + 1, y + 1, text, 0x0000_0000);
    draw_text_in(buffer, w, h, x, y, text, fg);
}

// ============================================================
//  Wrappers para la ventana principal (usan WIDTH/HEIGHT globales)
// ============================================================

pub fn draw_text_shadowed(buffer: &mut [u32], x: usize, y: usize, text: &str, fg: u32) {
    draw_text_shadowed_in(buffer, WIDTH, HEIGHT, x, y, text, fg);
}

// ============================================================
//  Barra de paleta
// ============================================================

pub fn draw_palette_bar(buffer: &mut [u32], palette: Palette) {
    const BAR_X: usize = 10;
    const BAR_W: usize = 260;
    const BAR_H: usize = 14;
    let bar_y = HEIGHT - 26;
    for i in 0..BAR_W {
        let t = i as f64 / (BAR_W - 1) as f64;
        let c = palette.color(t);
        for j in 0..BAR_H {
            buffer[(bar_y + j) * WIDTH + BAR_X + i] = c;
        }
    }
}

// ============================================================
//  Overlay de la ventana principal
// ============================================================

pub fn draw_overlay(
    buffer: &mut [u32],
    st: &State,
    mouse_screen: (f64, f64),
                    mouse_in: bool,
                    render_time: Duration,
                    recording_frames: Option<u32>,
) {
    let (cx, cy) = st.viewport.center();

    let mouse_line = if mouse_in {
        let (mx, my) = st.viewport.pixel_to_complex(mouse_screen.0, mouse_screen.1, WIDTH, HEIGHT);
        format!("Ratón: ({:+.10}, {:+.10})", mx, my)
    } else {
        "Ratón: fuera de la ventana".to_string()
    };

    let aa_str = if st.antialias { "ON " } else { "OFF" };
    let rec_str = match recording_frames {
        Some(n) => format!("   REC: {} frames", n),
        None => String::new(),
    };
    let preview_str = if render_time > Duration::from_millis(200) {
        "   (renderizando…)"
    } else {
        ""
    };

    let info = format!(
        "Modo: {}   Centro: ({:+.10}, {:+.10})   Zoom: {:.3e}\n\
Iter: {}   Paleta: {}   AA: {}   Tiempo: {}   Historial: {}{}\n\
{}{}",
st.kind.name(),
                       cx,
                       cy,
                       st.viewport.zoom(),
                       st.max_iter,
                       st.palette.name(),
                       aa_str,
                       format_time(render_time),
                           st.history.len(),
                       rec_str,
                       mouse_line,
                       preview_str,
    );

    // Caja translúcida de fondo
    let lines = info.lines().count();
    let max_line = info.lines().map(|l| l.chars().count()).max().unwrap_or(0);
    let char_w = get_raster_width(FontWeight::Regular, RasterHeight::Size16) as usize;
    let box_w = (max_line * char_w + 12).min(WIDTH - 20);
    let box_h = lines * 20 + 8; // 20 px por línea (Noto Sans Mono Size16)
    for y in 0..box_h {
        for x in 0..box_w {
            let px = 6 + x;
            let py = 6 + y;
            if px < WIDTH && py < HEIGHT {
                let idx = py * WIDTH + px;
                let bg = buffer[idx];
                let r = 3 * ((bg >> 16) & 0xFF) / 4;
                let g = 3 * ((bg >> 8) & 0xFF) / 4;
                let b = 3 * (bg & 0xFF) / 4;
                buffer[idx] = (r << 16) | (g << 8) | b;
            }
        }
    }

    // Texto en gris claro para reducir el "abigarrado"
    draw_text_shadowed(buffer, 10, 10, &info, 0x00E8_E8E8);
}

// ============================================================
//  Ventana de ayuda (F1)
// ============================================================

/// Dibuja el contenido de la ventana de ayuda sobre un buffer de
/// dimensiones `w × h` (típicamente HELP_WIDTH × HELP_HEIGHT).
pub fn draw_help_window(buffer: &mut [u32], w: usize, h: usize) {
    // Fondo oscuro uniforme
    for px in buffer.iter_mut() {
        *px = 0x001A_1A1A;
    }

    // Título
    draw_text_shadowed_in(
        buffer,
        w,
        h,
        20,
        16,
        "CONTROLES — Mandelbrot / Julia Explorer",
        0x00FF_FFFF,
    );

    // Separador horizontal
    let sep_y = 44;
    for x in 20..w.saturating_sub(20) {
        let idx = sep_y * w + x;
        if idx < buffer.len() {
            buffer[idx] = 0x0055_5555;
        }
    }

    // Listado de controles
    let lineas = [
        "  Rueda del ratón     Zoom continuo (centrado en cursor)",
        "  Click izquierdo      Zoom ×2 sobre el punto",
        "  Arrastrar            Pan (mover la vista)",
        "  Click derecho / U    Deshacer (undo)",
        "  J                    Julia desde el punto bajo el cursor",
        "  M                    Volver a Mandelbrot",
        "  I / O                Aumentar / reducir iteraciones (±1000)",
        "  P                    Cambiar paleta de color",
        "  A                    Antialiasing ON / OFF",
        "  Z                    Auto-zoom ON / OFF",
        "  V                    Grabar / detener secuencia de frames",
        "  S                    Captura PNG (sin leyenda)",
        "  Shift + S            Captura PNG (con leyenda)",
        "  F5 / F9              Guardar / cargar estado de la sesión",
        "  R                    Resetear al estado inicial",
        "  F1                   Abrir / cerrar esta ventana",
        "  Esc                  Salir del programa",
    ];

    let mut y = 62;
    for linea in &lineas {
        draw_text_shadowed_in(buffer, w, h, 20, y, linea, 0x00DD_DDDD);
        y += 22;
    }
}
