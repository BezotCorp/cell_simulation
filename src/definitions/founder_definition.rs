use serde::{Deserialize, Serialize};

/// Un groupe de cellules présentes au départ de l'expérience.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FounderDefinition {
    pub(crate) count: u32,
    /// Réserve initiale, en part de la capacité de chaque cellule.
    #[serde(default = "full")]
    pub(crate) energy: f32,
    /// Génome complet (séquences de 8 bits en texte) ; à défaut, celui de l'ancêtre.
    #[serde(default)]
    pub(crate) sequences: Option<Vec<String>>,
    /// Motifs ajoutés au génome : une séquence qui porte chacun est ajoutée à la fin.
    #[serde(default)]
    pub(crate) add_motifs: Vec<String>,
    /// Réparties au hasard sur les sites accessibles plutôt que groupées au point de départ.
    #[serde(default)]
    pub(crate) scatter: bool,
}

fn full() -> f32 {
    1.0
}

impl Default for FounderDefinition {
    fn default() -> Self {
        Self { count: 1, energy: 1.0, sequences: None, add_motifs: Vec::new(), scatter: false }
    }
}
