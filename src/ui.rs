use crate::palette::Palette;
use crate::state::State;
use crate::{HEIGHT, WIDTH};
use std::time::Duration;

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

pub fn draw_char(buffer: &mut [u32], x: usize, y: usize, ch: char, color: u32) {
    use font8x8::UnicodeFonts;
    if let Some(glyph) = font8x8::BASIC_FONTS.get(ch) {
        for (row, byte) in glyph.iter().enumerate() {
            if *byte == 0 { continue; }
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

pub fn draw_text(buffer: &mut [u32], x: usize, y: usize, text: &str, color: u32) {
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

pub fn draw_text_shadowed(buffer: &mut [u32], x: usize, y: usize, text: &str, fg: u32) {
    draw_text(buffer, x + 1, y + 1, text, 0x0000_0000);
    draw_text(buffer, x, y, text, fg);
}

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
    let box_w = (max_line * 8 + 12).min(WIDTH - 20);
    let box_h = lines * 10 + 8;
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

    draw_text_shadowed(buffer, 10, 10, &info, 0x00FF_FFFF);
}
