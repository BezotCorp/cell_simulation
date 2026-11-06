pub(crate) struct SimulationStats {
    cell_count: usize,
    generation: u32,
    water_level: f32,
    average_energy: f32,
}

impl SimulationStats {
    pub(crate) fn new(
        cell_count: usize,
        generation: u32,
        water_level: f32,
        average_energy: f32,
    ) -> Self {
        Self {
            cell_count,
            generation,
            water_level,
            average_energy,
        }
    }

    pub(crate) fn cell_count(&self) -> usize {
        self.cell_count
    }

    pub(crate) fn generation(&self) -> u32 {
        self.generation
    }

    pub(crate) fn water_level(&self) -> f32 {
        self.water_level
    }

    pub(crate) fn average_energy(&self) -> f32 {
        self.average_energy
    }
}
