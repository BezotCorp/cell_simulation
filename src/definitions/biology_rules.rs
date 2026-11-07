use serde::{Deserialize, Serialize};
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BiologyRules {
    energy_capacity: f32,
    maturity_seconds: u64,
    basal_metabolism: f32,
    movement_seconds: f32,
    movement_energy_seconds: f32,
    movement_reserve: f32,
}

impl BiologyRules {
    pub(crate) fn energy_capacity(&self) -> f32 {
        self.energy_capacity
    }
    pub(crate) fn maturity_seconds(&self) -> u64 {
        self.maturity_seconds
    }
    pub(crate) fn basal_metabolism(&self) -> f32 {
        self.basal_metabolism
    }
    pub(crate) fn movement_seconds(&self) -> f32 {
        self.movement_seconds
    }
    pub(crate) fn movement_energy_seconds(&self) -> f32 {
        self.movement_energy_seconds
    }
    pub(crate) fn movement_reserve(&self) -> f32 {
        self.movement_reserve
    }
}
