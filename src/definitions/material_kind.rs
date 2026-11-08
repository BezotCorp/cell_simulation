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

impl MaterialKind {
    pub(crate) const ALL: [MaterialKind; 17] = [Self::Plinth, Self::Rim, Self::Ground, Self::Soil, Self::Sand, Self::Rock, Self::Water, Self::Food, Self::Heat, Self::Oxygen, Self::Iron, Self::Calcium, Self::Phosphorus, Self::Healthy, Self::Starving, Self::Nucleus, Self::Selection];

    pub(crate) fn label(self) -> String {
        format!("{self:?}")
    }
}
