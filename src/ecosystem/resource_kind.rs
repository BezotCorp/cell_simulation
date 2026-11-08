use serde::{Deserialize, Serialize};

/// Ressource ou élément de décor qu'une intervention peut ajouter au monde.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) enum ResourceKind {
    Food,
    HeatSource,
    Oxygen,
    Puddle,
    Iron,
    Calcium,
    Phosphorus,
}

impl ResourceKind {
    pub(crate) const ALL: [ResourceKind; 7] = [
        Self::Food,
        Self::HeatSource,
        Self::Oxygen,
        Self::Puddle,
        Self::Iron,
        Self::Calcium,
        Self::Phosphorus,
    ];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Food => "Nourriture",
            Self::HeatSource => "Source de chaleur",
            Self::Oxygen => "Oxygène",
            Self::Puddle => "Flaque d'eau",
            Self::Iron => "Fer",
            Self::Calcium => "Calcium",
            Self::Phosphorus => "Phosphore",
        }
    }
}
