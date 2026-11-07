#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub(crate) struct TemperatureMap {
    size: usize,
    temperatures: Vec<Vec<f32>>,
    ambient_temperature: f32,
}

impl TemperatureMap {
    pub(crate) fn new(size: usize, ambient_temperature: f32) -> Self {
        Self {
            size,
            temperatures: vec![vec![ambient_temperature; size]; size],
            ambient_temperature,
        }
    }

    pub(crate) fn size(&self) -> usize {
        self.size
    }

    pub(crate) fn temperature(&self, x: usize, y: usize) -> f32 {
        self.temperatures[y][x]
    }

    pub(crate) fn ambient_temperature(&self) -> f32 {
        self.ambient_temperature
    }

    pub(crate) fn get(&self, x: usize, y: usize) -> f32 {
        self.temperature(x, y)
    }

    pub(crate) fn apply_heat_source(&mut self, x: usize, y: usize, intensity: f32) {
        // Seules les cases à distance < intensité sont modifiées : on ne balaie que ce carré,
        // dans le même ordre que le parcours complet, donc avec les mêmes additions flottantes.
        let reach = intensity.ceil() as i64;
        let (cx, cy, size) = (x as i64, y as i64, self.size() as i64);
        for dy in (cy - reach).max(0)..=(cy + reach).min(size - 1) {
            for dx in (cx - reach).max(0)..=(cx + reach).min(size - 1) {
                let distance = (((dx - cx).pow(2) + (dy - cy).pow(2)) as f32).sqrt();
                if distance < intensity {
                    self.temperatures[dy as usize][dx as usize] += (intensity - distance) * 2.0;
                }
            }
        }
    }

    pub(crate) fn validate(&self, size: usize) -> bool {
        self.size == size
            && self.ambient_temperature.is_finite()
            && self.temperatures.len() == size
            && self
                .temperatures
                .iter()
                .all(|row| row.len() == size && row.iter().all(|v| v.is_finite()))
    }
    pub(crate) fn reset_to_ambient(&mut self) {
        for row in &mut self.temperatures {
            for temp in row {
                *temp = self.ambient_temperature;
            }
        }
    }
}
