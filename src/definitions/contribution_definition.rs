use crate::definitions::{ConditionDefinition, PerCopyDefinition};
use serde::{Deserialize, Serialize};

/// Ajout à la quantité d'un produit lorsque toutes les conditions sont réunies. Une capacité a
/// plusieurs contributions : une forme primitive (un motif courant), puis des formes plus
/// développées (copies supplémentaires, motifs ordonnés, combinaisons).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ContributionDefinition {
    #[serde(default)]
    pub(crate) when: Vec<ConditionDefinition>,
    pub(crate) amount: f32,
    #[serde(default)]
    pub(crate) per_copy: Option<PerCopyDefinition>,
}
