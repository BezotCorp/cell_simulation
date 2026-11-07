use crate::definitions::{Definitions, SpeciesDefinition};
use std::sync::Arc;
use uuid::Uuid;

fn default_species() -> Arc<SpeciesDefinition> {
    Definitions::builtin().species().clone()
}
use std::time::Duration;

use crate::biology::CellGenes;
use crate::biology::Neuron;
use crate::habitat::Environment;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub(crate) struct Cell {
    id: Uuid,
    species_id: Uuid,
    #[serde(skip, default = "default_species")]
    species: Arc<SpeciesDefinition>,
    x: usize,
    y: usize,
    energy: f32,
    neuron: Neuron,
    genes: CellGenes,
    age: Duration,
    last_division: Duration,
    movement_elapsed: Duration,
}

impl Cell {
    pub(crate) fn new(x: usize, y: usize, species: Arc<SpeciesDefinition>) -> Self {
        let neuron = Neuron::new(
            species.initial_genes().neuron_complexity(),
            &species.neural(),
        );
        Self {
            id: Uuid::now_v7(),
            species_id: species.id(),
            energy: species.rules().energy_capacity(),
            genes: species.initial_genes().clone(),
            species,
            x,
            y,
            neuron,
            age: Duration::from_secs(0),
            last_division: Duration::from_secs(0),
            movement_elapsed: Duration::ZERO,
        }
    }

    pub(crate) fn id(&self) -> Uuid {
        self.id
    }
    pub(crate) fn species_id(&self) -> Uuid {
        self.species_id
    }
    /// Contrôle sans rien modifier que l'état restauré est compatible avec l'espèce.
    pub(crate) fn verify_species(&self, species: &SpeciesDefinition) -> Result<(), String> {
        if species.id() != self.species_id {
            return Err("Définition d'espèce absente du catalogue.".into());
        }
        self.genes.validate(&species.rules())?;
        self.neuron.validate(self.genes.neuron_complexity())?;
        if !self.energy.is_finite()
            || self.energy <= 0.0
            || self.energy > species.rules().energy_capacity()
            || self.last_division > self.age
        {
            return Err("État physiologique invalide.".into());
        }
        Ok(())
    }

    /// Adopte une espèce déjà vérifiée par [`Cell::verify_species`].
    pub(crate) fn adopt_species(&mut self, species: Arc<SpeciesDefinition>) {
        self.species = species;
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

    pub(crate) fn update(
        &mut self,
        temperature: f32,
        environment: Environment,
        food_available: f32,
        dt: Duration,
    ) -> bool {
        // Les dépenses sont des taux par seconde de temps simulé.
        self.energy -= self.maintenance_cost(environment, dt);

        // Met à jour l'âge
        self.age += dt;

        // La recherche de nourriture est indépendante de la température et du rendu.
        // Un déplacement coûte une seconde d'activité métabolique et ne peut survenir qu'une fois/seconde.
        self.movement_elapsed += dt;
        let neural_movement = self.neuron.process(&[temperature]);
        let wants_to_move = self.energy > self.species.rules().movement_reserve()
            && food_available < self.energy_deficit().min(self.genes.metabolism_rate())
            && (self.energy < self.genes.division_threshold() || neural_movement);
        let interval = Duration::from_secs_f32(self.species.rules().movement_seconds());
        if self.movement_elapsed >= interval {
            self.movement_elapsed -= interval;
            return wants_to_move;
        }
        false
    }

    pub(crate) fn maintenance_cost(&self, environment: Environment, dt: Duration) -> f32 {
        let penalty = match environment {
            Environment::Water => 1.0 - self.genes.water_adaptation(),
            Environment::Land => 1.0 - self.genes.land_adaptation(),
        };
        self.genes.metabolism_rate() * (1.0 + penalty) * dt.as_secs_f32()
    }

    pub(crate) fn try_divide(&mut self, rng: &mut impl rand::Rng) -> Option<Cell> {
        if self.energy >= self.genes.division_threshold()
            && self.age - self.last_division
                >= Duration::from_secs(self.species.rules().maturity_seconds())
        {
            // Crée une nouvelle cellule avec des gènes potentiellement mutés
            let mut new_genes = self.genes.clone();
            new_genes.mutate(rng, &self.species.rules());

            // La cellule parent perd de l'énergie
            self.energy /= 2.0;
            self.last_division = self.age;

            // Crée la cellule fille
            let mut new_cell = Cell::new(self.x, self.y, self.species.clone());
            new_cell.energy = self.energy;
            new_cell.neuron = self.neuron.clone();
            new_cell.neuron.evolve(
                new_genes.neuron_complexity(),
                new_genes.mutation_rate(),
                rng,
            );
            new_cell.genes = new_genes;
            Some(new_cell)
        } else {
            None
        }
    }

    pub(crate) fn move_to(&mut self, x: usize, y: usize) {
        if (self.x, self.y) == (x, y) {
            return;
        }
        self.energy -=
            self.genes.metabolism_rate() * self.species.rules().movement_energy_seconds();
        self.x = x;
        self.y = y;
    }

    pub(crate) fn energy_deficit(&self) -> f32 {
        (self.species.rules().energy_capacity() - self.energy).max(0.0)
    }

    pub(crate) fn feed(&mut self, amount: f32) {
        self.energy = (self.energy + amount).min(self.species.rules().energy_capacity());
    }
}
