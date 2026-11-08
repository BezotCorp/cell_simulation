use crate::genetics::Signal;
use serde::{Deserialize, Serialize};

/// Condition sur un signal du milieu d'acclimatation de l'organisme.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SignalRange {
    pub(crate) signal: Signal,
    pub(crate) at_least: f32,
    #[serde(default)]
    pub(crate) at_most: Option<f32>,
}

impl SignalRange {
    pub(crate) fn holds(&self, value: f32) -> bool {
        value >= self.at_least && self.at_most.is_none_or(|most| value <= most)
    }
}
