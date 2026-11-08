use crate::genetics::Sensor;
use serde::{Deserialize, Serialize};

/// Réseau neuronal : gain de la première unité, biais, et motifs qui choisissent le capteur lu
/// (le premier motif présent le long du génome l'emporte, sinon la température).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NeuralDefinition {
    pub(crate) gain: f32,
    pub(crate) bias: f32,
    #[serde(default)]
    pub(crate) sensors: Vec<(String, Sensor)>,
}
