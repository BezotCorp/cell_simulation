use crate::definitions::BiologyRules;
use rand::RngExt;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub(crate) struct CellGenes {
    metabolism_rate: f32,    // Énergie consommée par seconde simulée
    division_threshold: f32, // Énergie nécessaire pour la division
    mutation_rate: f32,      // Probabilité de mutation lors de la division
    water_adaptation: f32,   // Capacité à survivre dans l'eau (0-1)
    land_adaptation: f32,    // Capacité à survivre sur terre (0-1)
    neuron_complexity: u32,  // Nombre de connexions neuronales
}

impl CellGenes {
    pub(crate) fn mutate(&mut self, rng: &mut impl rand::Rng, rules: &BiologyRules) {
        if rng.random::<f32>() < self.mutation_rate {
            // Mute un attribut aléatoire
            match rng.random_range(0..6) {
                0 => {
                    self.metabolism_rate = (self.metabolism_rate * rng.random_range(0.8..1.2))
                        .max(rules.basal_metabolism())
                }
                1 => {
                    self.division_threshold = (self.division_threshold * rng.random_range(0.9..1.1))
                        .clamp(
                            2.0 * rules.basal_metabolism() * rules.maturity_seconds() as f32,
                            rules.energy_capacity(),
                        )
                }
                2 => {
                    self.mutation_rate =
                        (self.mutation_rate * rng.random_range(0.95..1.05)).clamp(0.0, 1.0)
                }
                3 => {
                    self.water_adaptation =
                        (self.water_adaptation + rng.random_range(-0.1..0.1)).clamp(0.0, 1.0)
                }
                4 => {
                    self.land_adaptation =
                        (self.land_adaptation + rng.random_range(-0.1..0.1)).clamp(0.0, 1.0)
                }
                5 => {
                    self.neuron_complexity = if rng.random_bool(0.5) {
                        self.neuron_complexity.saturating_add(1)
                    } else {
                        self.neuron_complexity.saturating_sub(1).max(1)
                    };
                }
                _ => {}
            }
        }
    }

    pub(crate) fn validate(&self, rules: &BiologyRules) -> Result<(), String> {
        if !self.metabolism_rate.is_finite()
            || self.metabolism_rate < rules.basal_metabolism()
            || !self.division_threshold.is_finite()
            || self.division_threshold > rules.energy_capacity()
            || self.division_threshold
                < 2.0 * rules.basal_metabolism() * rules.maturity_seconds() as f32
            || !self.mutation_rate.is_finite()
            || !(0.0..=1.0).contains(&self.mutation_rate)
            || !self.water_adaptation.is_finite()
            || !(0.0..=1.0).contains(&self.water_adaptation)
            || !self.land_adaptation.is_finite()
            || !(0.0..=1.0).contains(&self.land_adaptation)
            || self.neuron_complexity == 0
        {
            return Err("Génome incompatible avec les règles biologiques.".into());
        }

        Ok(())
    }

    pub(crate) fn metabolism_rate(&self) -> f32 {
        self.metabolism_rate
    }

    pub(crate) fn division_threshold(&self) -> f32 {
        self.division_threshold
    }

    pub(crate) fn mutation_rate(&self) -> f32 {
        self.mutation_rate
    }

    pub(crate) fn water_adaptation(&self) -> f32 {
        self.water_adaptation
    }

    pub(crate) fn land_adaptation(&self) -> f32 {
        self.land_adaptation
    }

    pub(crate) fn neuron_complexity(&self) -> u32 {
        self.neuron_complexity
    }
}
