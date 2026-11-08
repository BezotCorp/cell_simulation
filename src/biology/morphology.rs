use crate::biology::Anatomy;
use crate::definitions::PartDefinition;
use crate::genetics::{Levels, Product, Signals};

/// Forme d'un organisme : l'expression du génome dans son milieu d'acclimatation détermine sa taille,
/// ses réserves, son tissu nerveux et son anatomie. Elle se redéveloppe quand l'expression change.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Morphology {
    size: f32,
    capacity: f32,
    neural_units: usize,
    levels: Levels,
    anatomy: Anatomy,
}

/// Taille minimale d'un organisme et étendue de la plage, en diamètres de l'ancêtre.
const SIZE_MIN: f32 = 0.6;
const SIZE_RANGE: f32 = 0.8;

impl Morphology {
    /// Développe un corps à partir des produits exprimés à la naissance ; `base_capacity` est la
    /// capacité énergétique du corps de référence (taille 1).
    pub(crate) fn develop(
        levels: Levels,
        base_capacity: f32,
        parts: &[PartDefinition],
        medium: &Signals,
    ) -> Self {
        let size = SIZE_MIN + SIZE_RANGE * levels.get(Product::Size);
        // Les réserves croissent comme le volume.
        let capacity = base_capacity * size.powi(3);
        let neural_units = (levels.get(Product::Neural) * 8.0 + 0.001).floor() as usize;
        Self {
            size,
            capacity,
            neural_units,
            anatomy: Anatomy::of(parts, &levels, medium, size, neural_units),
            levels,
        }
    }

    /// Plus grande réserve qu'un corps puisse atteindre.
    pub(crate) fn max_capacity(base_capacity: f32) -> f32 {
        base_capacity * (SIZE_MIN + SIZE_RANGE).powi(3)
    }

    /// Diamètre relatif à celui de l'ancêtre.
    pub(crate) fn size(&self) -> f32 {
        self.size
    }

    /// Réserve d'énergie maximale.
    pub(crate) fn capacity(&self) -> f32 {
        self.capacity
    }

    pub(crate) fn neural_units(&self) -> usize {
        self.neural_units
    }

    /// Produits exprimés au cours du développement.
    pub(crate) fn levels(&self) -> &Levels {
        &self.levels
    }

    pub(crate) fn anatomy(&self) -> &Anatomy {
        &self.anatomy
    }
}
