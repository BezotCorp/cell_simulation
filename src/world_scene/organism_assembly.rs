use crate::biology::{Anatomy, AnatomyPart, PartKind};
use crate::definitions::MaterialKind;
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

/// Où se place le composant `index` d'une famille : l'avant de l'organisme est +Z, son arrière -Z.
fn direction(kind: PartKind, index: usize, count: usize, phase: f32) -> Vector3<f32> {
    let around = |spread: f32| {
        let angle = index as f32 * GOLDEN_ANGLE + phase;
        Vector3::new(angle.cos() * spread, 0.0, angle.sin() * spread)
    };
    match kind {
        PartKind::Flagellum => (Vector3::new(0.0, 0.0, -1.0) + around(0.18)).normalize(),
        PartKind::Ganglion => (Vector3::new(0.0, 0.35, 1.0) + around(0.2)).normalize(),
        PartKind::Cilia => {
            let angle = index as f32 / count.max(1) as f32 * std::f32::consts::TAU + phase;
            Vector3::new(angle.cos(), 0.1 * (index as f32 * 2.1).sin(), angle.sin()).normalize()
        }
        PartKind::Plate => spread(index, count, -0.35, 0.35, phase),
        PartKind::Chloroplast | PartKind::Thermophore => spread(index, count, 0.2, 1.0, phase),
        PartKind::Pad => spread(index, count, -1.0, -0.3, phase),
        PartKind::Spine | PartKind::Vacuole | PartKind::Nucleus => spread(index, count, -1.0, 1.0, phase),
    }
}

/// Forme, matériau, dimensions (dans le repère de la membrane, de rayon 1) et distance au centre
/// d'un composant procédural, d'après son développement `scale` de 0 à 1.
struct Shape {
    surface: fyrox::scene::mesh::surface::SurfaceResource,
    material: MaterialKind,
    size: Vector3<f32>,
    /// Axe local du modèle à aligner sur la direction du composant.
    axis: Vector3<f32>,
    reach: f32,
}

fn shape(kind: PartKind, scale: f32, surfaces: &Surfaces) -> Shape {
    let s = scale.clamp(0.0, 1.0);
    let (surface, material, size, axis, reach) = match kind {
        PartKind::Nucleus => (surfaces.sphere(), MaterialKind::Nucleus, Vector3::repeat(0.36), Vector3::y(), 0.78),
        PartKind::Vacuole => (surfaces.sphere(), MaterialKind::Water, Vector3::repeat(0.20 + 0.12 * s), Vector3::y(), 0.52),
        PartKind::Chloroplast => (surfaces.sphere(), MaterialKind::Food, Vector3::new(0.16, 0.08, 0.16) * (0.7 + 0.6 * s), Vector3::y(), 0.97),
        PartKind::Thermophore => (surfaces.crystal(), MaterialKind::Heat, Vector3::new(0.12, 0.30, 0.12) * (0.6 + 0.8 * s), Vector3::y(), 1.08),
        PartKind::Spine => (surfaces.crystal(), MaterialKind::Calcium, Vector3::new(0.08, 0.42 + 0.5 * s, 0.08), Vector3::y(), 1.15),
        PartKind::Flagellum => (geometry::cube(), MaterialKind::Calcium, Vector3::new(0.05, 0.05, 0.8 + 0.9 * s), Vector3::z(), 1.45),
        PartKind::Cilia => (geometry::cube(), MaterialKind::Oxygen, Vector3::new(0.04, 0.04, 0.26), Vector3::z(), 1.12),
        PartKind::Plate => (surfaces.sphere(), MaterialKind::Rock, Vector3::new(0.27, 0.07, 0.27) * (0.7 + 0.5 * s), Vector3::y(), 1.0),
        PartKind::Ganglion => (surfaces.sphere(), MaterialKind::Nucleus, Vector3::repeat(0.17 + 0.14 * s), Vector3::y(), 0.88),
        PartKind::Pad => (surfaces.sphere(), MaterialKind::Phosphorus, Vector3::new(0.22, 0.06, 0.22) * (0.7 + 0.5 * s), Vector3::y(), 1.0),
    };
    Shape {
        surface,
        material,
        size,
        axis,
        reach,
    }
}

/// Place un composant sur la membrane : un modèle de la bibliothèque si l'emplacement en a un de
/// chargé (modèles orientés selon +Y, de taille unitaire), sinon la forme procédurale. Le résultat
/// est lié à la membrane, donc il suit sa déformation et ses mouvements.
#[allow(clippy::too_many_arguments)]
fn place(
    scene: &mut Scene,
    membrane: Handle<Mesh>,
    part: &AnatomyPart,
    direction: Vector3<f32>,
    library: &ComponentLibrary,
    surfaces: &Surfaces,
    materials: &Materials,
) -> Handle<Node> {
    let form = shape(part.kind(), part.scale(), surfaces);
    let (node, rotation, size) = if let Some(model) = library.model(part.kind().slot(), part.variant()) {
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
            part.kind().label(),
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
    let centre = direction * form.reach;
    scene.graph[node]
        .local_transform_mut()
        .set_position(centre)
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
    // Noyau : tout organisme en a un, un peu décalé vers l'avant.
    let nucleus = AnatomyPart::new_nucleus();
    nodes.push(place(scene, membrane, &nucleus, Vector3::new(0.2, 0.62, 0.75).normalize(), library, surfaces, materials));

    let mut parts: Vec<&AnatomyPart> = anatomy.parts().iter().collect();
    parts.sort_by(|a, b| b.scale().total_cmp(&a.scale()).then(a.kind().cmp(&b.kind())));
    for part in parts.into_iter().take(detail.parts()) {
        let phase = unit(bits ^ part.kind() as u64) * std::f32::consts::TAU;
        let count = usize::from(part.count());
        for index in 0..count {
            if nodes.len() >= MAX_PARTS {
                return nodes;
            }
            nodes.push(place(
                scene,
                membrane,
                part,
                direction(part.kind(), index, count, phase),
                library,
                surfaces,
                materials,
            ));
        }
    }
    nodes
}
