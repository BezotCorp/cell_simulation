use serde::{Deserialize, Serialize};

/// Grandeur que lit le réseau neuronal d'un organisme.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum Sensor {
    #[default]
    Temperature,
    Light,
    Food,
    Energy,
}

impl Sensor {
    pub(crate) const ALL: [Sensor; 4] = [Self::Temperature, Self::Light, Self::Food, Self::Energy];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Temperature => "température",
            Self::Light => "lumière",
            Self::Food => "nourriture",
            Self::Energy => "énergie",
        }
    }
}
