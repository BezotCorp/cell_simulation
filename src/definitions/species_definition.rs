use crate::{
    biology::CellGenes,
    definitions::{BiologyRules, NeuralDefinition},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SpeciesDefinition {
    id: Uuid,
    name: String,
    initial_genes: CellGenes,
    neural: NeuralDefinition,
    rules: BiologyRules,
}

impl SpeciesDefinition {
    pub(crate) fn id(&self) -> Uuid {
        self.id
    }
    pub(crate) fn name(&self) -> &str {
        &self.name
    }
    pub(crate) fn initial_genes(&self) -> &CellGenes {
        &self.initial_genes
    }
    pub(crate) fn neural(&self) -> &NeuralDefinition {
        &self.neural
    }
    pub(crate) fn rules(&self) -> &BiologyRules {
        &self.rules
    }
}
