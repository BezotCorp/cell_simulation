use crate::genetics::Signal;
use serde::{Deserialize, Serialize};

/// Valeurs des signaux perçus par un organisme à un instant donné, chacune dans `[0, 1]`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct Signals {
    values: [f32; Signal::COUNT],
}

impl Signals {
    pub(crate) fn new(values: [f32; Signal::COUNT]) -> Self {
        Self {
            values: values.map(|value| if value.is_finite() { value.clamp(0.0, 1.0) } else { 0.0 }),
        }
    }

    pub(crate) fn get(&self, signal: Signal) -> f32 {
        self.values[signal.index()]
    }

    pub(crate) fn values(&self) -> &[f32; Signal::COUNT] {
        &self.values
    }

    pub(crate) fn is_valid(&self) -> bool {
        self.values.iter().all(|value| (0.0..=1.0).contains(value))
    }
}
