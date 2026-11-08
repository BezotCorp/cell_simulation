use crate::definitions::{
    MaterialKind, PartAxis, PartCount, PartDriver, PartPlacement, PartShape, PartVariant, SignalRange,
};
use serde::{Deserialize, Serialize};

/// Un composant visible, entièrement décrit par des données : ce qui le développe, quand il apparaît,
/// combien d'exemplaires, où, sous quelle forme, et ses variations selon son développement.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PartDefinition {
    /// Nom affiché ; c'est aussi l'emplacement de `data/assets/library.ron`.
    pub(crate) name: String,
    pub(crate) driver: PartDriver,
    /// Niveau de développement à partir duquel le composant apparaît.
    #[serde(default = "default_appears_at")]
    pub(crate) appears_at: f32,
    /// Conditions sur le milieu d'acclimatation : toutes doivent être vraies.
    #[serde(default)]
    pub(crate) when: Vec<SignalRange>,
    pub(crate) count: PartCount,
    pub(crate) placement: PartPlacement,
    pub(crate) shape: PartShape,
    pub(crate) material: MaterialKind,
    /// Dimensions (dans le repère de la membrane, de rayon 1) à développement nul puis complet.
    pub(crate) size: [[f32; 3]; 2],
    pub(crate) axis: PartAxis,
    /// Distance du centre à laquelle se place le composant.
    pub(crate) reach: f32,
    /// Variations, de la moins à la plus développée. Le numéro de la dernière atteinte choisit aussi
    /// le modèle de la liste `models`.
    #[serde(default)]
    pub(crate) variants: Vec<PartVariant>,
    /// Modèles GLB utilisés tour à tour selon la variante atteinte.
    #[serde(default)]
    pub(crate) models: Vec<String>,
}

fn default_appears_at() -> f32 {
    0.08
}

impl PartDefinition {
    /// Dimensions interpolées pour un développement `level`.
    pub(crate) fn dimensions(&self, level: f32) -> [f32; 3] {
        let t = level.clamp(0.0, 1.0);
        std::array::from_fn(|i| self.size[0][i] + (self.size[1][i] - self.size[0][i]) * t)
    }

    /// Numéro de la dernière variante atteinte à ce niveau (0 sans variante atteinte).
    pub(crate) fn variant_at(&self, level: f32) -> usize {
        self.variants.iter().filter(|v| level >= v.from).count()
    }
}
