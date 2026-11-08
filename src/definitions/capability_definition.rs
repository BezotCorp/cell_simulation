use crate::definitions::ContributionDefinition;
use crate::genetics::Product;
use serde::{Deserialize, Serialize};

/// Règles d'expression d'un produit : la somme de ses contributions (bornée à `[0, 1]`) donne sa
/// quantité dans l'organisme.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CapabilityDefinition {
    pub(crate) product: Product,
    pub(crate) contributions: Vec<ContributionDefinition>,
}
