use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Viewport {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}

impl Viewport {
    pub fn new_centered(cx: f64, cy: f64, half_width: f64, aspect: f64) -> Self {
        let half_height = half_width * aspect;
        Self {
            x_min: cx - half_width,
            x_max: cx + half_width,
            y_min: cy - half_height,
            y_max: cy + half_height,
        }
    }

    pub fn default_mandelbrot(aspect: f64) -> Self {
        Self::new_centered(-0.5, 0.0, 1.5, aspect)
    }

    pub fn default_julia(aspect: f64) -> Self {
        Self::new_centered(0.0, 0.0, 2.0, aspect)
    }

    pub fn from_center_zoom(cx: f64, cy: f64, zoom: f64, aspect: f64) -> Self {
        Self::new_centered(cx, cy, 1.5 / zoom, aspect)
    }

    pub fn center(&self) -> (f64, f64) {
        ((self.x_min + self.x_max) * 0.5, (self.y_min + self.y_max) * 0.5)
    }

    pub fn zoom(&self) -> f64 {
        let span = (self.x_max - self.x_min).min(self.y_max - self.y_min);
        3.0 / span
    }

    #[inline]
    pub fn pixel_to_complex(&self, px: f64, py: f64, w: usize, h: usize) -> (f64, f64) {
        let sx = (self.x_max - self.x_min) / (w as f64 - 1.0);
        let sy = (self.y_max - self.y_min) / (h as f64 - 1.0);
        (self.x_min + px * sx, self.y_min + py * sy)
    }

    pub fn zoom_at(&mut self, mx: f64, my: f64, factor: f64, w: usize, h: usize) {
        if factor <= 0.0 { return; }
        let (cx, cy) = self.pixel_to_complex(mx, my, w, h);
        let hw = (self.x_max - self.x_min) * 0.5 / factor;
        let hh = (self.y_max - self.y_min) * 0.5 / factor;
        self.x_min = cx - hw;
        self.x_max = cx + hw;
        self.y_min = cy - hh;
        self.y_max = cy + hh;
    }

    /// Desplaza el viewport en píxeles. `dx > 0` mueve la ventana hacia la derecha
    /// (se ve contenido más al este).
    pub fn pan_pixels(&mut self, dx: f64, dy: f64, w: usize, h: usize) {
        let sx = (self.x_max - self.x_min) / (w as f64 - 1.0);
        let sy = (self.y_max - self.y_min) / (h as f64 - 1.0);
        let shift_x = dx * sx;
        let shift_y = dy * sy;
        self.x_min += shift_x;
        self.x_max += shift_x;
        self.y_min += shift_y;
        self.y_max += shift_y;
    }
}
