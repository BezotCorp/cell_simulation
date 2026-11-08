use serde::{Deserialize, Serialize};

/// Dose : l'effet d'une contribution est multiplié par le nombre d'occurrences du motif, jusqu'à
/// `up_to` (un effet proportionnel au nombre de copies).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PerCopyDefinition {
    pub(crate) motif: String,
    pub(crate) up_to: u32,
}
