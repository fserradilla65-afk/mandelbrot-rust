mod cli;
mod fractal;
mod palette;
mod png;
mod render;
mod state;
mod ui;
mod viewport;

use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Scale, Window, WindowOptions};
use std::time::Instant;

pub const WIDTH: usize = 1000;
pub const HEIGHT: usize = 1000;
pub const ASPECT: f64 = HEIGHT as f64 / WIDTH as f64;

pub const MAX_ITER_DEFAULT: u32 = 1000;
const MAX_ITER_STEP: u32 = 1000;
const MAX_ITER_MIN: u32 = 100;
const ZOOM_FACTOR: f64 = 2.0;
const SCROLL_ZOOM_BASE: f64 = 1.15;
const DRAG_THRESHOLD_SQ: f64 = 9.0;
const HELP_WIDTH: usize = 620;
const HELP_HEIGHT: usize = 520;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut help_window: Option<Window> = None;
    let mut help_buffer = vec![0u32; HELP_WIDTH * HELP_HEIGHT];
    let args = cli::parse_args(ASPECT)?;
    let mut st = state::State::new(args);

    let mut window = Window::new(
        "Mandelbrot / Julia Explorer (Rust) — F1: Controles — Esc: Salir",
                                 WIDTH,
                                 HEIGHT,
                                 WindowOptions {
                                     scale: Scale::FitScreen,
                                     resize: false,
                                     ..WindowOptions::default()
                                 },
    )
    .map_err(|e| format!("no se pudo crear la ventana: {}", e))?;

    window.set_target_fps(60);

    let mut renderer = render::Renderer::new();
    let mut display = vec![0u32; WIDTH * HEIGHT];

    // Estado del ratón
    let mut mouse_screen = (WIDTH as f64 * 0.5, HEIGHT as f64 * 0.5);
    let mut left_was_down = false;
    let mut right_was_down = false;
    let mut drag_origin: Option<(f64, f64)> = None;
    let mut dragging = false;
    let mut last_mouse = mouse_screen;

    // Grabadora
    let mut recorder: Option<png::FrameRecorder> = None;

    // Primer render
    renderer.request(st.to_render_params(false));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mut changed = false;

        // ---------- Posición del ratón ----------
        let inside = window.get_mouse_pos(MouseMode::Discard);
        let mouse_in = inside.is_some();
        if let Some((mx, my)) = inside {
            mouse_screen = (mx as f64, my as f64);
        }

        // ---------- Rueda del ratón ----------
        if let Some((_, dy)) = window.get_scroll_wheel() {
            if dy != 0.0 && mouse_in {
                st.push_history();
                let factor = SCROLL_ZOOM_BASE.powf(dy as f64);  // <-- cambio aquí
                st.viewport.zoom_at(mouse_screen.0, mouse_screen.1, factor, WIDTH, HEIGHT);
                changed = true;
            }
        }

        // ---------- Botones del ratón ----------
        let left = window.get_mouse_down(MouseButton::Left);
        let right = window.get_mouse_down(MouseButton::Right);

        if left && !left_was_down {
            drag_origin = Some(mouse_screen);
            dragging = false;
        }
        if left {
            if let Some(origin) = drag_origin {
                let dx = mouse_screen.0 - origin.0;
                let dy = mouse_screen.1 - origin.1;
                if !dragging && dx * dx + dy * dy > DRAG_THRESHOLD_SQ {
                    dragging = true;
                    st.push_history();
                }
                if dragging {
                    let ddx = mouse_screen.0 - last_mouse.0;
                    let ddy = mouse_screen.1 - last_mouse.1;
                    if ddx != 0.0 || ddy != 0.0 {
                        // Arrastrar la imagen a la derecha mueve el viewport a la izquierda
                        st.viewport.pan_pixels(-ddx, -ddy, WIDTH, HEIGHT);
                        changed = true;
                    }
                }
            }
        } else if left_was_down {
            if !dragging {
                if let Some(origin) = drag_origin {
                    st.push_history();
                    st.viewport.zoom_at(origin.0, origin.1, ZOOM_FACTOR, WIDTH, HEIGHT);
                    changed = true;
                }
            }
            drag_origin = None;
            dragging = false;
        }
        left_was_down = left;

        if right && !right_was_down {
            if st.undo() {
                changed = true;
            }
        }
        right_was_down = right;

        // ---------- Teclado ----------
        if window.is_key_pressed(Key::I, KeyRepeat::No) {
            st.max_iter = st.max_iter.saturating_add(MAX_ITER_STEP);
            changed = true;
        }
        if window.is_key_pressed(Key::O, KeyRepeat::No) {
            st.max_iter = st.max_iter.saturating_sub(MAX_ITER_STEP).max(MAX_ITER_MIN);
            changed = true;
        }
        if window.is_key_pressed(Key::P, KeyRepeat::No) {
            st.palette = st.palette.next();
            changed = true;
        }
        if window.is_key_pressed(Key::A, KeyRepeat::No) {
            st.antialias = !st.antialias;
            changed = true;
        }
        if window.is_key_pressed(Key::U, KeyRepeat::No) && st.undo() {
            changed = true;
        }
        if window.is_key_pressed(Key::R, KeyRepeat::No) {
            st.reset();
            changed = true;
        }
        if window.is_key_pressed(Key::J, KeyRepeat::No) {
            let (px, py) = if mouse_in {
                mouse_screen
            } else {
                (WIDTH as f64 * 0.5, HEIGHT as f64 * 0.5)
            };
            if st.jump_to_julia(px, py, WIDTH, HEIGHT) {
                changed = true;
            }
        }
        if window.is_key_pressed(Key::M, KeyRepeat::No) && st.back_to_mandelbrot() {
            changed = true;
        }
        if window.is_key_pressed(Key::Z, KeyRepeat::No) {
            st.auto_zoom = !st.auto_zoom;
        }

        // Guardar capturas
        if window.is_key_pressed(Key::S, KeyRepeat::No) {
            let shift =
            window.is_key_down(Key::LeftShift) || window.is_key_down(Key::RightShift);
            let (buf, tag) = if shift {
                (&display[..], "legend")
            } else {
                (renderer.frame(), "clean")
            };
            match png::save_png(buf, WIDTH, HEIGHT, tag) {
                Ok(p) => eprintln!("[OK] Captura: {}", p.display()),
                Err(e) => eprintln!("[ERROR] {}", e),
            }
        }

        // Guardar / cargar estado
        if window.is_key_pressed(Key::F5, KeyRepeat::No) {
            match st.save_to_file() {
                Ok(p) => eprintln!("[OK] Estado guardado: {}", p.display()),
                Err(e) => eprintln!("[ERROR] {}", e),
            }
        }
        if window.is_key_pressed(Key::F9, KeyRepeat::No) {
            match st.load_from_file() {
                Ok(()) => changed = true,
                Err(e) => eprintln!("[ERROR] {}", e),
            }
        }
        if window.is_key_pressed(Key::F1, KeyRepeat::No) {
            if help_window.is_some() {
                help_window = None;
            } else {
                match Window::new(
                    "Ayuda — Controles",
                    HELP_WIDTH,
                    HELP_HEIGHT,
                    WindowOptions {
                        resize: false,
                        ..WindowOptions::default()
                    },
                ) {
                    Ok(mut w) => {
                        w.set_target_fps(30);
                        help_window = Some(w);
                    }
                    Err(e) => eprintln!("[ERROR] No se pudo crear ventana de ayuda: {}", e),
                }
            }
        }
        // Grabar secuencia de frames
        if window.is_key_pressed(Key::V, KeyRepeat::No) {
            if recorder.is_some() {
                if let Some(r) = recorder.take() {
                    eprintln!(
                        "[OK] Grabación detenida: {} frames en {}",
                        r.count(),
                              r.dir().display()
                    );
                }
            } else {
                match png::FrameRecorder::new(30) {
                    Ok(r) => {
                        eprintln!("[OK] Grabando a 30 fps en {}", r.dir().display());
                        recorder = Some(r);
                    }
                    Err(e) => eprintln!("[ERROR] {}", e),
                }
            }
        }

        // Auto-zoom
        if st.auto_zoom {
            st.viewport.zoom_at(
                WIDTH as f64 * 0.5,
                HEIGHT as f64 * 0.5,
                1.005,
                WIDTH,
                HEIGHT,
            );
            changed = true;
        }

        // ---------- Solicitar render ----------
        if changed {
            renderer.request(st.to_render_params(dragging));
        }

        // ---------- Recoger render completado ----------
        renderer.poll();

        // ---------- Componer frame ----------
        display.copy_from_slice(renderer.frame());
        ui::draw_palette_bar(&mut display, st.palette);
        ui::draw_overlay(
            &mut display,
            &st,
            mouse_screen,
            mouse_in,
            renderer.last_render_time(),
                         recorder.as_ref().map(|r| r.count()),
        );

        // ---------- Grabar frame ----------
        if let Some(rec) = recorder.as_mut() {
            if let Err(e) = rec.tick(&display, WIDTH, HEIGHT) {
                eprintln!("[ERROR] grabando frame: {}", e);
            }
        }
        // Actualizar ventana de ayuda si existe
        if let Some(ref mut hw) = help_window {
            if !hw.is_open() {
                help_window = None;
            } else {
                ui::draw_help_window(&mut help_buffer, HELP_WIDTH, HELP_HEIGHT);
                if hw.update_with_buffer(&help_buffer, HELP_WIDTH, HELP_HEIGHT).is_err() {
                    help_window = None;
                }
            }
        }
        // ---------- Presentar ----------
        if window
            .update_with_buffer(&display, WIDTH, HEIGHT)
            .is_err()
            {
                break;
            }

            last_mouse = mouse_screen;
        let _ = Instant::now(); // mantiene el import por si quieres medir
    }

    Ok(())
}
