use crate::definitions::MaterialKind;
use crate::world_scene::{
    geometry::mesh,
    materials::Materials,
    surfaces::Surfaces,
};
use fyrox::{
    core::{
        algebra::{UnitQuaternion, Vector3},
        pool::Handle,
    },
    scene::{graph::Graph, mesh::Mesh},
};
use uuid::Uuid;

/// Marque de sélection : un anneau lumineux, autour de l'organisme choisi ou posé sur le site
/// choisi. Une seule instance existe pour toute la scène.
pub(super) struct SelectionMarker {
    ring: Handle<Mesh>,
}

impl SelectionMarker {
    pub(super) fn new(graph: &mut Graph, materials: &Materials, surfaces: &Surfaces) -> Self {
        let ring = mesh(
            graph,
            "Sélection de l'organisme",
            Uuid::now_v7(),
            Vector3::zeros(),
            Vector3::repeat(0.1),
            surfaces.ring(),
            materials.get(MaterialKind::Selection),
        );
        graph[ring].set_cast_shadows(false);
        let marker = Self { ring };
        marker.hide(graph);
        marker
    }

    pub(super) fn hide(&self, graph: &mut Graph) {
        graph[self.ring].set_visibility(false);
    }

    /// Anneau tournant autour d'un organisme de rayon `radius` centré en `position`.
    pub(super) fn show_organism(
        &self,
        graph: &mut Graph,
        position: Vector3<f32>,
        radius: f32,
        time: f32,
    ) {
        let size = radius.max(0.05) * (1.9 + 0.08 * (time * 3.0).sin());
        graph[self.ring].set_visibility(true);
        graph[self.ring]
            .local_transform_mut()
            .set_position(Vector3::new(position.x, 0.03, position.z))
            .set_rotation(UnitQuaternion::from_axis_angle(&Vector3::y_axis(), time * 1.2))
            .set_scale(Vector3::repeat(size));
    }

    /// Anneau posé au centre d'un site de la grille technique, sans bordure carrée.
    pub(super) fn show_site(&self, graph: &mut Graph, center: (f32, f32), spacing: f32, time: f32) {
        let size = 0.62 * spacing * (1.0 + 0.04 * (time * 3.0).sin());
        graph[self.ring].set_visibility(true);
        graph[self.ring]
            .local_transform_mut()
            .set_position(Vector3::new(center.0, 0.03, center.1))
            .set_rotation(UnitQuaternion::from_axis_angle(&Vector3::y_axis(), time * 0.6))
            .set_scale(Vector3::repeat(size));
    }
}
