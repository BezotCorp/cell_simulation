use serde::{Deserialize, Serialize};

/// Sens dans lequel un motif est lu dans une séquence de 8 bits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum Orientation {
    /// Du bit de poids fort au bit de poids faible, comme le motif est écrit.
    #[default]
    Forward,
    /// En sens inverse : la fenêtre de bits est lue de droite à gauche.
    Reverse,
    /// Dans l'un ou l'autre sens.
    Both,
}

impl Orientation {
    pub(crate) const ALL: [Orientation; 3] = [Self::Forward, Self::Reverse, Self::Both];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Forward => "sens direct",
            Self::Reverse => "sens inverse",
            Self::Both => "les deux sens",
        }
    }
}
