use crate::definitions::Definitions;
use crate::ecosystem::{Lineage, Simulation, SimulationRng};
use crate::habitat::Grid;
use crate::persistence::legacy::{legacy_cell::LegacyCell, legacy_traits::LegacyTraits};
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

/// Simulation d'un fichier antérieur au génome : seule la forme des organismes diffère.
#[derive(Deserialize)]
pub(super) struct LegacySimulation {
    grid: Grid,
    cells: Vec<LegacyCell>,
    generation: u32,
    rng: SimulationRng,
    id: Uuid,
    seed: u64,
    ticks: u64,
    lineage: Lineage,
}

impl LegacySimulation {
    pub(super) fn into_simulation(
        self,
        mut traits: HashMap<Uuid, LegacyTraits>,
        definitions: Arc<Definitions>,
    ) -> Simulation {
        let cells = self
            .cells
            .into_iter()
            .map(|cell| {
                let traits = traits.remove(&cell.id()).unwrap_or_else(LegacyTraits::founder);
                cell.into_cell(&traits, definitions.species().clone())
            })
            .collect();
        Simulation::from_parts(
            self.grid,
            cells,
            self.generation,
            self.rng,
            self.id,
            self.seed,
            self.ticks,
            self.lineage,
            definitions,
        )
    }
}
