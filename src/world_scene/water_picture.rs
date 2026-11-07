use crate::hydrology::Hydrology;

/// Peinture de l'eau : relief ombré, humidité du sol, nappes (turbides selon les sédiments) et
/// nuages, calculés à partir de l'état hydrologique. Les champs, définis par site, sont
/// interpolés entre les centres de sites : aucune frontière de case n'apparaît.
pub(super) struct WaterPicture {
    size: usize,
    side: usize,
    sky_side: usize,
}

type Rgb = [f32; 3];

fn smoothstep(low: f32, high: f32, value: f32) -> f32 {
    let t = ((value - low) / (high - low)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn mix(a: Rgb, b: Rgb, t: f32) -> Rgb {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// Superpose une couche (couleur, opacité) à un empilement prémultiplié.
fn over(stack: &mut ([f32; 3], f32), color: Rgb, alpha: f32) {
    for channel in 0..3 {
        stack.0[channel] = color[channel] * alpha + stack.0[channel] * (1.0 - alpha);
    }
    stack.1 = alpha + stack.1 * (1.0 - alpha);
}

impl WaterPicture {
    pub(super) fn new(size: usize) -> Self {
        Self {
            size,
            side: size * (384 / size.max(1)).clamp(2, 16),
            sky_side: size * (256 / size.max(1)).clamp(2, 8),
        }
    }

    pub(super) fn side(&self) -> u32 {
        self.side as u32
    }

    pub(super) fn sky_side(&self) -> u32 {
        self.sky_side as u32
    }

    /// Échantillon bilinéaire d'un champ par site, au point `(u, v)` exprimé en sites.
    fn sample(&self, field: &[f32], u: f32, v: f32) -> f32 {
        let last = self.size as i64 - 1;
        let (x, y) = (u - 0.5, v - 0.5);
        let (fx, fy) = (x.floor(), y.floor());
        let (tx, ty) = (x - fx, y - fy);
        let at = |dx: i64, dy: i64| {
            let cx = (fx as i64 + dx).clamp(0, last) as usize;
            let cy = (fy as i64 + dy).clamp(0, last) as usize;
            field[cy * self.size + cx]
        };
        (at(0, 0) * (1.0 - tx) + at(1, 0) * tx) * (1.0 - ty)
            + (at(0, 1) * (1.0 - tx) + at(1, 1) * tx) * ty
    }

    fn field(&self, hydrology: &Hydrology, read: impl Fn(&Hydrology, usize, usize) -> f32) -> Vec<f32> {
        (0..self.size * self.size)
            .map(|index| read(hydrology, index % self.size, index / self.size))
            .collect()
    }

    /// Surcouche du sol : RGBA (alpha droit) de côté [`WaterPicture::side`].
    pub(super) fn surface(&self, hydrology: &Hydrology) -> Vec<u8> {
        let size = self.size;
        let depth = self.field(hydrology, |h, x, y| h.depth(x, y));
        let wetness = self.field(hydrology, |h, x, y| h.wetness(x, y));
        let mud = self.field(hydrology, |h, x, y| h.turbidity(x, y));
        let elevation = self.field(hydrology, |h, x, y| h.elevation(x, y));
        let slope = |dx: i64, dy: i64| {
            (0..size * size)
                .map(|index| {
                    let at = |ox: i64, oy: i64| {
                        let x = ((index % size) as i64 + ox).clamp(0, size as i64 - 1) as usize;
                        let y = ((index / size) as i64 + oy).clamp(0, size as i64 - 1) as usize;
                        elevation[y * size + x]
                    };
                    (at(dx, dy) - at(-dx, -dy)) * 0.5
                })
                .collect::<Vec<f32>>()
        };
        let (rise_east, rise_south) = (slope(1, 0), slope(0, 1));

        let mut pixels = vec![0u8; self.side * self.side * 4];
        for ty in 0..self.side {
            for tx in 0..self.side {
                let u = (tx as f32 + 0.5) / self.side as f32 * size as f32;
                let v = (ty as f32 + 0.5) / self.side as f32 * size as f32;
                let mut stack = ([0.0; 3], 0.0);

                // Relief éclairé depuis le nord-ouest : les ravines et les dépôts se lisent.
                let shade = 0.5 - 1.3 * 0.7 * (self.sample(&rise_east, u, v) + self.sample(&rise_south, u, v));
                if shade > 0.5 {
                    over(&mut stack, [255.0; 3], ((shade - 0.5) * 0.5).clamp(0.0, 0.4));
                } else {
                    over(&mut stack, [0.0; 3], ((0.5 - shade) * 0.8).clamp(0.0, 0.55));
                }

                // Humidité : le sol détrempé fonce, le sol sec pâlit.
                let wet = self.sample(&wetness, u, v);
                over(&mut stack, [46.0, 34.0, 24.0], 0.36 * wet.powf(1.2));
                over(&mut stack, [196.0, 160.0, 98.0], 0.16 * (1.0 - wet / 0.3).clamp(0.0, 1.0));

                // Nappes d'eau, de la lame translucide au bassin profond ; la boue la trouble.
                let water = self.sample(&depth, u, v);
                let alpha = (1.0 - (-water * 5.0).exp()) * 0.88;
                if alpha > 0.002 {
                    let clear = mix([78.0, 178.0, 196.0], [14.0, 52.0, 112.0], (water / 0.9).min(1.0));
                    let color = mix(clear, [118.0, 90.0, 60.0], self.sample(&mud, u, v).clamp(0.0, 0.85));
                    over(&mut stack, color, alpha);
                }

                let offset = (ty * self.side + tx) * 4;
                let coverage = stack.1.max(1e-4);
                for channel in 0..3 {
                    pixels[offset + channel] = (stack.0[channel] / coverage).clamp(0.0, 255.0) as u8;
                }
                pixels[offset + 3] = (stack.1 * 255.0).clamp(0.0, 255.0) as u8;
            }
        }
        pixels
    }

    /// Voile nuageux : blanc, plus sombre et dense sous la pluie.
    pub(super) fn sky(&self, hydrology: &Hydrology) -> Vec<u8> {
        let size = self.size;
        let cloud = self.field(hydrology, |h, x, y| h.cloud(x, y));
        let rain = self.field(hydrology, |h, x, y| h.rain(x, y));
        let mut pixels = vec![0u8; self.sky_side * self.sky_side * 4];
        for ty in 0..self.sky_side {
            for tx in 0..self.sky_side {
                let u = (tx as f32 + 0.5) / self.sky_side as f32 * size as f32;
                let v = (ty as f32 + 0.5) / self.sky_side as f32 * size as f32;
                let density = smoothstep(0.002, 0.07, self.sample(&cloud, u, v));
                let storm = smoothstep(0.0, 0.01, self.sample(&rain, u, v));
                let color = mix([238.0, 243.0, 250.0], [112.0, 124.0, 144.0], storm);
                let offset = (ty * self.sky_side + tx) * 4;
                for channel in 0..3 {
                    pixels[offset + channel] = color[channel] as u8;
                }
                // Le voile s'estompe vers les bords du monde au lieu de s'y couper net.
                let edge = smoothstep(0.0, 0.12, (tx.min(self.sky_side - 1 - tx).min(ty).min(self.sky_side - 1 - ty)) as f32 / self.sky_side as f32);
                pixels[offset + 3] = (density * edge * (0.38 + 0.22 * storm) * 255.0) as u8;
            }
        }
        pixels
    }
}
