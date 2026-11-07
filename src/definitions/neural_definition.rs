use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NeuralDefinition {
    temperature_weight: f32,
    bias: f32,
}

impl NeuralDefinition {
    pub(crate) fn temperature_weight(&self) -> f32 {
        self.temperature_weight
    }
    pub(crate) fn bias(&self) -> f32 {
        self.bias
    }
}
