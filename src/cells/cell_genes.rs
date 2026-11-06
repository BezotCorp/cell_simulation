use rand::RngExt;

#[derive(Clone)]
pub struct CellGenes {
    pub metabolism_rate: f32,    // Vitesse de consommation d'énergie
    pub division_threshold: f32, // Énergie nécessaire pour la division
    pub mutation_rate: f32,      // Probabilité de mutation lors de la division
    pub water_adaptation: f32,   // Capacité à survivre dans l'eau (0-1)
    pub land_adaptation: f32,    // Capacité à survivre sur terre (0-1)
    pub neuron_complexity: u32,  // Nombre de connexions neuronales
}

impl CellGenes {
    pub fn new() -> Self {
        Self {
            metabolism_rate: 1.0,
            division_threshold: 80.0,
            mutation_rate: 0.1,
            water_adaptation: 1.0, // Commence adaptée à l'eau
            land_adaptation: 0.1,  // Peu adaptée à la terre
            neuron_complexity: 1,
        }
    }

    pub fn mutate(&mut self) {
        let mut rng = rand::rng();
        if rng.random::<f32>() < self.mutation_rate {
            // Mute un attribut aléatoire
            match rng.random_range(0..6) {
                0 => self.metabolism_rate *= rng.random_range(0.8..1.2),
                1 => self.division_threshold *= rng.random_range(0.9..1.1),
                2 => self.mutation_rate *= rng.random_range(0.95..1.05),
                3 => {
                    self.water_adaptation =
                        (self.water_adaptation + rng.random_range(-0.1..0.1)).clamp(0.0, 1.0)
                }
                4 => {
                    self.land_adaptation =
                        (self.land_adaptation + rng.random_range(-0.1..0.1)).clamp(0.0, 1.0)
                }
                5 => {
                    self.neuron_complexity =
                        ((self.neuron_complexity as f32) * rng.random_range(0.8..1.2)) as u32
                }
                _ => {}
            }
        }
    }
}
