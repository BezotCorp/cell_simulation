use crate::world_scene::{
    geometry::{self, mesh},
    habitat_animation::HabitatAnimation,
    habitat_part::HabitatPart,
    habitat_visual::HabitatVisual,
    materials::Materials,
    noise::cell_value,
    surfaces::Surfaces,
};
use crate::{
    definitions::MaterialKind,
    habitat::{Entity, Mineral, Terrain},
};
use fyrox::{
    core::algebra::{UnitQuaternion, Vector3},
    scene::graph::Graph,
};
use uuid::Uuid;

fn mineral_material(mineral: &Mineral) -> MaterialKind {
    match mineral {
        Mineral::Iron => MaterialKind::Iron,
        Mineral::Calcium => MaterialKind::Calcium,
        Mineral::Phosphorus => MaterialKind::Phosphorus,
    }
}

/// Construit l'habillage d'un habitat. Le sable, la terre et les habitats vides ne sont que
/// des couleurs du sol : ils n'ont aucun objet. `(x, z)` est le centre de l'habitat et
/// `cell` ses indices, qui fixent des variations stables d'un lancement à l'autre. Quand
/// `detailed` est faux (très nombreux habitats garnis), chaque ressource se réduit à un seul
/// élément sans ombre : le dessin se simplifie, jamais le contenu du monde.
pub(super) fn build(
    graph: &mut Graph,
    entity: &Entity,
    center: (f32, f32),
    cell: (usize, usize),
    spacing: f32,
    materials: &Materials,
    surfaces: &Surfaces,
    detailed: bool,
) -> Option<HabitatVisual> {
    let (x, z) = center;
    let seed = (cell.0 * 7919 + cell.1 * 104_729) as u32;
    let random = |salt: i32| cell_value(cell.0 as i32 * 31 + salt, cell.1 as i32 * 17 - salt, seed);
    let mut parts = Vec::new();
    let mut add = |graph: &mut Graph,
                   name: &str,
                   offset: Vector3<f32>,
                   scale: Vector3<f32>,
                   surface,
                   material,
                   rotation: Option<UnitQuaternion<f32>>| {
        let position = Vector3::new(x, 0.0, z) + offset;
        let node = mesh(graph, name, Uuid::now_v7(), position, scale, surface, material);
        graph[node].set_cast_shadows(detailed);
        if let Some(rotation) = rotation {
            graph[node].local_transform_mut().set_rotation(rotation);
        }
        parts.push(HabitatPart::new(node, position, scale));
    };
    let animation = match entity {
        Entity::Empty | Entity::Obstacle(Terrain::Sand | Terrain::Soil) => return None,
        Entity::Obstacle(Terrain::Wall) => {
            add(
                graph,
                "Mur",
                Vector3::new(0.0, 0.42, 0.0),
                Vector3::new(0.9 * spacing, 0.84, 0.9 * spacing),
                geometry::cube(),
                materials.get(MaterialKind::Rock),
                None,
            );
            HabitatAnimation::Still
        }
        Entity::Obstacle(Terrain::Rock) => {
            // Amas de trois blocs de tailles et d'orientations différentes.
            for index in 0..if detailed { 3 } else { 1 } {
                let angle = random(index) * std::f32::consts::TAU;
                let distance = if index == 0 { 0.0 } else { 0.22 * spacing };
                let width = (0.30 + 0.16 * random(10 + index)) * spacing;
                let height = (0.22 + 0.34 * random(20 + index)) * if index == 0 { 1.5 } else { 1.0 };
                add(
                    graph,
                    "Roche",
                    Vector3::new(angle.cos() * distance, height * 0.55, angle.sin() * distance),
                    Vector3::new(width, height, width * (0.8 + 0.4 * random(30 + index))),
                    surfaces.rock(seed as usize + index as usize),
                    materials.get(MaterialKind::Rock),
                    Some(UnitQuaternion::from_euler_angles(
                        random(40 + index) * 0.5,
                        random(50 + index) * std::f32::consts::TAU,
                        random(60 + index) * 0.4,
                    )),
                );
            }
            HabitatAnimation::Still
        }
        Entity::HeatSource(_) => {
            add(
                graph,
                "Source thermique",
                Vector3::new(0.0, 0.05, 0.0),
                Vector3::new(0.36 * spacing, 0.22, 0.36 * spacing),
                surfaces.rock(seed as usize),
                materials.get(MaterialKind::Heat),
                None,
            );
            if detailed {
                add(
                    graph,
                    "Cœur de chaleur",
                    Vector3::new(0.0, 0.17, 0.0),
                    Vector3::new(0.16 * spacing, 0.13, 0.16 * spacing),
                    surfaces.sphere(),
                    materials.get(MaterialKind::Heat),
                    None,
                );
            }
            HabitatAnimation::Pulse
        }
        Entity::Food(_) => {
            for index in 0..if detailed { 3 } else { 1 } {
                let angle = (index as f32 + random(1)) * std::f32::consts::TAU / 3.0;
                let radius = (0.075 + 0.04 * random(2 + index)) * spacing;
                add(
                    graph,
                    "Baie",
                    Vector3::new(angle.cos() * 0.11 * spacing, radius, angle.sin() * 0.11 * spacing),
                    Vector3::repeat(radius),
                    surfaces.sphere(),
                    materials.get(MaterialKind::Food),
                    None,
                );
            }
            HabitatAnimation::Still
        }
        Entity::Oxygen(_) => {
            for index in 0..if detailed { 3 } else { 1 } {
                let angle = (index as f32 + random(3)) * std::f32::consts::TAU / 3.0;
                let radius = (0.045 + 0.035 * random(4 + index)) * spacing;
                add(
                    graph,
                    "Bulle d'oxygène",
                    Vector3::new(
                        angle.cos() * 0.15 * spacing,
                        0.16 + 0.1 * index as f32,
                        angle.sin() * 0.15 * spacing,
                    ),
                    Vector3::repeat(radius),
                    surfaces.sphere(),
                    materials.get(MaterialKind::Oxygen),
                    None,
                );
            }
            HabitatAnimation::Bob
        }
        // L'eau n'est plus un décor fixe : la surcouche hydrologique la dessine là où elle se trouve.
        Entity::Water(_) => HabitatAnimation::Still,
        Entity::Minerals(mineral) => {
            for index in 0..if detailed { 3 } else { 1 } {
                let angle = (index as f32 + random(5)) * std::f32::consts::TAU / 3.0;
                let height = 0.16 + 0.12 * random(6 + index);
                add(
                    graph,
                    "Cristal",
                    Vector3::new(angle.cos() * 0.12 * spacing, height * 0.35, angle.sin() * 0.12 * spacing),
                    Vector3::new(0.09 * spacing, height, 0.09 * spacing),
                    surfaces.crystal(),
                    materials.get(mineral_material(mineral)),
                    Some(UnitQuaternion::from_euler_angles(
                        0.25 * (random(7 + index) - 0.5),
                        random(8 + index) * std::f32::consts::TAU,
                        0.35 * (random(9 + index) - 0.5),
                    )),
                );
            }
            HabitatAnimation::Still
        }
    };
    Some(HabitatVisual::new(parts, animation, random(99) * std::f32::consts::TAU))
}
