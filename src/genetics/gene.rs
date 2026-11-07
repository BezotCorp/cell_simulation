use crate::genetics::{Product, Signal};
use rand::RngExt;
use serde::{Deserialize, Serialize};

/// Résolution des valeurs entières d'un gène : 1/1024.
pub(crate) const SCALE: f32 = 1024.0;
/// Nombre de régulateurs d'un gène : les signaux du milieu puis les produits de l'organisme.
pub(crate) const REGULATORS: usize = Signal::COUNT + Product::COUNT;

/// Un gène : une expression de base, des sensibilités aux signaux du milieu et aux produits déjà
/// fabriqués (régulation), et des effets sur chaque produit. Les valeurs sont des entiers de
/// résolution 1/1024 : l'hérédité est exacte et le génome compact.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Gene {
    bias: i16,
    inputs: [i16; REGULATORS],
    effects: [i16; Product::COUNT],
}

fn quantize(value: f32) -> i16 {
    (value * SCALE).round().clamp(f32::from(i16::MIN), f32::from(i16::MAX)) as i16
}

impl Gene {
    pub(crate) fn new(
        bias: f32,
        inputs: [f32; REGULATORS],
        effects: [f32; Product::COUNT],
    ) -> Self {
        Self {
            bias: quantize(bias),
            inputs: inputs.map(quantize),
            effects: effects.map(quantize),
        }
    }

    pub(crate) fn bias(&self) -> f32 {
        f32::from(self.bias) / SCALE
    }

    /// Sensibilité au régulateur `index` (signaux d'abord, produits ensuite).
    pub(crate) fn input(&self, index: usize) -> f32 {
        f32::from(self.inputs[index]) / SCALE
    }

    pub(crate) fn effect(&self, product: Product) -> f32 {
        f32::from(self.effects[product.index()]) / SCALE
    }

    /// Activité du gène, de 0 à 1, pour des signaux et des produits donnés : un seuil linéaire
    /// centré sur 0,5, donc une expression constante quand toutes les sensibilités sont nulles.
    pub(crate) fn activity(&self, signals: &[f32; Signal::COUNT], products: &[f32; Product::COUNT]) -> f32 {
        let mut drive = f32::from(self.bias);
        for (weight, value) in self.inputs[..Signal::COUNT].iter().zip(signals) {
            drive += f32::from(*weight) * value;
        }
        for (weight, value) in self.inputs[Signal::COUNT..].iter().zip(products) {
            drive += f32::from(*weight) * value;
        }
        (0.5 + drive / SCALE).clamp(0.0, 1.0)
    }

    pub(crate) fn contribute(&self, activity: f32, into: &mut [f32; Product::COUNT]) {
        for (slot, effect) in into.iter_mut().zip(&self.effects) {
            *slot += activity * f32::from(*effect) / SCALE;
        }
    }

    /// Modifie au hasard une valeur du gène (expression de base, sensibilité ou effet). Le tirage
    /// est uniforme : rien n'oriente la mutation vers ce qui serait utile.
    pub(crate) fn mutate_value(&mut self, rng: &mut impl rand::Rng, step: i16) {
        let fields = 1 + REGULATORS + Product::COUNT;
        let which = rng.random_range(0..fields);
        let delta = rng.random_range(-step..=step);
        let target = match which {
            0 => &mut self.bias,
            n if n <= REGULATORS => &mut self.inputs[n - 1],
            n => &mut self.effects[n - 1 - REGULATORS],
        };
        *target = target.saturating_add(delta);
    }

    /// Un gène issu de rien : effets et sensibilités faibles tirés au hasard.
    pub(crate) fn random(rng: &mut impl rand::Rng) -> Self {
        let mut gene = Self {
            bias: rng.random_range(-512..=512),
            inputs: [0; REGULATORS],
            effects: [0; Product::COUNT],
        };
        for _ in 0..rng.random_range(1..=3) {
            gene.effects[rng.random_range(0..Product::COUNT)] = rng.random_range(-307..=307);
        }
        for _ in 0..rng.random_range(0..=2) {
            gene.inputs[rng.random_range(0..REGULATORS)] = rng.random_range(-512..=512);
        }
        gene
    }

    /// Valeurs lisibles : nombre de sensibilités et d'effets non nuls.
    pub(crate) fn connections(&self) -> usize {
        self.inputs.iter().filter(|v| **v != 0).count()
    }

    pub(crate) fn reach(&self) -> usize {
        self.effects.iter().filter(|v| **v != 0).count()
    }
}
