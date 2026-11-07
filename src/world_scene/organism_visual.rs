use crate::{biology::Cell, definitions::MaterialKind, world_scene::materials::Materials};
use fyrox::graph::SceneGraph;
use fyrox::{
    core::{algebra::Vector3, pool::Handle},
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

pub(crate) struct OrganismVisual {
    body: Handle<Node>,
    membrane: Handle<Mesh>,
    position: Vector3<f32>,
    target_position: Vector3<f32>,
    /// Rayon de la membrane (taille d'énergie incluse), lissé comme la position.
    radius: f32,
    target_radius: f32,
    /// Palier de matériau déjà appliqué, pour ne pas réaffecter le matériau à chaque pas.
    material: Option<(u8, u8)>,
}

impl OrganismVisual {
    pub(crate) fn new(
        graph: &mut Graph,
        cell: &Cell,
        radius: f32,
        position: Vector3<f32>,
        materials: &Materials,
    ) -> Self {
        let body = PivotBuilder::new(
            BaseBuilder::new()
                .with_name("Organisme")
                .with_instance_id(SceneNodeId(cell.id()))
                .with_local_transform(
                    TransformBuilder::new()
                        .with_local_position(position)
                        .build(),
                ),
        )
        .build(graph);
        let membrane = crate::world_scene::geometry::mesh(
            graph,
            "Membrane",
            Uuid::now_v7(),
            Vector3::zeros(),
            Vector3::repeat(radius),
            true,
            materials.get(MaterialKind::Healthy),
        );
        graph.link_nodes(membrane, body);
        // Le noyau est enfant de la membrane : sa taille et son décalage suivent son rayon.
        let nucleus = crate::world_scene::geometry::mesh(
            graph,
            "Noyau",
            Uuid::now_v7(),
            Vector3::new(0.2, 0.2, -0.1),
            Vector3::repeat(0.42),
            true,
            materials.get(MaterialKind::Nucleus),
        );
        graph.link_nodes(nucleus, membrane);
        Self {
            body: body.transmute(),
            membrane,
            position,
            target_position: position,
            radius,
            target_radius: radius,
            material: None,
        }
    }

    /// Disposition de l'occupant `index` parmi `count` sur un habitat : spirale de Vogel
    /// dans la dalle, avec un rayon réduit pour que les voisins ne se recouvrent pas.
    /// Renvoie le décalage horizontal et le plus grand rayon sans recouvrement.
    pub(crate) fn slot(index: usize, count: usize, spacing: f32) -> (Vector3<f32>, f32) {
        if count <= 1 {
            return (Vector3::zeros(), f32::INFINITY);
        }
        let reach = 0.40 * spacing;
        let fraction = (index as f32 + 0.5) / count as f32;
        let angle = index as f32 * std::f32::consts::PI * (3.0 - 5.0_f32.sqrt());
        let distance = reach * fraction.sqrt();
        // Écart moyen entre voisins d'une répartition uniforme sur le disque.
        let neighbor = reach * (std::f32::consts::PI / count as f32).sqrt();
        (
            Vector3::new(angle.cos() * distance, 0.0, angle.sin() * distance),
            neighbor * 0.40,
        )
    }

    pub(crate) fn body(&self) -> Handle<Node> {
        self.body
    }
    pub(crate) fn membrane(&self) -> Handle<Mesh> {
        self.membrane
    }
    pub(crate) fn position(&self) -> Vector3<f32> {
        self.position
    }
    pub(crate) fn radius(&self) -> f32 {
        self.radius
    }

    /// Nouvelle cible issue d'un pas biologique. Une cible lointaine (chargement, division
    /// d'un voisin éloigné) est atteinte sans glissement à travers toute la grille.
    pub(crate) fn set_target(&mut self, position: Vector3<f32>, radius: f32, teleport: f32) {
        self.target_position = position;
        self.target_radius = radius;
        if (self.position - position).norm() > teleport {
            self.position = position;
            self.radius = radius;
        }
    }

    /// Enregistre les paliers de matériau et indique s'ils ont changé.
    pub(crate) fn update_material(&mut self, gradient_band: u8, mode: u8) -> bool {
        self.material.replace((gradient_band, mode)) != Some((gradient_band, mode))
    }

    /// Rapproche l'affichage de la cible pendant `dt` secondes réelles et renvoie vrai
    /// si une transformation doit être réécrite.
    pub(crate) fn animate(&mut self, dt: f32) -> bool {
        let remaining = (self.target_position - self.position).norm()
            + (self.target_radius - self.radius).abs();
        if remaining <= SETTLED {
            let changed = remaining > 0.0;
            self.position = self.target_position;
            self.radius = self.target_radius;
            return changed;
        }
        let blend = 1.0 - (-dt / SMOOTHING_SECONDS).exp();
        self.position += (self.target_position - self.position) * blend;
        self.radius += (self.target_radius - self.radius) * blend;
        true
    }
}
