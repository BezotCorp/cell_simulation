use crate::genetics::Product;
use serde::{Deserialize, Serialize};

/// Quantité de chaque produit fabriquée par un organisme, de 0 à 1 : résultat de l'expression du
/// génome dans un milieu donné.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct Levels {
    values: [f32; Product::COUNT],
}

impl Levels {
    pub(crate) fn new(values: [f32; Product::COUNT]) -> Self {
        Self { values }
    }

    pub(crate) fn get(&self, product: Product) -> f32 {
        self.values[product.index()]
    }

    /// Vrai si un produit diffère de plus de `threshold` entre les deux expressions.
    pub(crate) fn differs_from(&self, other: &Levels, threshold: f32) -> bool {
        self.values.iter().zip(other.values).any(|(a, b)| (a - b).abs() > threshold)
    }

    pub(crate) fn is_valid(&self) -> bool {
        self.values.iter().all(|value| (0.0..=1.0).contains(value))
    }
}
