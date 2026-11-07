use crate::definitions::GeneDefinition;
use crate::genetics::{Genome, MAX_GENES, NEURAL_UNITS};
use serde::{Deserialize, Serialize};

/// Génome de l'ancêtre dans le catalogue : ses gènes et le câblage de son réseau neuronal (actif
/// seulement si l'expression du produit « tissu nerveux » le justifie).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GenomeDefinition {
    genes: Vec<GeneDefinition>,
    #[serde(default)]
    wiring: Vec<f32>,
    #[serde(default)]
    neural_bias: f32,
    #[serde(default)]
    sensor: u8,
}

impl GenomeDefinition {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.genes.is_empty()
            || self.genes.len() > MAX_GENES
            || self.wiring.len() > NEURAL_UNITS
            || self.sensor >= 4
            || !self.neural_bias.is_finite()
            || self.wiring.iter().any(|weight| !weight.is_finite())
        {
            return Err("Génome du catalogue invalide.".into());
        }
        self.genes.iter().try_for_each(GeneDefinition::validate)
    }

    /// Poids de la première connexion du réseau et biais : ce que `neural` portait auparavant.
    pub(crate) fn first_weight(&self) -> f32 {
        self.wiring.first().copied().unwrap_or(1.0)
    }

    pub(crate) fn neural_bias(&self) -> f32 {
        self.neural_bias
    }

    pub(crate) fn to_genome(&self) -> Genome {
        let mut wiring = [1.0; NEURAL_UNITS];
        for (slot, weight) in wiring.iter_mut().zip(&self.wiring) {
            *slot = *weight;
        }
        Genome::new(
            self.genes.iter().map(GeneDefinition::to_gene).collect(),
            wiring,
            self.neural_bias,
            self.sensor,
        )
    }
}
