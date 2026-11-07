use serde::{Deserialize, Serialize};

/// Instantané des grandeurs suivies d'une population. L'énergie vaut NaN quand elle est
/// inconnue (historique reconstruit depuis la filiation d'une ancienne sauvegarde).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct PopulationSample {
    time: f64,
    cells: f32,
    births: f32,
    deaths: f32,
    energy: f32,
}

impl PopulationSample {
    pub(crate) fn new(time: f64, cells: f32, births: f32, deaths: f32, energy: f32) -> Self {
        Self {
            time,
            cells,
            births,
            deaths,
            energy,
        }
    }

    /// Temps simulé en secondes.
    pub(crate) fn time(&self) -> f64 {
        self.time
    }
    pub(crate) fn cells(&self) -> f32 {
        self.cells
    }
    pub(crate) fn births(&self) -> f32 {
        self.births
    }
    pub(crate) fn deaths(&self) -> f32 {
        self.deaths
    }
    pub(crate) fn energy(&self) -> f32 {
        self.energy
    }

    pub(crate) fn is_valid(&self) -> bool {
        self.time.is_finite()
            && self.time >= 0.0
            && [self.cells, self.births, self.deaths]
                .iter()
                .all(|v| v.is_finite() && *v >= 0.0)
            && !self.energy.is_infinite()
    }
}
