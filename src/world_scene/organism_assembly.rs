use crate::biology::{Anatomy, AnatomyPart};
use crate::definitions::{MaterialKind, PartAxis, PartDefinition, PartPlacement, PartShape};
use crate::world_scene::{
    component_library::ComponentLibrary,
    detail_level::DetailLevel,
    geometry::{self, mesh},
    materials::Materials,
    surfaces::Surfaces,
};
use fyrox::{
    core::{
        algebra::{UnitQuaternion, Vector3},
        pool::Handle,
    },
    graph::SceneGraph,
    resource::model::ModelResourceExtension,
    scene::{Scene, mesh::Mesh, node::Node},
};
use uuid::Uuid;

/// Plus grand nombre de composants dessinés pour un organisme, tous types confondus.
const MAX_PARTS: usize = 40;
/// Angle d'or : répartition régulière de points sur une sphère.
const GOLDEN_ANGLE: f32 = 2.399_963;

fn unit(seed: u64) -> f32 {
    let mut z = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xBF58_476D_1CE4_E5B9;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    ((z ^ (z >> 31)) >> 40) as f32 / (1u64 << 24) as f32
}

/// Point `index` sur `count` d'une répartition régulière de la sphère unité entre les hauteurs
/// `low` et `high` (de -1 à 1), tourné d'une phase propre à l'organisme.
fn spread(index: usize, count: usize, low: f32, high: f32, phase: f32) -> Vector3<f32> {
    let y = low + (high - low) * (index as f32 + 0.5) / count.max(1) as f32;
    let ring = (1.0 - y * y).max(0.0).sqrt();
    let angle = index as f32 * GOLDEN_ANGLE + phase;
    Vector3::new(angle.cos() * ring, y, angle.sin() * ring)
}

/// Où se place le composant `index` d'une famille décrite par `placement`.
fn direction(placement: PartPlacement, index: usize, count: usize, phase: f32) -> Vector3<f32> {
    match placement {
        PartPlacement::Spread { low, high } => spread(index, count, low, high, phase),
        PartPlacement::Ring => {
            let angle = index as f32 / count.max(1) as f32 * std::f32::consts::TAU + phase;
            Vector3::new(angle.cos(), 0.1 * (index as f32 * 2.1).sin(), angle.sin()).normalize()
        }
        PartPlacement::Bundle { front, lift, around } => {
            let angle = index as f32 * GOLDEN_ANGLE + phase;
            let axis = Vector3::new(0.0, lift, if front { 1.0 } else { -1.0 });
            (axis + Vector3::new(angle.cos() * around, 0.0, angle.sin() * around)).normalize()
        }
        PartPlacement::Fixed { x, y, z } => Vector3::new(x, y, z).try_normalize(1e-6).unwrap_or_else(Vector3::y),
    }
}

/// Forme procédurale d'un composant : surface, matériau, dimensions et axe local à aligner sur la
/// direction du composant. Le matériau et l'échelle peuvent être remplacés par la variante atteinte.
struct Shape {
    surface: fyrox::scene::mesh::surface::SurfaceResource,
    material: MaterialKind,
    size: Vector3<f32>,
    axis: Vector3<f32>,
}

fn shape(definition: &PartDefinition, part: &AnatomyPart, surfaces: &Surfaces) -> Shape {
    let variant = part.variant().checked_sub(1).and_then(|index| definition.variants.get(index));
    let scale = variant.map_or(1.0, |variant| variant.scale);
    Shape {
        surface: match definition.shape {
            PartShape::Sphere => surfaces.sphere(),
            PartShape::Crystal => surfaces.crystal(),
            PartShape::Cube => geometry::cube(),
        },
        material: variant.and_then(|variant| variant.material).unwrap_or(definition.material),
        size: Vector3::from(definition.dimensions(part.level())) * scale,
        axis: match definition.axis {
            PartAxis::Y => Vector3::y(),
            PartAxis::Z => Vector3::z(),
        },
    }
}

/// Modèle 3D d'un composant : celui de sa variante, sinon l'un de ses modèles, sinon l'emplacement
/// de la bibliothèque qui porte son nom.
fn model(
    definition: &PartDefinition,
    part: &AnatomyPart,
    library: &ComponentLibrary,
) -> Option<fyrox::resource::model::ModelResource> {
    let chosen = part
        .variant()
        .checked_sub(1)
        .and_then(|index| definition.variants.get(index))
        .and_then(|variant| variant.model.as_deref())
        .or_else(|| {
            let count = definition.models.len();
            (count > 0).then(|| definition.models[part.variant() % count].as_str())
        });
    match chosen {
        Some(file) => library.file(file),
        None => library.model(&definition.name, part.variant() as u8).cloned(),
    }
}

/// Place un composant sur la membrane : un modèle de la bibliothèque s'il est chargé (modèles
/// orientés selon +Y, de taille unitaire), sinon la forme procédurale. Le résultat est lié à la
/// membrane, donc il suit sa déformation et ses mouvements.
#[allow(clippy::too_many_arguments)]
fn place(
    scene: &mut Scene,
    membrane: Handle<Mesh>,
    definition: &PartDefinition,
    part: &AnatomyPart,
    direction: Vector3<f32>,
    library: &ComponentLibrary,
    surfaces: &Surfaces,
    materials: &Materials,
) -> Handle<Node> {
    let form = shape(definition, part, surfaces);
    let (node, rotation, size) = if let Some(model) = model(definition, part, library) {
        let node = model.instantiate(scene);
        let length = form.size.max();
        (
            node,
            UnitQuaternion::rotation_between(&Vector3::y(), &direction).unwrap_or_else(UnitQuaternion::identity),
            Vector3::repeat(length),
        )
    } else {
        let node: Handle<Node> = mesh(
            &mut scene.graph,
            &definition.name,
            Uuid::now_v7(),
            Vector3::zeros(),
            form.size,
            form.surface.clone(),
            materials.get(form.material),
        )
        .transmute();
        (
            node,
            UnitQuaternion::rotation_between(&form.axis, &direction).unwrap_or_else(UnitQuaternion::identity),
            form.size,
        )
    };
    scene.graph[node]
        .local_transform_mut()
        .set_position(direction * definition.reach)
        .set_rotation(rotation)
        .set_scale(size);
    if let Ok(mesh) = scene.graph.try_get_mut_of_type::<Mesh>(node.transmute()) {
        mesh.set_cast_shadows(false);
    }
    scene.graph.link_nodes(node, membrane);
    node
}

/// Assemble l'anatomie d'un organisme sur sa membrane. Les familles les plus développées passent
/// en premier : une finesse réduite ne garde que l'essentiel. Renvoie les nœuds créés.
#[allow(clippy::too_many_arguments)]
pub(super) fn assemble(
    scene: &mut Scene,
    membrane: Handle<Mesh>,
    id: Uuid,
    anatomy: &Anatomy,
    definitions: &[PartDefinition],
    detail: DetailLevel,
    library: &ComponentLibrary,
    surfaces: &Surfaces,
    materials: &Materials,
) -> Vec<Handle<Node>> {
    let mut nodes = Vec::new();
    if detail == DetailLevel::Bare {
        return nodes;
    }
    let bits = id.as_u128() as u64;
    let mut parts: Vec<&AnatomyPart> = anatomy.parts().iter().collect();
    parts.sort_by(|a, b| b.level().total_cmp(&a.level()).then(a.part().cmp(&b.part())));
    for part in parts.into_iter().take(detail.parts()) {
        let Some(definition) = definitions.get(part.part()) else {
            continue;
        };
        let phase = unit(bits ^ part.part() as u64) * std::f32::consts::TAU;
        let count = usize::from(part.count());
        for index in 0..count {
            if nodes.len() >= MAX_PARTS {
                return nodes;
            }
            nodes.push(place(
                scene,
                membrane,
                definition,
                part,
                direction(definition.placement, index, count, phase),
                library,
                surfaces,
                materials,
            ));
        }
    }
    nodes
}
