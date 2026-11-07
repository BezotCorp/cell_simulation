use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MaterialDefinition {
    id: Uuid,
    color: [u8; 4],
    roughness: f32,
    metallic: f32,
    emission: f32,
}

impl MaterialDefinition {
    pub(crate) fn id(&self) -> Uuid {
        self.id
    }
    pub(crate) fn color(&self) -> [u8; 4] {
        self.color
    }
    pub(crate) fn roughness(&self) -> f32 {
        self.roughness
    }
    pub(crate) fn metallic(&self) -> f32 {
        self.metallic
    }
    pub(crate) fn emission(&self) -> f32 {
        self.emission
    }
}
