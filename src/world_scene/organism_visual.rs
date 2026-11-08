use crate::world_scene::{
    component_library::ComponentLibrary, detail_level::DetailLevel, geometry::mesh,
    materials::Materials, organism_assembly, surfaces::Surfaces,
};
use crate::{biology::Cell, definitions::MaterialKind};
use fyrox::graph::SceneGraph;
use fyrox::{
    scene::Scene,
    core::{
        algebra::{UnitQuaternion, Vector3},
        pool::Handle,
    },
    scene::{
        base::{BaseBuilder, SceneNodeId},
        graph::Graph,
        mesh::Mesh,
        node::Node,
        pivot::PivotBuilder,
        transform::TransformBuilder,
    },
};
use uuid::Uuid;

/// Durée caractéristique du lissage des déplacements, en secondes réelles : indépendante
/// de la cadence d'affichage comme de la vitesse de la simulation.
const SMOOTHING_SECONDS: f32 = 0.09;
/// Seuil sous lequel une position ou une taille est considérée atteinte.
const SETTLED: f32 = 1e-4;
/// Durée du rétrécissement d'une cellule morte avant que sa représentation disparaisse.
const FADE_SECONDS: f32 = 0.5;
/// Vitesse (unités de monde par seconde) au-delà de laquelle une cellule s'oriente et s'étire.
const MOVING_SPEED: f32 = 0.25;

pub(super) struct OrganismVisual {
    body: Handle<Node>,
    membrane: Handle<Mesh>,
    components: Vec<Handle<Node>>,
    detail: DetailLevel,
    /// Anatomie actuellement dessinée : elle change quand le corps se redéveloppe.
    anatomy: crate::biology::Anatomy,
    /// Allongement du corps selon la mobilité : un corps sessile est rond, un nageur est allongé.
    aspect: f32,
    position: Vector3<f32>,
    target_position: Vector3<f32>,
    /// Rayon de la membrane (taille d'énergie incluse), lissé comme la position.
    radius: f32,
    target_radius: f32,
    yaw: f32,
    stretch: f32,
    speed: f32,
    phase: f32,
    /// Palier de matériau déjà appliqué, pour ne pas réaffecter le matériau à chaque pas.
    material: Option<(u8, u8)>,
    /// Temps restant avant la disparition d'un organisme mort.
    fade: Option<f32>,
}

impl OrganismVisual {
    pub(super) fn new(
        graph: &mut Graph,
        cell: &Cell,
        surfaces: &Surfaces,
        materials: &Materials,
        radius: f32,
        position: Vector3<f32>,
        casts_shadows: bool,
    ) -> Self {
        let id = cell.id();
        let body = PivotBuilder::new(
            BaseBuilder::new()
                .with_name("Organisme")
                .with_instance_id(SceneNodeId(id))
                .with_local_transform(
                    TransformBuilder::new()
                        .with_local_position(position)
                        .build(),
                ),
        )
        .build(graph);
        let membrane = mesh(
            graph,
            "Membrane",
            Uuid::now_v7(),
            Vector3::zeros(),
            Vector3::repeat(radius),
            surfaces.cell(id.as_u128() as usize),
            materials.get(MaterialKind::Healthy),
        );
        graph[membrane].set_cast_shadows(casts_shadows);
        graph.link_nodes(membrane, body);
        let bits = id.as_u128();
        Self {
            body: body.transmute(),
            membrane,
            components: Vec::new(),
            detail: DetailLevel::Bare,
            anatomy: crate::biology::Anatomy::default(),
            aspect: 0.9 + 0.25 * cell.physiology().motility(),
            position,
            target_position: position,
            radius,
            target_radius: radius,
            yaw: ((bits >> 7) % 628) as f32 / 100.0,
            stretch: 0.0,
            speed: 0.0,
            phase: ((bits >> 19) % 628) as f32 / 100.0,
            material: None,
            fade: None,
        }
    }

    /// Facteur de taille d'un organisme selon le nombre de voisins dans son site : les foules
    /// restent lisibles sans jamais masquer ni déplacer un individu.
    pub(super) fn crowd_scale(neighbors: usize) -> f32 {
        (1.0 / (1.0 + neighbors.saturating_sub(1) as f32 * 0.02))
            .sqrt()
            .max(0.35)
    }

    pub(super) fn body(&self) -> Handle<Node> {
        self.body
    }
    pub(super) fn position(&self) -> Vector3<f32> {
        self.position
    }
    pub(super) fn radius(&self) -> f32 {
        self.radius
    }

    /// Vrai une fois la disparition de l'organisme mort terminée.
    pub(super) fn is_gone(&self) -> bool {
        self.fade.is_some_and(|remaining| remaining <= 0.0)
    }

    /// Nouvelle cible issue d'un pas biologique. Une cible lointaine (chargement, division
    /// d'un voisin éloigné) est atteinte sans glissement à travers toute la grille.
    pub(super) fn set_target(&mut self, position: Vector3<f32>, radius: f32, teleport: f32) {
        self.target_position = position;
        self.target_radius = radius;
        if (self.position - position).norm() > teleport {
            self.position = position;
            self.radius = radius;
        }
    }

    /// Démarre la disparition : la cellule rétrécit sur place puis son nœud est retiré.
    pub(super) fn begin_fade(&mut self) {
        if self.fade.is_none() {
            self.fade = Some(FADE_SECONDS);
            self.target_radius = 0.0;
            self.target_position.y = 0.0;
        }
    }

    /// Enregistre les paliers de matériau et indique s'ils ont changé.
    pub(super) fn update_material(&mut self, gradient_band: u8, mode: u8) -> bool {
        self.material.replace((gradient_band, mode)) != Some((gradient_band, mode))
    }

    pub(super) fn membrane(&self) -> Handle<Mesh> {
        self.membrane
    }

    /// Fixe la finesse du dessin. L'anatomie est assemblée d'après le phénotype de l'organisme
    /// (composants de la bibliothèque, ou formes procédurales à défaut) à chaque changement de
    /// finesse vers un niveau détaillé, et masquée quand l'organisme redevient une simple membrane.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn set_detail(
        &mut self,
        scene: &mut Scene,
        cell: &Cell,
        parts: &[crate::definitions::PartDefinition],
        library: &ComponentLibrary,
        surfaces: &Surfaces,
        materials: &Materials,
        detail: DetailLevel,
    ) {
        if detail == self.detail && *cell.morphology().anatomy() == self.anatomy {
            return;
        }
        for node in self.components.drain(..) {
            scene.graph.remove_node(node);
        }
        self.detail = detail;
        self.anatomy = cell.morphology().anatomy().clone();
        self.components = organism_assembly::assemble(
            scene,
            self.membrane,
            cell.id(),
            cell.morphology().anatomy(),
            parts,
            detail,
            library,
            surfaces,
            materials,
        );
    }

    /// Rapproche l'affichage de la cible pendant `dt` secondes réelles et renvoie vrai
    /// si une transformation doit être réécrite.
    pub(super) fn animate(&mut self, dt: f32) -> bool {
        if let Some(remaining) = &mut self.fade {
            *remaining -= dt;
        }
        let before = self.position;
        let remaining = (self.target_position - self.position).norm()
            + (self.target_radius - self.radius).abs();
        let settled = remaining <= SETTLED;
        if settled {
            self.position = self.target_position;
            self.radius = self.target_radius;
        } else {
            let blend = 1.0 - (-dt / SMOOTHING_SECONDS).exp();
            self.position += (self.target_position - self.position) * blend;
            self.radius += (self.target_radius - self.radius) * blend;
        }
        // Cap et étirement suivent le mouvement effectif, lui-même indépendant des images.
        let travelled = self.position - before;
        let speed = if dt > 0.0 { travelled.norm() / dt } else { 0.0 };
        self.speed += (speed - self.speed) * (1.0 - (-dt / 0.12).exp());
        let moving = self.speed > MOVING_SPEED;
        if moving && (travelled.x != 0.0 || travelled.z != 0.0) {
            let wanted = travelled.x.atan2(travelled.z);
            let mut delta = wanted - self.yaw;
            delta = (delta + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
                - std::f32::consts::PI;
            self.yaw += delta * (1.0 - (-dt / 0.08).exp());
        }
        let stretch_target = if moving {
            (self.speed / 1.6).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let before_stretch = self.stretch;
        self.stretch += (stretch_target - self.stretch) * (1.0 - (-dt / 0.10).exp());
        !settled || (self.stretch - before_stretch).abs() > 1e-5 || self.stretch > 1e-3
    }

    /// Écrit la pose courante dans la scène. `time` anime la respiration des cellules détaillées.
    pub(super) fn write(&self, graph: &mut Graph, time: f32) {
        graph[self.body]
            .local_transform_mut()
            .set_position(self.position)
            .set_rotation(UnitQuaternion::from_axis_angle(&Vector3::y_axis(), self.yaw));
        let breath = if self.detail != DetailLevel::Bare && self.fade.is_none() {
            1.0 + 0.035 * (time * 2.2 + self.phase).sin()
        } else {
            1.0
        };
        let along = self.radius * breath * self.aspect * (1.0 + 0.30 * self.stretch);
        let across = self.radius * breath * (1.0 - 0.12 * self.stretch);
        graph[self.membrane]
            .local_transform_mut()
            .set_scale(Vector3::new(across, across, along));
    }

    /// Une cellule détaillée respire en continu : elle doit être réécrite à chaque image.
    pub(super) fn is_breathing(&self) -> bool {
        self.detail != DetailLevel::Bare && self.fade.is_none()
    }
}
