use crate::definitions::Definitions;
use crate::ecosystem::SimulationClock;
use crate::persistence::{
    Session, SessionView,
    legacy::{legacy_simulation::LegacySimulation, legacy_traits::LegacyTraits},
};
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

/// Corps de session d'un fichier antérieur au génome (versions 1 à 6).
#[derive(Deserialize)]
pub(crate) struct LegacySession {
    simulation: LegacySimulation,
    view: SessionView,
    clock: SimulationClock,
}

impl LegacySession {
    /// Convertit la session : `traits` vient de la section « vivant » des fichiers de version 6.
    pub(crate) fn into_session(
        self,
        traits: Vec<(Uuid, LegacyTraits)>,
        definitions: Arc<Definitions>,
    ) -> Session {
        let simulation = self
            .simulation
            .into_simulation(traits.into_iter().collect::<HashMap<_, _>>(), definitions);
        Session::from_parts(simulation, self.view, self.clock)
    }
}
