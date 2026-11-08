use crate::genetics::Signal;
use serde::{Deserialize, Serialize};

/// Condition d'une contribution biologique, écrite dans le catalogue avec les noms des motifs.
/// Toutes les conditions d'une contribution doivent être vraies pour qu'elle s'applique.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) enum ConditionDefinition {
    /// Le motif apparaît au moins `at_least` fois dans le génome (et au plus `at_most`, s'il est
    /// donné : un répresseur). Les occurrences qui se chevauchent comptent chacune.
    Count {
        motif: String,
        at_least: u32,
        #[serde(default)]
        at_most: Option<u32>,
    },
    /// Tous ces motifs se trouvent dans une même séquence de 8 bits (une combinaison).
    SameSequence { motifs: Vec<String> },
    /// Une occurrence de `first` précède une occurrence de `then` le long du génome (l'ordre compte).
    Ordered { first: String, then: String },
    /// Au moins `copies` séquences complètes identiques portent le motif (une duplication).
    Repeated { motif: String, copies: u32 },
    /// Le motif apparaît dans les `sequences` premières séquences du génome (la position compte).
    Within { motif: String, sequences: u32 },
    /// Le milieu (ou l'état de l'organisme) a un signal compris entre `at_least` et `at_most` : la
    /// contribution ne s'exprime que dans ces conditions.
    Signal {
        signal: Signal,
        at_least: f32,
        #[serde(default)]
        at_most: Option<f32>,
    },
}
