use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub(crate) enum MaterialKind {
    Plinth,
    Rim,
    Ground,
    Soil,
    Sand,
    Rock,
    Water,
    Food,
    Heat,
    Oxygen,
    Iron,
    Calcium,
    Phosphorus,
    Healthy,
    Starving,
    Nucleus,
    Selection,
}
