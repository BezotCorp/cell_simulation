use serde::{Deserialize, Serialize};

/// Nombre d'exemplaires d'un composant : `⌈niveau × per_level × max(taille, 0,5)⌉`, entre 1 et `max`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PartCount {
    pub(crate) per_level: f32,
    pub(crate) max: u8,
}
