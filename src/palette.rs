use serde::{Deserialize, Serialize};

pub const LUT_SIZE: usize = 2048;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Palette {
    Viridis,
    Inferno,
    Ocean,
    Turbo,
    NipySpectral,
    Cubehelix,
    Hot,
    Flag,
    Twilight,
    Grayscale,
}

impl Palette {
    /// Orden de ciclado
    pub const CYCLE: [Palette; 10] = [
        Palette::Viridis,
        Palette::Inferno,
        Palette::Ocean,
        Palette::Turbo,
        Palette::Twilight,
        Palette::Cubehelix,
        Palette::Hot,
        Palette::NipySpectral,
        Palette::Flag,
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
            Palette::Cubehelix => "Cubehelix",
            Palette::Ocean => "Ocean",
            Palette::Turbo => "Turbo",
            Palette::NipySpectral => "NipySpectral",
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
            Palette::Cubehelix => interpolate(&CUBEHELIX, t),
            Palette::Ocean => interpolate(&OCEAN, t),
            Palette::Turbo => interpolate(&TURBO, t),
            Palette::NipySpectral => interpolate(&NIPY_SPECTRAL, t),
            Palette::Hot => interpolate(&HOT, t),
            Palette::Flag => interpolate(&FLAG, t),
            Palette::Twilight => interpolate(&TWILIGHT, t),
        };
        ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
    }

    /// Devuelve la paleta asociada a un número del 0 al 9.
    /// El 1 corresponde a la primera paleta del ciclo y el 0 a la última (Grayscale).
    pub fn from_digit(d: u8) -> Option<Self> {
        if d == 0 {
            // La tecla 0 va directa a Grayscale (el final del ciclo)
            Some(Palette::Grayscale)
        } else if (1..=9).contains(&d) {
            // Las teclas 1..9 mapean al principio del ciclo
            // (1 -> índice 0, 2 -> índice 1, ...)
            Some(Self::CYCLE[(d - 1) as usize])
        } else {
            None
        }
    }    
}
// Paleta viridis: 8 puntos de muestreo
const VIRIDIS: [(f64, (u8, u8, u8)); 8] = [
    (0.00, ( 68,   1,  84)),
    (0.14, ( 70,  49, 126)),
    (0.29, ( 54,  91, 140)),
    (0.43, ( 39, 126, 142)),
    (0.57, ( 31, 161, 135)),
    (0.71, ( 73, 193, 109)),
    (0.86, (159, 217,  56)),
    (1.00, (253, 231,  36)),
];
// Paleta inferno: 8 puntos de muestreo
const INFERNO: [(f64, (u8, u8, u8)); 8] = [
    (0.00, (  0,   0,   3)),
    (0.14, ( 39,  11,  82)),
    (0.29, (101,  21, 110)),
    (0.43, (158,  41,  99)),
    (0.57, (212,  72,  65)),
    (0.71, (245, 124,  21)),
    (0.86, (250, 193,  40)),
    (1.00, (252, 254, 164)),
];
// Paleta ocean: 8 puntos de muestreo
const OCEAN: [(f64, (u8, u8, u8)); 8] = [
    (0.00, (  0, 127,   0)),
    (0.14, (  0,  73,  36)),
    (0.29, (  0,  18,  73)),
    (0.43, (  0,  35, 109)),
    (0.57, (  0,  91, 146)),
    (0.71, ( 35, 145, 182)),
    (0.86, (146, 200, 219)),
    (1.00, (255, 255, 255)),
];
// Paleta turbo: 8 puntos de muestreo
const TURBO: [(f64, (u8, u8, u8)); 8] = [
    (0.00, ( 48,  18,  59)),
    (0.14, ( 70, 117, 237)),
    (0.29, ( 27, 207, 212)),
    (0.43, ( 97, 252, 108)),
    (0.57, (209, 232,  52)),
    (0.71, (254, 155,  45)),
    (0.86, (217,  56,   6)),
    (1.00, (122,   4,   2)),
];
// Paleta hot: 7 puntos de muestreo
const HOT: [(f64, (u8, u8, u8)); 7] = [
    (0.00, ( 10,   0,   0)),
    (0.17, (120,   0,   0)),
    (0.33, (233,   0,   0)),
    (0.50, (255,  91,   0)),
    (0.67, (255, 201,   0)),
    (0.83, (255, 255,  89)),
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
// Paleta twilight: 8 puntos de muestreo
const TWILIGHT: [(f64, (u8, u8, u8)); 8] = [
    (0.00, (225, 216, 226)),
    (0.14, (137, 172, 196)),
    (0.29, ( 95,  97, 180)),
    (0.43, ( 73,  21, 100)),
    (0.57, ( 80,  20,  68)),
    (0.71, (164,  66,  79)),
    (0.86, (201, 152, 123)),
    (1.00, (225, 216, 225)),
];
// Paleta cubehelix: 8 puntos de muestreo
const CUBEHELIX: [(f64, (u8, u8, u8)); 8] = [
    (0.00, (  0,   0,   0)),
    (0.14, ( 26,  35,  65)),
    (0.29, ( 27,  97,  68)),
    (0.43, (104, 122,  48)),
    (0.57, (199, 122, 124)),
    (0.71, (205, 162, 224)),
    (0.86, (198, 225, 241)),
    (1.00, (255, 255, 255)),
];
// Paleta NipySpectral: negra → violeta → azul → verde → amarillo → naranja → rojo → gris.
// No es perceptualmente uniforme, pero separa las zonas de escape muy bien.
const NIPY_SPECTRAL: [(f64, (u8, u8, u8)); 9] = [
    (0.000, (  0,   0,   0)), // Negro (interior o escape muy bajo)
    (0.125, ( 70,   0, 130)), // Violeta oscuro
    (0.250, (  0,   0, 200)), // Azul
    (0.375, (  0, 180, 180)), // Cian / Verde azulado
    (0.500, (  0, 200,   0)), // Verde
    (0.625, (200, 200,   0)), // Amarillo
    (0.750, (255, 100,   0)), // Naranja
    (0.875, (200,   0,   0)), // Rojo
    (1.000, (128, 128, 128)), // Gris claro (escape muy alto)
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
