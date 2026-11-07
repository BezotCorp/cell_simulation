#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PopulationSummary {
    cells: usize,
    births: u64,
    deaths: u64,
    elapsed_seconds: f64,
    average_energy: f32,
    water_level: f32,
}

impl PopulationSummary {
    pub(crate) fn new(
        cells: usize,
        births: u64,
        deaths: u64,
        elapsed_seconds: f64,
        average_energy: f32,
        water_level: f32,
    ) -> Self {
        Self {
            cells,
            births,
            deaths,
            elapsed_seconds,
            average_energy,
            water_level,
        }
    }

    pub(crate) fn cells(&self) -> usize {
        self.cells
    }
    pub(crate) fn births(&self) -> u64 {
        self.births
    }
    pub(crate) fn deaths(&self) -> u64 {
        self.deaths
    }
    pub(crate) fn elapsed_seconds(&self) -> f64 {
        self.elapsed_seconds
    }
    pub(crate) fn average_energy(&self) -> f32 {
        self.average_energy
    }
    pub(crate) fn water_level(&self) -> f32 {
        self.water_level
    }
}
