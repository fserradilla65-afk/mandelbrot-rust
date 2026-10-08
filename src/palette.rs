use serde::{Deserialize, Serialize};

pub const LUT_SIZE: usize = 2048;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Palette {
    Viridis,
    Inferno,
    Magma,
    Plasma,
    Turbo,
    Cividis,
    Hot,
    Flag,
    Twilight,
    Grayscale,
}

impl Palette {
    /// Orden de ciclado — Grayscale siempre al final.
    pub const CYCLE: [Palette; 10] = [
        Palette::Viridis,
        Palette::Inferno,
        Palette::Magma,
        Palette::Plasma,
        Palette::Turbo,
        Palette::Cividis,
        Palette::Hot,
        Palette::Flag,
        Palette::Twilight,
        Palette::Grayscale,
    ];

    pub fn next(self) -> Self {
        let i = Self::CYCLE.iter().position(|p| *p == self).unwrap_or(0);
        Self::CYCLE[(i + 1) % Self::CYCLE.len()]
    }

    pub fn name(self) -> &'static str {
        match self {
            Palette::Viridis => "Viridis",
            Palette::Inferno => "Inferno",
            Palette::Magma => "Magma",
            Palette::Plasma => "Plasma",
            Palette::Turbo => "Turbo",
            Palette::Cividis => "Cividis",
            Palette::Hot => "Hot",
            Palette::Flag => "Flag",
            Palette::Twilight => "Twilight",
            Palette::Grayscale => "Grayscale",
        }
    }

    pub fn color(self, t: f64) -> u32 {
        let t = t.clamp(0.0, 1.0);
        let (r, g, b): (u8, u8, u8) = match self {
            Palette::Grayscale => {
                let v = (t * 255.0).round() as u8;
                (v, v, v)
            }
            Palette::Viridis => interpolate(&VIRIDIS, t),
            Palette::Inferno => interpolate(&INFERNO, t),
            Palette::Magma => interpolate(&MAGMA, t),
            Palette::Plasma => interpolate(&PLASMA, t),
            Palette::Turbo => interpolate(&TURBO, t),
            Palette::Cividis => interpolate(&CIVIDIS, t),
            Palette::Hot => interpolate(&HOT, t),
            Palette::Flag => interpolate(&FLAG, t),
            Palette::Twilight => interpolate(&TWILIGHT, t),
        };
        ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
    }
}

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
const MAGMA: [(f64, (u8, u8, u8)); 5] = [
    (0.00, (  0,   0,   4)),
    (0.25, ( 81,  18, 124)),
    (0.50, (183,  55, 121)),
    (0.75, (252, 137,  97)),
    (1.00, (252, 253, 191)),
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
// Azul → gris → amarillo, segura para daltonismo
const CIVIDIS: [(f64, (u8, u8, u8)); 5] = [
    (0.00, (  0,  32,  76)),
    (0.25, ( 39,  70, 119)),
    (0.50, ( 87, 108, 117)),
    (0.75, (154, 148,  90)),
    (1.00, (253, 233,  68)),
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

#[inline]
fn interpolate(anchors: &[(f64, (u8, u8, u8))], t: f64) -> (u8, u8, u8) {
    if t <= anchors[0].0 {
        return anchors[0].1;
    }
    let last = anchors[anchors.len() - 1];
    if t >= last.0 {
        return last.1;
    }
    for w in anchors.windows(2) {
        let (t0, c0) = w[0];
        let (t1, c1) = w[1];
        if t >= t0 && t <= t1 {
            let f = (t - t0) / (t1 - t0);
            let mix = |a: u8, b: u8| -> u8 {
                (a as f64 + f * (b as f64 - a as f64)).round().clamp(0.0, 255.0) as u8
            };
            return (mix(c0.0, c1.0), mix(c0.1, c1.1), mix(c0.2, c1.2));
        }
    }
    last.1
}

/// Tabla precalculada de 2048 entradas por paleta. Se construye una vez
/// por render y se indexa en el hot path (8 KB, cabe en L1).
pub struct PaletteLut {
    table: [u32; LUT_SIZE],
}

impl PaletteLut {
    pub fn new(p: Palette) -> Self {
        let mut table = [0u32; LUT_SIZE];
        for (i, slot) in table.iter_mut().enumerate() {
            *slot = p.color(i as f64 / (LUT_SIZE - 1) as f64);
        }
        Self { table }
    }

    #[inline(always)]
    pub fn sample(&self, t: f64) -> u32 {
        let idx = (t * (LUT_SIZE - 1) as f64) as usize;
        self.table[if idx >= LUT_SIZE { LUT_SIZE - 1 } else { idx }]
    }
}
