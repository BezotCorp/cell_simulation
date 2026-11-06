use rand::RngExt;
use std::time::Duration;

use crate::cells::CellGenes;
use crate::cells::Neuron;

pub(crate) struct Cell {
    x: usize,
    y: usize,
    energy: f32,
    neuron: Neuron,
    genes: CellGenes,
    age: Duration,
    last_division: Duration,
}

impl Cell {
    pub(crate) fn new(x: usize, y: usize) -> Self {
        Self {
            x,
            y,
            energy: 100.0,
            neuron: Neuron::new(),
            genes: CellGenes::new(),
            age: Duration::from_secs(0),
            last_division: Duration::from_secs(0),
        }
    }

    pub(crate) fn x(&self) -> usize {
        self.x
    }

    pub(crate) fn y(&self) -> usize {
        self.y
    }

    pub(crate) fn energy(&self) -> f32 {
        self.energy
    }

    pub(crate) fn neuron(&self) -> &Neuron {
        &self.neuron
    }

    pub(crate) fn genes(&self) -> &CellGenes {
        &self.genes
    }

    pub(crate) fn age(&self) -> Duration {
        self.age
    }

    pub(crate) fn last_division(&self) -> Duration {
        self.last_division
    }

    pub(crate) fn update(&mut self, temperature: f32, environment_type: &str, grid_size: usize) {
        // Consomme de l'énergie selon le métabolisme et l'environnement
        let environment_penalty = match environment_type {
            "water" => 1.0 - self.genes.water_adaptation,
            "land" => 1.0 - self.genes.land_adaptation,
            _ => 0.0,
        };

        self.energy -= self.genes.metabolism_rate * (1.0 + environment_penalty);

        // Met à jour l'âge
        self.age += Duration::from_millis(16);

        // Prépare les entrées pour le neurone
        let inputs = vec![temperature];

        // Le neurone décide si on doit bouger
        if self.neuron.process(&inputs) && self.energy > 10.0 {
            self.energy -= 5.0 * self.genes.metabolism_rate;
            self.move_random(grid_size);
        }
    }

    pub(crate) fn try_divide(&mut self) -> Option<Cell> {
        if self.energy >= self.genes.division_threshold
            && self.age - self.last_division > Duration::from_secs(30)
        {
            // Crée une nouvelle cellule avec des gènes potentiellement mutés
            let mut new_genes = self.genes.clone();
            new_genes.mutate();

            // La cellule parent perd de l'énergie
            self.energy /= 2.0;
            self.last_division = self.age;

            // Crée la cellule fille
            let mut new_cell = Cell::new(self.x, self.y);
            new_cell.energy = self.energy;
            new_cell.genes = new_genes;
            Some(new_cell)
        } else {
            None
        }
    }

    fn move_random(&mut self, grid_size: usize) {
        let mut rng = rand::rng();
        let direction = match rng.random_range(0..4) {
            0 => (1, 0),
            1 => (-1, 0),
            2 => (0, 1),
            _ => (0, -1),
        };

        self.x = (self.x as i32 + direction.0).rem_euclid(grid_size as i32) as usize;
        self.y = (self.y as i32 + direction.1).rem_euclid(grid_size as i32) as usize;
    }

    pub(crate) fn feed(&mut self, amount: f32) {
        self.energy = (self.energy + amount).min(100.0);
    }
}
