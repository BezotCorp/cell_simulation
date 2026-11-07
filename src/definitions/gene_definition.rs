use crate::genetics::{Gene, Product, REGULATORS, Signal};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Description lisible d'un gène dans le catalogue : expression de base, sensibilités aux signaux
/// du milieu et aux produits (régulation) et effets sur les produits.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GeneDefinition {
    /// Au-dessus de 0,5 le gène s'exprime en l'absence de tout signal, en dessous il attend d'être activé.
    #[serde(default)]
    bias: f32,
    #[serde(default)]
    signals: BTreeMap<Signal, f32>,
    #[serde(default)]
    products: BTreeMap<Product, f32>,
    #[serde(default)]
    effects: BTreeMap<Product, f32>,
}

impl GeneDefinition {
    pub(crate) fn validate(&self) -> Result<(), String> {
        let all = self
            .signals
            .values()
            .chain(self.products.values())
            .chain(self.effects.values())
            .chain([&self.bias]);
        if all.into_iter().any(|value| !value.is_finite() || value.abs() > 30.0) {
            return Err("Gène du catalogue invalide.".into());
        }
        Ok(())
    }

    pub(crate) fn to_gene(&self) -> Gene {
        let mut inputs = [0.0; REGULATORS];
        for (signal, weight) in &self.signals {
            inputs[signal.index()] = *weight;
        }
        for (product, weight) in &self.products {
            inputs[Signal::COUNT + product.index()] = *weight;
        }
        let mut effects = [0.0; Product::COUNT];
        for (product, weight) in &self.effects {
            effects[product.index()] = *weight;
        }
        Gene::new(self.bias, inputs, effects)
    }
}
