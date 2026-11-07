use crate::world_scene::geometry::mesh;
use crate::world_scene::habitat_visual::TileState;
use crate::world_scene::materials::{Gradient, Materials};
use crate::{
    definitions::MaterialKind,
    habitat::{Entity, Mineral, Terrain},
    persistence::ViewMode,
};
use fyrox::{
    core::{
        algebra::{UnitQuaternion, Vector3},
        pool::Handle,
    },
    material::MaterialResource,
    scene::{graph::Graph, mesh::Mesh},
};
use uuid::Uuid;

/// Dalle de sol d'un habitat. Elle ne projette pas d'ombre : elle est toujours plane.
pub(crate) fn tile(
    graph: &mut Graph,
    id: Uuid,
    x: f32,
    z: f32,
    spacing: f32,
    materials: &Materials,
) -> Handle<Mesh> {
    let tile = mesh(
        graph,
        "Habitat",
        id,
        Vector3::new(x, -0.08, z),
        Vector3::new(0.95 * spacing, 0.16, 0.95 * spacing),
        false,
        materials.get(MaterialKind::Ground),
    );
    graph[tile].set_cast_shadows(false);
    tile
}

/// Ressource ou substrat posé sur un habitat non vide.
pub(crate) fn object(
    graph: &mut Graph,
    entity: &Entity,
    x: f32,
    z: f32,
    spacing: f32,
    materials: &Materials,
) -> Handle<Mesh> {
    let (height, width, sphere) = match entity {
        Entity::Obstacle(Terrain::Rock) => (0.65, 0.72, false),
        Entity::Obstacle(Terrain::Wall) => (0.85, 0.85, false),
        Entity::HeatSource(_) => (0.27, 0.38, true),
        Entity::Food(_) => (0.08, 0.20, true),
        Entity::Oxygen(_) => (0.18, 0.11, true),
        Entity::Water(_) => (0.02, 0.82, false),
        Entity::Obstacle(_) => (0.02, 0.88, false),
        Entity::Minerals(_) => (0.17, 0.2, false),
        Entity::Empty => (0.0, 0.01, false),
    };
    let object = mesh(
        graph,
        "Ressource / substrat",
        Uuid::now_v7(),
        Vector3::new(x, height / 2.0, z),
        Vector3::new(width * spacing, height.max(0.01), width * spacing),
        sphere,
        materials.get(kind(entity)),
    );
    if matches!(entity, Entity::Obstacle(Terrain::Rock)) {
        graph[object]
            .local_transform_mut()
            .set_rotation(UnitQuaternion::from_euler_angles(0.12, (x + z) * 0.4, 0.08));
    }
    object
}

pub(crate) fn kind(entity: &Entity) -> MaterialKind {
    match entity {
        Entity::Empty => MaterialKind::Ground,
        Entity::Food(_) => MaterialKind::Food,
        Entity::HeatSource(_) => MaterialKind::Heat,
        Entity::Obstacle(Terrain::Rock | Terrain::Wall) => MaterialKind::Rock,
        Entity::Obstacle(Terrain::Sand) => MaterialKind::Sand,
        Entity::Obstacle(Terrain::Soil) => MaterialKind::Soil,
        Entity::Water(_) => MaterialKind::Water,
        Entity::Oxygen(_) => MaterialKind::Oxygen,
        Entity::Minerals(Mineral::Iron) => MaterialKind::Iron,
        Entity::Minerals(Mineral::Calcium) => MaterialKind::Calcium,
        Entity::Minerals(Mineral::Phosphorus) => MaterialKind::Phosphorus,
    }
}

/// Apparence de la dalle : température en vue thermique, sinon eau ou sol.
pub(crate) fn tile_state(mode: ViewMode, temperature: f32, water: bool) -> TileState {
    match mode {
        ViewMode::Temperature => {
            TileState::Temperature(Materials::band((temperature - 15.0) / 45.0))
        }
        _ if water => TileState::Water,
        _ => TileState::Ground,
    }
}

pub(crate) fn tile_material(state: TileState, materials: &Materials) -> MaterialResource {
    match state {
        TileState::Temperature(band) => materials.gradient(Gradient::Temperature, band),
        TileState::Water => materials.get(MaterialKind::Water),
        TileState::Ground => materials.get(MaterialKind::Ground),
    }
}
