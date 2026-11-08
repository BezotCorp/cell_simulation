use serde::{Deserialize, Serialize};

/// Forme géométrique de base d'un composant procédural.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum PartShape {
    Sphere,
    Crystal,
    Cube,
}
