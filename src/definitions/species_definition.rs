use crate::definitions::{BiologyRules, GenomeDefinition};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SpeciesDefinition {
    id: Uuid,
    name: String,
    /// Génome de l'ancêtre : il ne fixe que le point de départ, la suite relève de l'évolution.
    genome: GenomeDefinition,
    rules: BiologyRules,
}

impl SpeciesDefinition {
    pub(crate) fn id(&self) -> Uuid {
        self.id
    }
    pub(crate) fn name(&self) -> &str {
        &self.name
    }
    pub(crate) fn genome(&self) -> &GenomeDefinition {
        &self.genome
    }
    pub(crate) fn rules(&self) -> &BiologyRules {
        &self.rules
    }
}
