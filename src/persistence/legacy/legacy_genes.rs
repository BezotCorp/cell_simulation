use serde::Deserialize;

/// Gènes directs des fichiers antérieurs au génome (versions 1 à 6) : paramètres du phénotype.
#[derive(Deserialize)]
pub(super) struct LegacyGenes {
    pub(super) metabolism_rate: f32,
    pub(super) division_threshold: f32,
    #[allow(dead_code)]
    pub(super) mutation_rate: f32,
    pub(super) water_adaptation: f32,
    pub(super) land_adaptation: f32,
    pub(super) neuron_complexity: u32,
}
