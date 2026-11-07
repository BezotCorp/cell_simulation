use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HabitatPlacement {
    x: usize,
    y: usize,
    entity: crate::habitat::Entity,
}

impl HabitatPlacement {
    pub(crate) fn x(&self) -> usize {
        self.x
    }
    pub(crate) fn y(&self) -> usize {
        self.y
    }
    pub(crate) fn entity(&self) -> &crate::habitat::Entity {
        &self.entity
    }
}
