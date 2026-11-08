use crate::definitions::{
    CapabilityDefinition, CostDefinition, MotifDefinition, MutationDefinition, NeuralDefinition,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Toute la génétique du jeu, définie par les données : les motifs, les règles qui en font des
/// capacités, le réseau neuronal, les taux de mutation et les coûts. Le moteur n'en connaît aucun.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GeneticsDefinition {
    pub(crate) motifs: BTreeMap<String, MotifDefinition>,
    pub(crate) capabilities: Vec<CapabilityDefinition>,
    pub(crate) neural: NeuralDefinition,
    pub(crate) mutation: MutationDefinition,
    pub(crate) costs: CostDefinition,
}
