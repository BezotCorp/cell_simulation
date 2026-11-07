use fyrox::{
    core::{algebra::Vector3, pool::Handle},
    material::MaterialResource,
    scene::{
        base::{BaseBuilder, SceneNodeId},
        graph::Graph,
        mesh::{
            Mesh, MeshBuilder,
            surface::{SurfaceBuilder, SurfaceResource},
        },
        transform::TransformBuilder,
    },
};
use uuid::Uuid;

/// Maille unitaire du moteur, partagée par toutes les instances.
pub(super) fn cube() -> SurfaceResource {
    fyrox::scene::mesh::surface::CUBE.resource.clone()
}

pub(super) fn mesh(
    graph: &mut Graph,
    name: &str,
    id: Uuid,
    position: Vector3<f32>,
    scale: Vector3<f32>,
    surface: SurfaceResource,
    material: MaterialResource,
) -> Handle<Mesh> {
    MeshBuilder::new(
        BaseBuilder::new()
            .with_name(name)
            .with_instance_id(SceneNodeId(id))
            .with_local_transform(
                TransformBuilder::new()
                    .with_local_position(position)
                    .with_local_scale(scale)
                    .build(),
            ),
    )
    .with_surfaces(vec![SurfaceBuilder::new(surface).with_material(material).build()])
    .build(graph)
}
