use crate::definitions::SpeciesDefinition;
use crate::genetics::{Gene, Genome, NEURAL_UNITS, Product, REGULATORS};
use crate::persistence::legacy::{
    legacy_genes::LegacyGenes, legacy_neuron::LegacyNeuron, legacy_traits::LegacyTraits,
};
use crate::biology::Cell;
use serde::Deserialize;
use std::{sync::Arc, time::Duration};
use uuid::Uuid;

/// Organisme d'un fichier antérieur au génome.
#[derive(Deserialize)]
pub(super) struct LegacyCell {
    id: Uuid,
    #[allow(dead_code)]
    species_id: Uuid,
    x: usize,
    y: usize,
    energy: f32,
    neuron: LegacyNeuron,
    genes: LegacyGenes,
    age: Duration,
    last_division: Duration,
    movement_elapsed: Duration,
}

impl LegacyCell {
    pub(super) fn id(&self) -> Uuid {
        self.id
    }

    /// Traduit les paramètres directs en un génome équivalent : un gène constitutif par produit
    /// exprimé, dont l'effet reproduit la valeur de l'ancien paramètre. Le phénotype retrouvé est
    /// celui du fichier (à la résolution du génome près), et l'évolution repart de là.
    pub(super) fn into_cell(self, traits: &LegacyTraits, species: Arc<SpeciesDefinition>) -> Cell {
        let capacity = species.rules().energy_capacity();
        let genes = &self.genes;
        let units = genes.neuron_complexity.min(NEURAL_UNITS as u32) as f32;
        let mut levels = [0.0_f32; Product::COUNT];
        let mut set = |product: Product, value: f32| levels[product.index()] = value.clamp(0.0, 1.0);
        set(Product::Metabolism, (genes.metabolism_rate / 0.5).max(1.0).log2() / 2.0);
        set(Product::Size, 0.5);
        set(Product::Fecundity, (genes.division_threshold / capacity - 0.35) / 0.6);
        set(Product::Repair, 0.25);
        set(Product::Hydration, genes.water_adaptation);
        set(Product::Cuticle, genes.land_adaptation);
        set(Product::Neural, units / NEURAL_UNITS as f32);
        for (product, value) in [
            Product::Digestive,
            Product::Pigment,
            Product::Thermal,
            Product::Raptorial,
            Product::Motor,
            Product::Adhesin,
        ]
        .into_iter()
        .zip(traits.values)
        {
            set(product, value);
        }
        let genome_genes = Product::ALL
            .iter()
            .filter(|product| levels[product.index()] > 0.0)
            .map(|product| {
                let mut effects = [0.0; Product::COUNT];
                effects[product.index()] = levels[product.index()];
                Gene::new(0.5, [0.0; REGULATORS], effects)
            })
            .collect();
        let mut wiring = [1.0; NEURAL_UNITS];
        for (slot, connection) in wiring.iter_mut().zip(&self.neuron.connections) {
            *slot = connection.weight;
        }
        let genome = Genome::new(genome_genes, wiring, self.neuron.bias, traits.sensor.min(3));
        Cell::from_legacy(
            self.id,
            self.x,
            self.y,
            self.energy,
            genome,
            self.age,
            self.last_division,
            self.movement_elapsed,
            species,
        )
    }
}
