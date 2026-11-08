use serde::{Deserialize, Serialize};

/// Axe local d'une forme procédurale qui se dirige vers l'extérieur du corps.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum PartAxis {
    Y,
    Z,
}
