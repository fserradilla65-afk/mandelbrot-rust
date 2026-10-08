use crate::fractal::{self, FractalKind};
use crate::palette::{Palette, PaletteLut};
use crate::viewport::Viewport;
use crate::{HEIGHT, WIDTH};
use rayon::prelude::*;
use std::sync::mpsc;
use std::time::{Duration, Instant};

const ESCAPE_SQ: f64 = 4.0;
const COLOR_SCALE: f64 = 0.04;

#[derive(Clone, Copy)]
pub struct RenderParams {
    pub viewport: Viewport,
    pub kind: FractalKind,
    pub max_iter: u32,
    pub palette: Palette,
    pub antialias: bool,
    pub preview: bool,
}

// ---------------------------------------------------------------
//  Render puro (síncrono)
// ---------------------------------------------------------------

pub fn render(buffer: &mut [u32], w: usize, h: usize, p: &RenderParams) {
    let lut = PaletteLut::new(p.palette);

    if p.preview {
        let sw = (w + 1) / 2;
        let sh = (h + 1) / 2;
        let mut small = vec![0u32; sw * sh];
        render_impl(&mut small, sw, sh, p, &lut, false);
        // Upscale nearest-neighbor
        for y in 0..h {
            let sy = (y / 2).min(sh - 1);
            let src = &small[sy * sw..(sy + 1) * sw];
            let row = &mut buffer[y * w..(y + 1) * w];
            for x in 0..w {
                row[x] = src[(x / 2).min(sw - 1)];
            }
        }
    } else {
        render_impl(buffer, w, h, p, &lut, p.antialias);
    }
}

fn render_impl(
    buffer: &mut [u32],
    w: usize,
    h: usize,
    p: &RenderParams,
    lut: &PaletteLut,
    aa: bool,
) {
    let vp = p.viewport;
    let scale_x = (vp.x_max - vp.x_min) / (w as f64 - 1.0);
    let scale_y = (vp.y_max - vp.y_min) / (h as f64 - 1.0);
    let kind = p.kind;
    let max_iter = p.max_iter;
    let is_mandel = matches!(kind, FractalKind::Mandelbrot);

    // AA 2x2
    let samples: u32 = if aa { 2 } else { 1 };
    let sample_step_x = scale_x / samples as f64;
    let sample_step_y = scale_y / samples as f64;
    let half = 0.5 * (samples as f64 - 1.0);
    let inv_n = 1.0 / (samples * samples) as f64;

    buffer.par_chunks_mut(w).enumerate().for_each(|(y, row)| {
        let cy = vp.y_min + y as f64 * scale_y;
        for x in 0..w {
            let cx = vp.x_min + x as f64 * scale_x;

            row[x] = if aa {
                let (mut r, mut g, mut b) = (0u32, 0u32, 0u32);
                for sy in 0..samples {
                    for sx in 0..samples {
                        let px = cx + (sx as f64 - half) * sample_step_x;
                        let py = cy + (sy as f64 - half) * sample_step_y;
                        let c = sample_pixel(px, py, kind, max_iter, lut, is_mandel);
                        r += (c >> 16) & 0xFF;
                        g += (c >> 8) & 0xFF;
                        b += c & 0xFF;
                    }
                }
                let (r, g, b) = (
                    (r as f64 * inv_n).round() as u32,
                                 (g as f64 * inv_n).round() as u32,
                                 (b as f64 * inv_n).round() as u32,
                );
                (r << 16) | (g << 8) | b
            } else {
                sample_pixel(cx, cy, kind, max_iter, lut, is_mandel)
            };
        }
    });
}

#[inline(always)]
fn sample_pixel(
    cx: f64,
    cy: f64,
    kind: FractalKind,
    max_iter: u32,
    lut: &PaletteLut,
    is_mandel: bool,
) -> u32 {
    // Interior trivial (solo Mandelbrot).
    if is_mandel && fractal::in_mandelbrot_interior(cx, cy) {
        return 0x0000_0000;
    }

    let (zr0, zi0, cr, ci) = match kind {
        FractalKind::Mandelbrot => (0.0, 0.0, cx, cy),
        FractalKind::Julia { c } => (cx, cy, c.re, c.im),
    };

    let (iter, zr, zi) = fractal::iterate(zr0, zi0, cr, ci, max_iter, ESCAPE_SQ, true);
    if iter >= max_iter {
        return 0x0000_0000;
    }

    let z_sq = zr * zr + zi * zi;
    let log2_z = 0.5 * z_sq.log2();       // log₂|zₙ|
    let nu = log2_z.log2();               // log₂(log₂|zₙ|)
    let smooth = iter as f64 + 1.0 - nu;

    let t = (smooth * COLOR_SCALE).rem_euclid(1.0);
    lut.sample(t)
}

// ---------------------------------------------------------------
//  Renderer en hilo aparte con descarte de frames obsoletos
// ---------------------------------------------------------------

struct Request {
    generation: u64,
    params: RenderParams,
    buffer: Vec<u32>,
}

struct Response {
    generation: u64,
    buffer: Vec<u32>,
}

pub struct Renderer {
    req_tx: mpsc::Sender<Request>,
    resp_rx: mpsc::Receiver<Response>,
    next_generation: u64,
    in_flight: Option<u64>,
    display: Vec<u32>,
    spare: Option<Vec<u32>>,
    pending: Option<RenderParams>,
    submit_time: Option<Instant>,
    last_duration: Duration,
}

impl Renderer {
    pub fn new() -> Self {
        let (req_tx, req_rx) = mpsc::channel::<Request>();
        let (resp_tx, resp_rx) = mpsc::channel::<Response>();

        std::thread::Builder::new()
        .name("fractal-renderer".into())
        .spawn(move || {
            while let Ok(req) = req_rx.recv() {
                let Request { generation, params, mut buffer } = req;
                render(&mut buffer, WIDTH, HEIGHT, &params);
                if resp_tx.send(Response { generation, buffer }).is_err() {
                    break;
                }
            }
        })
        .expect("no se pudo crear el hilo de render");

        Self {
            req_tx,
            resp_rx,
            next_generation: 1,
            in_flight: None,
            display: vec![0u32; WIDTH * HEIGHT],
            spare: Some(vec![0u32; WIDTH * HEIGHT]),
            pending: None,
            submit_time: None,
            last_duration: Duration::ZERO,
        }
    }

    /// Pide un render. Si ya hay uno en vuelo, guarda solo el último pedido.
    pub fn request(&mut self, params: RenderParams) {
        if self.in_flight.is_some() {
            self.pending = Some(params);
        } else {
            self.submit(params);
        }
    }

    fn submit(&mut self, params: RenderParams) {
        let buffer = match self.spare.take() {
            Some(b) => b,
            None => return,
        };
        let generation = self.next_generation;
        self.next_generation += 1;
        self.in_flight = Some(generation);
        self.submit_time = Some(Instant::now());
        let req = Request { generation, params, buffer };
        if let Err(e) = self.req_tx.send(req) {
            let Request { buffer, .. } = e.0;
            self.spare = Some(buffer);
            self.in_flight = None;
            self.submit_time = None;
        }
    }

    /// Recoge respuestas. Devuelve `true` si el frame visible se actualizó.
    pub fn poll(&mut self) -> bool {
        let mut updated = false;
        while let Ok(resp) = self.resp_rx.try_recv() {
            if Some(resp.generation) == self.in_flight {
                if let Some(t0) = self.submit_time.take() {
                    self.last_duration = t0.elapsed();
                }
                self.spare = Some(std::mem::replace(&mut self.display, resp.buffer));
                self.in_flight = None;
                updated = true;
            } else {
                self.spare = Some(resp.buffer);
            }
        }
        if updated {
            if let Some(p) = self.pending.take() {
                self.submit(p);
            }
        }
        updated
    }

    pub fn frame(&self) -> &[u32] {
        &self.display
    }

    pub fn last_render_time(&self) -> Duration {
        self.last_duration
    }
}
