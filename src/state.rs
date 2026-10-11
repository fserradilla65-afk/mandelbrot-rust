use crate::cli::CliArgs;
use crate::fractal::FractalKind;
use crate::palette::Palette;
use crate::render::RenderParams;
use crate::viewport::Viewport;
use crate::{ASPECT, MAX_ITER_DEFAULT};
use num_complex::Complex;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;

pub const HISTORY_MAX: usize = 500;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub viewport: Viewport,
    pub kind: FractalKind,
}

#[derive(Serialize, Deserialize)]
pub struct SavedState {
    pub version: u32,
    pub center_x: f64,
    pub center_y: f64,
    pub half_width: f64,
    pub half_height: f64,
    pub kind: FractalKind,
    pub max_iter: u32,
    pub palette: Palette,
    pub antialias: bool,
}
pub struct State {
    pub viewport: Viewport,
    pub kind: FractalKind,
    pub max_iter: u32,
    pub palette: Palette,
    pub antialias: bool,
    pub auto_zoom: bool,
    pub history: VecDeque<Snapshot>,
    pub last_mandelbrot: Option<Viewport>,
    pub initial_viewport: Viewport,
    pub initial_kind: FractalKind,
    pub initial_max_iter: u32,
}

impl State {
    pub fn new(args: CliArgs) -> Self {
        Self {
            viewport: args.viewport,
            kind: args.kind,
            max_iter: MAX_ITER_DEFAULT,
            palette: Palette::Viridis,
            antialias: false,
            auto_zoom: false,
            history: VecDeque::new(),
            last_mandelbrot: matches!(args.kind, FractalKind::Mandelbrot)
                .then_some(args.viewport),
            initial_viewport: args.viewport,
            initial_kind: args.kind,
            initial_max_iter: MAX_ITER_DEFAULT,
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        Snapshot { viewport: self.viewport, kind: self.kind }
    }

    pub fn push_history(&mut self) {
        if self.history.len() >= HISTORY_MAX {
            self.history.pop_front();
        }
        self.history.push_back(self.snapshot());
    }

    pub fn undo(&mut self) -> bool {
        if let Some(s) = self.history.pop_back() {
            self.viewport = s.viewport;
            self.kind = s.kind;
            true
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        self.viewport = self.initial_viewport;
        self.kind = self.initial_kind;
        self.max_iter = self.initial_max_iter;
        self.history.clear();
        self.last_mandelbrot = matches!(self.initial_kind, FractalKind::Mandelbrot)
            .then_some(self.initial_viewport);
    }

    pub fn jump_to_julia(&mut self, mx: f64, my: f64, w: usize, h: usize) -> bool {
        if !matches!(self.kind, FractalKind::Mandelbrot) {
            return false;
        }
        let (cr, ci) = self.viewport.pixel_to_complex(mx, my, w, h);
        self.push_history();
        self.last_mandelbrot = Some(self.viewport);
        self.kind = FractalKind::Julia { c: Complex::new(cr, ci) };
        self.viewport = Viewport::default_julia(ASPECT);
        true
    }

    pub fn back_to_mandelbrot(&mut self) -> bool {
        if matches!(self.kind, FractalKind::Mandelbrot) {
            return false;
        }
        self.push_history();
        self.kind = FractalKind::Mandelbrot;
        self.viewport = self.last_mandelbrot.unwrap_or(self.initial_viewport);
        true
    }

    pub fn to_render_params(&self, preview: bool) -> RenderParams {
        RenderParams {
            viewport: self.viewport,
            kind: self.kind,
            max_iter: self.max_iter,
            palette: self.palette,
            antialias: self.antialias,
            preview,
        }
    }

    pub fn save_to_file(&self) -> Result<PathBuf, String> {
        let path = crate::png::captures_dir()?.join("mandelbrot_state.json");
        let (cx, cy) = self.viewport.center();
        let s = SavedState {
            version: 2,
            center_x: cx,
            center_y: cy,
            half_width: (self.viewport.x_max - self.viewport.x_min) * 0.5,
            half_height: (self.viewport.y_max - self.viewport.y_min) * 0.5,
            kind: self.kind,
            max_iter: self.max_iter,
            palette: self.palette,
            antialias: self.antialias,
        };
        let json = serde_json::to_string_pretty(&s)
        .map_err(|e| format!("no se pudo serializar: {}", e))?;
        std::fs::write(&path, json)
        .map_err(|e| format!("no se pudo escribir '{}': {}", path.display(), e))?;
        Ok(path)
    }

    pub fn load_from_file(&mut self) -> Result<(), String> {
        let path = crate::png::captures_dir()?.join("mandelbrot_state.json");
        let data = std::fs::read_to_string(&path)
        .map_err(|e| format!("no se pudo leer '{}': {}", path.display(), e))?;
        let s: SavedState = serde_json::from_str(&data)
        .map_err(|e| format!("JSON inválido: {}", e))?;
        self.viewport = Viewport {
            x_min: s.center_x - s.half_width,
            x_max: s.center_x + s.half_width,
            y_min: s.center_y - s.half_height,
            y_max: s.center_y + s.half_height,
        };
        self.kind = s.kind;
        self.max_iter = s.max_iter.max(MAX_ITER_DEFAULT.min(100));
        self.palette = s.palette;
        self.antialias = s.antialias;
        self.history.clear();
        self.last_mandelbrot = matches!(s.kind, FractalKind::Mandelbrot)
        .then_some(self.viewport);
        Ok(())
    }
}
