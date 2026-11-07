use crate::definitions::{Definitions, SpeciesDefinition};
use std::sync::Arc;
use uuid::Uuid;

fn default_species() -> Arc<SpeciesDefinition> {
    Definitions::builtin().species().clone()
}

fn centered() -> f32 {
    0.5
}
use std::time::Duration;

use crate::biology::CellGenes;
use crate::biology::Neuron;
use crate::habitat::Environment;
use crate::hydrology::Flow;

/// Vitesse de nage d'un organisme de métabolisme et d'adaptation aquatique égaux à 1, en sites par
/// seconde : la poussée dont il dispose face au courant. Elle croît avec le métabolisme (puissance)
/// et l'adaptation à l'eau (efficacité de la nage).
const SWIM_SPEED: f32 = 0.12;
/// Part du métabolisme dépensée en plus pour tenir tête, à pleine poussée, au courant.
const SWIM_COST: f32 = 0.3;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub(crate) struct Cell {
    id: Uuid,
    species_id: Uuid,
    #[serde(skip, default = "default_species")]
    species: Arc<SpeciesDefinition>,
    /// Site de la grille technique qui contient l'organisme (index de calcul, pas un enclos).
    x: usize,
    y: usize,
    /// Position réelle dans le site, en fraction de sa largeur. Elle est écrite dans une section
    /// dédiée du fichier BIN afin que le corps de la session garde son format historique.
    #[serde(skip, default = "centered")]
    east: f32,
    #[serde(skip, default = "centered")]
    south: f32,
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
            east: 0.5,
            south: 0.5,
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

    /// Position réelle en unités de site : `(x + est, y + sud)`, dans `[0, taille)`.
    pub(crate) fn position(&self) -> (f32, f32) {
        (self.x as f32 + self.east, self.y as f32 + self.south)
    }

    /// Fractions de position dans le site, pour la persistance.
    pub(crate) fn offsets(&self) -> (f32, f32) {
        (self.east, self.south)
    }

    /// Contrôle des fractions lues dans une sauvegarde.
    pub(crate) fn offsets_are_valid(east: f32, south: f32) -> bool {
        (0.0..1.0).contains(&east) && (0.0..1.0).contains(&south)
    }

    pub(crate) fn restore_offsets(&mut self, east: f32, south: f32) {
        self.east = east;
        self.south = south;
    }

    /// Sauvegardes sans positions fines : le point dans le site est dérivé de l'identité, ce
    /// qui évite d'empiler tous les organismes d'un site sur le même point.
    pub(crate) fn settle_legacy_offsets(&mut self) {
        let bytes = self.id.as_bytes();
        let fraction = |a: u8, b: u8| 0.15 + 0.7 * (u16::from_le_bytes([a, b]) as f32 / 65_536.0);
        self.east = fraction(bytes[10], bytes[11]);
        self.south = fraction(bytes[14], bytes[15]);
    }

    /// Pose l'organisme en `point` (unités de site), avec bouclage du monde torique.
    fn set_position(&mut self, point: (f32, f32), size: usize) {
        let extent = size as f32;
        let split = |value: f32| {
            let wrapped = value.rem_euclid(extent);
            let site = (wrapped.floor() as usize).min(size - 1);
            (site, (wrapped - site as f32).clamp(0.0, 0.999_999))
        };
        (self.x, self.east) = split(point.0);
        (self.y, self.south) = split(point.1);
    }

    /// Installe un nouveau-né à `point` sans coût énergétique.
    pub(crate) fn settle(&mut self, point: (f32, f32), size: usize) {
        self.set_position(point, size);
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

    /// Poussée maximale de nage contre le courant, en sites par seconde.
    pub(crate) fn swim_capacity(&self) -> f32 {
        SWIM_SPEED * self.genes.water_adaptation() * self.genes.metabolism_rate()
    }

    /// Subit le courant de son site pendant `dt`. L'organisme nage contre l'écoulement avec toute
    /// sa poussée disponible (cela lui coûte de l'énergie) ; le reste du courant l'emporte. Une
    /// lame d'eau mince ne l'entraîne que partiellement. Renvoie le déplacement (est, sud) en sites.
    pub(crate) fn swim_against(&mut self, flow: Flow, dt: Duration) -> Option<(f32, f32)> {
        let wetted = flow.wetted();
        let speed = flow.speed() * wetted;
        if speed < 1e-4 {
            return None;
        }
        let capacity = self.swim_capacity();
        let held = speed.min(capacity);
        if capacity > 0.0 {
            self.energy -=
                self.genes.metabolism_rate() * SWIM_COST * (held / capacity) * dt.as_secs_f32();
        }
        let carried = (speed - held) * dt.as_secs_f32() / speed;
        (carried > 0.0).then(|| (flow.east() * wetted * carried, flow.south() * wetted * carried))
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
            new_cell.east = self.east;
            new_cell.south = self.south;
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

    /// Déplacement actif vers `point` (unités de site) : il coûte le même effort quelle que soit
    /// la direction.
    pub(crate) fn move_to(&mut self, point: (f32, f32), size: usize) {
        if point == self.position() {
            return;
        }
        self.energy -=
            self.genes.metabolism_rate() * self.species.rules().movement_energy_seconds();
        self.set_position(point, size);
    }

    pub(crate) fn energy_deficit(&self) -> f32 {
        (self.species.rules().energy_capacity() - self.energy).max(0.0)
    }

    pub(crate) fn feed(&mut self, amount: f32) {
        self.energy = (self.energy + amount).min(self.species.rules().energy_capacity());
    }
}
