use crate::definitions::habitat_placement::HabitatPlacement;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WorldDefinition {
    #[serde(default)]
    pub(crate) placements: Vec<HabitatPlacement>,
    pub(crate) size: usize,
    pub(crate) seed: u64,
    pub(crate) food_capacity: f32,
    pub(crate) food_regeneration_per_second: f32,
    pub(crate) food_attempts: usize,
    pub(crate) resource_spread_probability: f64,
    pub(crate) terrain_probability: f64,
    pub(crate) heat_sources: usize,
    pub(crate) heat_intensity: [f32; 2],
    pub(crate) oxygen_probability: f64,
    pub(crate) water_probability: f64,
    pub(crate) ambient_temperature: f32,
    pub(crate) water_decay_per_second: f32,
}

impl WorldDefinition {
    pub(crate) fn placements(&self) -> &Vec<HabitatPlacement> {
        &self.placements
    }
    pub(crate) fn size(&self) -> usize {
        self.size
    }
    pub(crate) fn seed(&self) -> u64 {
        self.seed
    }
    pub(crate) fn food_capacity(&self) -> f32 {
        self.food_capacity
    }
    pub(crate) fn food_regeneration_per_second(&self) -> f32 {
        self.food_regeneration_per_second
    }
    pub(crate) fn food_attempts(&self) -> usize {
        self.food_attempts
    }
    pub(crate) fn resource_spread_probability(&self) -> f64 {
        self.resource_spread_probability
    }
    pub(crate) fn terrain_probability(&self) -> f64 {
        self.terrain_probability
    }
    pub(crate) fn heat_sources(&self) -> usize {
        self.heat_sources
    }
    pub(crate) fn heat_intensity(&self) -> [f32; 2] {
        self.heat_intensity
    }
    pub(crate) fn oxygen_probability(&self) -> f64 {
        self.oxygen_probability
    }
    pub(crate) fn water_probability(&self) -> f64 {
        self.water_probability
    }
    pub(crate) fn ambient_temperature(&self) -> f32 {
        self.ambient_temperature
    }
    pub(crate) fn water_decay_per_second(&self) -> f32 {
        self.water_decay_per_second
    }
}
