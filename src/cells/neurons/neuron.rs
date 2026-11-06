use crate::cells::neurons::NeuronConnection;
use rand::RngExt;

#[derive(Clone)]
pub(crate) struct Neuron {
    connections: Vec<NeuronConnection>,
    bias: f32,
    activation: f32,
    complexity: u32,
}

impl Neuron {
    pub(crate) fn new() -> Self {
        Self {
            connections: Vec::new(),
            bias: -20.0,
            activation: 0.0,
            complexity: 1,
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

    pub(crate) fn evolve(&mut self, new_complexity: u32) {
        let mut rng = rand::rng();

        // Ajuste la complexité du réseau neuronal
        while self.complexity < new_complexity {
            // Ajoute une nouvelle connexion
            self.connections.push(NeuronConnection::new(
                rng.random_range(-1.0..1.0),
                rng.random_range(0..self.complexity as usize),
                self.complexity as usize,
            ));
            self.complexity += 1;
        }

        // Mute quelques connexions existantes
        for connection in &mut self.connections {
            if rng.random_bool(0.1) {
                // 10% de chance de mutation
                *connection.weight_mut() += rng.random_range(-0.2..0.2);
            }
        }

        // Mute le biais
        if rng.random_bool(0.1) {
            self.bias += rng.random_range(-1.0..1.0);
        }
    }

    pub(crate) fn process(&mut self, inputs: &[f32]) -> bool {
        if self.complexity == 1 {
            // Comportement simple pour une cellule primitive
            return inputs[0] * 0.8 + self.bias > 0.0;
        }

        // Réinitialise les activations
        let mut activations = vec![0.0; self.complexity as usize];

        // Copie les entrées
        for (i, &input) in inputs.iter().enumerate().take(activations.len()) {
            activations[i] = input;
        }

        // Propage l'activation à travers le réseau
        for connection in &self.connections {
            activations[connection.to_id()] +=
                activations[connection.from_id()] * connection.weight();
        }

        // Applique la fonction d'activation (tanh) sur le dernier neurone
        self.activation = (activations.last().unwrap_or(&0.0) + self.bias).tanh();

        self.activation > 0.0
    }
}
