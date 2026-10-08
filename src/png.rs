use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub fn captures_dir() -> Result<PathBuf, String> {
    let base = dirs::picture_dir()
        .or_else(dirs::home_dir)
        .ok_or_else(|| "no se pudo determinar la carpeta de imágenes".to_string())?;
    let dir = base.join("Capturas");
    fs::create_dir_all(&dir)
        .map_err(|e| format!("no se pudo crear '{}': {}", dir.display(), e))?;
    Ok(dir)
}

fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn save_png(buffer: &[u32], w: usize, h: usize, tag: &str) -> Result<PathBuf, String> {
    let dir = captures_dir()?;
    let filename = dir.join(format!("mandelbrot_{}_{}.png", timestamp(), tag));
    write_png(buffer, w, h, &filename)?;
    Ok(filename)
}

fn write_png(buffer: &[u32], w: usize, h: usize, path: &Path) -> Result<(), String> {
    let mut bytes = Vec::with_capacity(w * h * 3);
    for &p in buffer {
        bytes.push(((p >> 16) & 0xFF) as u8);
        bytes.push(((p >> 8) & 0xFF) as u8);
        bytes.push((p & 0xFF) as u8);
    }
    image::save_buffer(path, &bytes, w as u32, h as u32, image::ColorType::Rgb8)
        .map_err(|e| format!("no se pudo guardar '{}': {}", path.display(), e))
}

/// Graba una secuencia de PNG numerados (útil para componer vídeo con ffmpeg).
pub struct FrameRecorder {
    dir: PathBuf,
    counter: u32,
    last_save: Instant,
    interval: Duration,
}

impl FrameRecorder {
    pub fn new(fps: u32) -> Result<Self, String> {
        let fps = fps.max(1);
        let dir = captures_dir()?.join(format!("frames_{}", timestamp()));
        fs::create_dir_all(&dir)
            .map_err(|e| format!("no se pudo crear '{}': {}", dir.display(), e))?;
        Ok(Self {
            dir,
            counter: 0,
            last_save: Instant::now() - Duration::from_secs(1),
            interval: Duration::from_millis(1000 / fps as u64),
        })
    }

    pub fn tick(&mut self, buffer: &[u32], w: usize, h: usize) -> Result<bool, String> {
        if self.last_save.elapsed() < self.interval {
            return Ok(false);
        }
        let path = self.dir.join(format!("frame_{:05}.png", self.counter));
        write_png(buffer, w, h, &path)?;
        self.counter += 1;
        self.last_save = Instant::now();
        Ok(true)
    }

    pub fn dir(&self) -> &Path { &self.dir }
    pub fn count(&self) -> u32 { self.counter }
}
