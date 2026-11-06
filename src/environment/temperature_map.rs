pub(crate) struct TemperatureMap {
    size: usize,
    temperatures: Vec<Vec<f32>>,
    ambient_temperature: f32,
}

impl TemperatureMap {
    pub(crate) fn new(size: usize) -> Self {
        Self {
            size,
            temperatures: vec![vec![20.0; size]; size],
            ambient_temperature: 20.0,
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
        self.temperatures[y][x]
    }

    pub(crate) fn apply_heat_source(&mut self, x: usize, y: usize, intensity: f32) {
        for dy in 0..self.size {
            for dx in 0..self.size {
                let distance =
                    (((dx as i32 - x as i32).pow(2) + (dy as i32 - y as i32).pow(2)) as f32).sqrt();
                if distance < intensity {
                    self.temperatures[dy][dx] += (intensity - distance) * 2.0;
                }
            }
        }
    }

    pub(crate) fn reset_to_ambient(&mut self) {
        for row in &mut self.temperatures {
            for temp in row {
                *temp = self.ambient_temperature;
            }
        }
    }
}
