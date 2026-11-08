use crate::definitions::{MaterialDefinition, MaterialKind};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Appearance {
    pub(crate) title: String,
    pub(crate) cell_radius: f32,
    pub(crate) habitat_spacing: f32,
    pub(crate) materials: BTreeMap<MaterialKind, MaterialDefinition>,
}

impl Appearance {
    pub(crate) fn title(&self) -> &str {
        &self.title
    }
    pub(crate) fn cell_radius(&self) -> f32 {
        self.cell_radius
    }
    pub(crate) fn habitat_spacing(&self) -> f32 {
        self.habitat_spacing
    }
    pub(crate) fn materials(&self) -> &BTreeMap<MaterialKind, MaterialDefinition> {
        &self.materials
    }
}
