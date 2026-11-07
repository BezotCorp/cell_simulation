use crate::persistence::legacy::legacy_neuron_connection::LegacyNeuronConnection;
use serde::Deserialize;

/// Réseau neuronal des anciens fichiers : une chaîne de connexions et un biais.
#[derive(Deserialize)]
pub(super) struct LegacyNeuron {
    pub(super) connections: Vec<LegacyNeuronConnection>,
    pub(super) bias: f32,
    #[allow(dead_code)]
    activation: f32,
    #[allow(dead_code)]
    complexity: u32,
}
