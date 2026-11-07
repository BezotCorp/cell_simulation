use crate::biology::NeuronConnection;
use rand::RngExt;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub(crate) struct Neuron {
    connections: Vec<NeuronConnection>,
    bias: f32,
    activation: f32,
    complexity: u32,
}

impl Neuron {
    pub(crate) fn new(complexity: u32, definition: &crate::definitions::NeuralDefinition) -> Self {
        Self {
            connections: (0..complexity as usize)
                .map(|index| {
                    NeuronConnection::new(
                        if index == 0 {
                            definition.temperature_weight()
                        } else {
                            1.0
                        },
                        index,
                        index + 1,
                    )
                })
                .collect(),
            bias: definition.bias(),
            activation: 0.0,
            complexity,
        }
    }

    pub(crate) fn connections(&self) -> &[NeuronConnection] {
        &self.connections
    }
    pub(crate) fn bias(&self) -> f32 {
        self.bias
    }
    pub(crate) fn activation(&self) -> f32 {
        self.activation
    }
    pub(crate) fn complexity(&self) -> u32 {
        self.complexity
    }

    /// Le nœud 0 reçoit la température ; les unités de calcul sont numérotées 1..=complexity.
    /// Ajouter une unité de gain 1 préserve le signal hérité avant mutation des poids.
    pub(crate) fn evolve(
        &mut self,
        new_complexity: u32,
        mutation_rate: f32,
        rng: &mut impl rand::Rng,
    ) {
        let new_complexity = new_complexity.max(1);
        self.connections
            .retain(|c| c.to_id() <= new_complexity as usize);
        while self.complexity < new_complexity {
            self.connections.push(NeuronConnection::new(
                1.0,
                self.complexity as usize,
                self.complexity as usize + 1,
            ));
            self.complexity += 1;
        }
        self.complexity = new_complexity;
        for connection in &mut self.connections {
            if rng.random_bool(mutation_rate as f64) {
                connection.mutate_weight(rng);
            }
        }
        if rng.random_bool(mutation_rate as f64) {
            self.bias += rng.random_range(-1.0..1.0);
        }
        // L'activité instantanée n'est pas un caractère héréditaire.
        self.activation = 0.0;
    }

    /// Le réseau est une chaîne validée : l'unité `k` reçoit uniquement la sortie de `k - 1`,
    /// le signal se propage donc sans tampon intermédiaire (même résultat que l'accumulation).
    pub(crate) fn process(&mut self, inputs: &[f32]) -> bool {
        let signal = self.connections.iter().fold(
            inputs.first().copied().unwrap_or(0.0),
            |signal, connection| signal * connection.weight(),
        );
        self.activation = (signal + self.bias).tanh();
        self.activation > 0.0
    }
    pub(crate) fn validate(&self, expected: u32) -> Result<(), String> {
        if self.complexity != expected
            || self.complexity == 0
            || self.connections.len() != self.complexity as usize
            || !self.bias.is_finite()
            || !self.activation.is_finite()
        {
            return Err("Réseau neuronal invalide.".into());
        }
        for (index, edge) in self.connections.iter().enumerate() {
            if edge.from_id() != index || edge.to_id() != index + 1 || !edge.weight().is_finite() {
                return Err("Topologie neuronale invalide.".into());
            }
        }
        Ok(())
    }
}
