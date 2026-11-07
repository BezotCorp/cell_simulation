use fyrox::{
    core::{algebra::Vector3, pool::Handle},
    material::MaterialResource,
    scene::{
        base::{BaseBuilder, SceneNodeId},
        graph::Graph,
        mesh::{
            Mesh, MeshBuilder,
            surface::{self, SurfaceBuilder},
        },
        transform::TransformBuilder,
    },
};
use uuid::Uuid;
pub(crate) fn mesh(
    graph: &mut Graph,
    name: &str,
    id: Uuid,
    position: Vector3<f32>,
    scale: Vector3<f32>,
    sphere: bool,
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
    .with_surfaces(vec![
        SurfaceBuilder::new(if sphere {
            surface::SPHERE.resource.clone()
        } else {
            surface::CUBE.resource.clone()
        })
        .with_material(material)
        .build(),
    ])
    .build(graph)
}
