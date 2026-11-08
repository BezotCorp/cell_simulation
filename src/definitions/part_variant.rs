use crate::definitions::MaterialKind;
use serde::{Deserialize, Serialize};

/// Variation d'un composant à partir d'un niveau de développement : remplace son matériau, agrandit
/// ou réduit ses dimensions, et choisit un autre modèle 3D.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PartVariant {
    pub(crate) from: f32,
    #[serde(default)]
    pub(crate) material: Option<MaterialKind>,
    #[serde(default = "one")]
    pub(crate) scale: f32,
    /// Modèle GLB (relatif à `data/assets/`) qui remplace la forme procédurale.
    #[serde(default)]
    pub(crate) model: Option<String>,
}

fn one() -> f32 {
    1.0
}
