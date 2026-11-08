use serde::{Deserialize, Serialize};

/// Quelles cellules supprimer lors d'une intervention.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) enum KillTarget {
    Everyone,
    /// Part de la population, de 0 à 1, tirée au hasard.
    Fraction(f32),
    /// Un nombre de cellules tirées au hasard.
    Count(u32),
    /// Les cellules les moins énergétiques.
    Weakest(u32),
    /// Toutes les cellules à moins de `radius` sites de `(x, y)`.
    Around { x: usize, y: usize, radius: f32 },
}
