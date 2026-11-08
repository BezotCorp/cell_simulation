use crate::biology::AnatomyPart;
use crate::definitions::{PartDefinition, PartDriver};
use crate::genetics::{Levels, Signals};

/// Composants visibles d'un organisme, déduits de ses produits et de son milieu par les définitions
/// de composants de l'espèce. La liste n'est pas fermée : une combinaison inédite de produits donne
/// une combinaison inédite de composants.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Anatomy {
    parts: Vec<AnatomyPart>,
}

impl Anatomy {
    /// Anatomie d'un corps de taille `size` (en diamètres de l'ancêtre) dont les produits sont
    /// `levels`, développé dans le milieu `medium`.
    pub(crate) fn of(
        definitions: &[PartDefinition],
        levels: &Levels,
        medium: &Signals,
        size: f32,
        neural_units: usize,
    ) -> Self {
        let parts = definitions
            .iter()
            .enumerate()
            .filter_map(|(index, definition)| {
                let level = match definition.driver {
                    PartDriver::Product(product) => levels.get(product),
                    PartDriver::NeuralUnits => neural_units as f32 / 8.0,
                    PartDriver::Always => 1.0,
                };
                let present = level >= definition.appears_at
                    && definition.when.iter().all(|range| range.holds(medium.get(range.signal)));
                present.then(|| {
                    let count = (level * definition.count.per_level * size.max(0.5))
                        .ceil()
                        .clamp(1.0, f32::from(definition.count.max.max(1)));
                    AnatomyPart::new(index, count as u8, level.min(1.0), definition.variant_at(level))
                })
            })
            .collect();
        Self { parts }
    }

    pub(crate) fn parts(&self) -> &[AnatomyPart] {
        &self.parts
    }
}

