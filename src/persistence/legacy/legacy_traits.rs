use serde::Deserialize;

/// Mécanismes des fichiers de version 6 : six expressions (ingestion, photosynthèse,
/// thermosynthèse, prédation, motilité, adhésion) et le capteur du réseau.
#[derive(Deserialize)]
pub(crate) struct LegacyTraits {
    pub(super) values: [f32; 6],
    pub(super) sensor: u8,
}

impl LegacyTraits {
    /// Mécanismes de l'ancêtre, pour les fichiers qui n'en portent pas.
    pub(super) fn founder() -> Self {
        Self {
            values: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
            sensor: 0,
        }
    }
}
