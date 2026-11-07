use crate::world_scene::{habitat_animation::HabitatAnimation, habitat_part::HabitatPart};
use fyrox::{core::algebra::Vector3, scene::graph::Graph};

/// Représentation d'un habitat non vide : ses éléments et leur éventuelle animation.
pub(super) struct HabitatVisual {
    parts: Vec<HabitatPart>,
    animation: HabitatAnimation,
    phase: f32,
    /// Palier de remplissage de la nourriture déjà appliqué.
    food_bucket: Option<u8>,
}

impl HabitatVisual {
    pub(super) fn new(parts: Vec<HabitatPart>, animation: HabitatAnimation, phase: f32) -> Self {
        Self {
            parts,
            animation,
            phase,
            food_bucket: None,
        }
    }

    pub(super) fn animation(&self) -> HabitatAnimation {
        self.animation
    }

    /// Enregistre le palier de nourriture et indique s'il a changé.
    pub(super) fn update_food_bucket(&mut self, bucket: u8) -> bool {
        self.food_bucket.replace(bucket) != Some(bucket)
    }

    /// Les baies grossissent avec la réserve du site (`fill` dans [0, 1]).
    pub(super) fn set_fill(&self, graph: &mut Graph, fill: f32) {
        let growth = 0.40 + 0.60 * fill;
        for part in &self.parts {
            graph[part.node()]
                .local_transform_mut()
                .set_scale(part.scale() * growth)
                .set_position(Vector3::new(
                    part.position().x,
                    part.position().y * growth,
                    part.position().z,
                ));
        }
    }

    /// Anime l'habitat à l'instant `time` (secondes réelles).
    pub(super) fn animate(&self, graph: &mut Graph, time: f32) {
        match self.animation {
            HabitatAnimation::Still => {}
            HabitatAnimation::Pulse => {
                let pulse = 1.0 + 0.07 * (time * 2.6 + self.phase).sin();
                for part in &self.parts {
                    graph[part.node()]
                        .local_transform_mut()
                        .set_scale(part.scale() * pulse);
                }
            }
            HabitatAnimation::Bob => {
                for (index, part) in self.parts.iter().enumerate() {
                    let lift = 0.035 * (time * 1.4 + self.phase + index as f32 * 2.1).sin();
                    graph[part.node()]
                        .local_transform_mut()
                        .set_position(part.position() + Vector3::new(0.0, lift, 0.0));
                }
            }
        }
    }
}
