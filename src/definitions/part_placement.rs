use serde::{Deserialize, Serialize};

/// Où se placent les exemplaires d'un composant sur la membrane. L'avant de l'organisme est +Z.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) enum PartPlacement {
    /// Répartition régulière entre deux hauteurs (de -1, dessous, à 1, dessus).
    Spread { low: f32, high: f32 },
    /// Couronne équatoriale.
    Ring,
    /// Faisceau autour de l'axe avant (`front`) ou arrière, légèrement soulevé et écarté.
    Bundle { front: bool, lift: f32, around: f32 },
    /// Un seul exemplaire, dans cette direction.
    Fixed { x: f32, y: f32, z: f32 },
}
