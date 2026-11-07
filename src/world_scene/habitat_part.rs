use fyrox::{
    core::{algebra::Vector3, pool::Handle},
    scene::mesh::Mesh,
};

/// Élément d'une ressource ou d'un substrat : sa maille et sa pose de repos.
pub(super) struct HabitatPart {
    node: Handle<Mesh>,
    position: Vector3<f32>,
    scale: Vector3<f32>,
}

impl HabitatPart {
    pub(super) fn new(node: Handle<Mesh>, position: Vector3<f32>, scale: Vector3<f32>) -> Self {
        Self {
            node,
            position,
            scale,
        }
    }

    pub(super) fn node(&self) -> Handle<Mesh> {
        self.node
    }

    pub(super) fn position(&self) -> Vector3<f32> {
        self.position
    }

    pub(super) fn scale(&self) -> Vector3<f32> {
        self.scale
    }
}
