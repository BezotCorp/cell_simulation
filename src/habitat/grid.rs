use crate::definitions::{Definitions, WorldDefinition};
use std::sync::Arc;
use uuid::Uuid;
fn default_world() -> Arc<WorldDefinition> {
    Definitions::builtin().world().clone()
}
use crate::habitat::Entity;
use crate::habitat::Environment;
use crate::habitat::TemperatureMap;
use crate::habitat::Terrain;
use rand::RngExt;
use std::collections::HashMap;
use std::time::Duration;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub(crate) struct Grid {
    #[serde(skip, default = "default_world")]
    definition: Arc<WorldDefinition>,
    habitat_ids: Vec<Uuid>,
    size: usize,
    entities: Vec<Vec<Entity>>,
    temperature_map: TemperatureMap,
    update_elapsed: Duration,
    water_level: f32, // Niveau d'eau global (diminue progressivement)
    /// Index dérivé des identités persistantes, reconstruit à chaque liaison de définition.
    #[serde(skip)]
    habitat_index: HashMap<Uuid, usize>,
    /// Indices des sites alimentaires : la régénération ne parcourt pas toute la grille.
    #[serde(skip)]
    food_sites: Vec<usize>,
}

impl Grid {
    pub(crate) fn new(definition: Arc<WorldDefinition>, rng: &mut impl rand::Rng) -> Self {
        let size = definition.size();
        let mut grid = Self {
            habitat_ids: (0..size * size).map(|_| Uuid::now_v7()).collect(),
            temperature_map: TemperatureMap::new(size, definition.ambient_temperature()),
            definition,
            size,
            entities: vec![vec![Entity::Empty; size]; size],
            update_elapsed: Duration::ZERO,
            water_level: 1.0, // Commence complètement submergé
            habitat_index: HashMap::new(),
            food_sites: Vec::new(),
        };

        grid.initialize_environment(rng);
        grid.rebuild_indexes();
        grid
    }

    /// Reconstruit les index dérivés de l'état persistant (identités, sites alimentaires).
    fn rebuild_indexes(&mut self) {
        self.habitat_index = self
            .habitat_ids
            .iter()
            .enumerate()
            .map(|(index, id)| (*id, index))
            .collect();
        self.food_sites = self
            .entities
            .iter()
            .flatten()
            .enumerate()
            .filter(|(_, entity)| matches!(entity, Entity::Food(_)))
            .map(|(index, _)| index)
            .collect();
    }

    /// Position d'un habitat à partir de son identité persistante, sans parcours de la grille.
    pub(crate) fn habitat_position(&self, id: Uuid) -> Option<(usize, usize)> {
        self.habitat_index
            .get(&id)
            .map(|index| (index % self.size, index / self.size))
    }

    /// Indices à plat (`y * size + x`) des sites alimentaires.
    pub(crate) fn food_sites(&self) -> &[usize] {
        &self.food_sites
    }

    pub(crate) fn habitat_ids(&self) -> impl Iterator<Item = Uuid> + '_ {
        self.habitat_ids.iter().copied()
    }

    pub(crate) fn habitat_id(&self, x: usize, y: usize) -> Uuid {
        self.habitat_ids[y * self.size + x]
    }
    /// Contrôle sans rien modifier qu'un état restauré est compatible avec la définition.
    pub(crate) fn verify_definition(&self, definition: &WorldDefinition) -> Result<(), String> {
        if self.size != definition.size()
            || self.entities.len() != self.size
            || self.entities.iter().any(|r| r.len() != self.size)
            || self.habitat_ids.len() != self.size * self.size
            || !self.temperature_map.validate(self.size)
            || !self.water_level.is_finite()
            || !(0.0..=1.0).contains(&self.water_level)
            || self.update_elapsed >= Duration::from_secs(1)
        {
            return Err("Géométrie ou état du monde invalide.".into());
        }
        for entity in self.entities.iter().flatten() {
            let valid = match entity {
                Entity::Food(n) => n.is_finite() && *n >= 0.0 && *n <= definition.food_capacity(),
                Entity::HeatSource(n) => n.is_finite() && *n >= 0.0,
                Entity::Water(n) | Entity::Oxygen(n) => n.is_finite() && (0.0..=1.0).contains(n),
                _ => true,
            };
            if !valid {
                return Err("État de ressource invalide.".into());
            }
        }
        Ok(())
    }

    /// Adopte une définition déjà vérifiée par [`Grid::verify_definition`].
    pub(crate) fn adopt_definition(&mut self, definition: Arc<WorldDefinition>) {
        self.definition = definition;
        self.rebuild_indexes();
    }

    pub(crate) fn size(&self) -> usize {
        self.size
    }

    pub(crate) fn temperature_map(&self) -> &TemperatureMap {
        &self.temperature_map
    }

    pub(crate) fn water_level(&self) -> f32 {
        self.water_level
    }

    fn initialize_environment(&mut self, rng: &mut impl rand::Rng) {
        // Crée le terrain de base
        for y in 0..self.size {
            for x in 0..self.size {
                if rng.random_bool(self.definition.terrain_probability()) {
                    let terrain = match rng.random_range(0..3) {
                        0 => Terrain::Rock,
                        1 => Terrain::Sand,
                        _ => Terrain::Soil,
                    };
                    self.entities[y][x] = Entity::Obstacle(terrain);
                }
            }
        }

        // Ajoute des sources de chaleur
        for _ in 0..self.definition.heat_sources() {
            let x = rng.random_range(0..self.size);
            let y = rng.random_range(0..self.size);
            if let Entity::Empty = self.entities[y][x] {
                let intensity = rng.random_range(
                    self.definition.heat_intensity()[0]..self.definition.heat_intensity()[1],
                );
                self.entities[y][x] = Entity::HeatSource(intensity);
                self.temperature_map.apply_heat_source(x, y, intensity);
            }
        }

        // Place les ressources abiotiques une seule fois.
        for y in 0..self.size {
            for x in 0..self.size {
                if let Entity::Empty = self.entities[y][x] {
                    if rng.random_bool(self.definition.oxygen_probability()) {
                        self.entities[y][x] = Entity::Oxygen(rng.random_range(0.5..1.0));
                    } else if rng.random_bool(self.definition.water_probability()) {
                        self.entities[y][x] = Entity::Water(rng.random_range(0.5..1.0));
                    }
                }
            }
        }

        // Ajoute des ressources
        self.add_random_resources(rng);
        if !self.definition.placements().is_empty() {
            for placement in self.definition.placements() {
                self.entities[placement.y()][placement.x()] = placement.entity().clone();
            }
            // Recompute from the final shared definition, including replaced heat sources.
            self.temperature_map =
                TemperatureMap::new(self.size, self.definition.ambient_temperature());
            for y in 0..self.size {
                for x in 0..self.size {
                    if let Entity::HeatSource(intensity) = self.entities[y][x] {
                        self.temperature_map.apply_heat_source(x, y, intensity);
                    }
                }
            }
        }
    }

    fn add_random_resources(&mut self, rng: &mut impl rand::Rng) {
        // Ajoute de la nourriture
        for _ in 0..self.definition.food_attempts() {
            let x = rng.random_range(0..self.size);
            let y = rng.random_range(0..self.size);
            if let Entity::Empty = self.entities[y][x] {
                self.entities[y][x] = Entity::Food(self.definition.food_capacity());
                self.food_sites.push(y * self.size + x);
            }
        }
    }

    pub(crate) fn update(&mut self, dt: Duration, rng: &mut impl rand::Rng) {
        // Chaque site alimentaire produit un débit fini, avec une réserve maximale.
        let regeneration = self.definition.food_regeneration_per_second() * dt.as_secs_f32();
        let capacity = self.definition.food_capacity();
        for &index in &self.food_sites {
            if let Entity::Food(energy) = &mut self.entities[index / self.size][index % self.size] {
                *energy = (*energy + regeneration).min(capacity);
            }
        }
        self.update_elapsed += dt;
        while self.update_elapsed >= Duration::from_secs(1) {
            self.update_elapsed -= Duration::from_secs(1);
            // Diminue progressivement le niveau d'eau (simulation géologique)
            self.water_level *= self.definition.water_decay_per_second(); // Très lente diminution
            // Met à jour les températures
            self.temperature_map.reset_to_ambient();
            for y in 0..self.size {
                for x in 0..self.size {
                    if let Entity::HeatSource(intensity) = self.entities[y][x] {
                        self.temperature_map.apply_heat_source(x, y, intensity);
                    }
                }
            }
            // Ajoute de nouvelles ressources
            if rng.random_bool(self.definition.resource_spread_probability()) {
                self.add_random_resources(rng);
            }
        }
    }

    /// Cherche parmi les quatre voisins, avec départage aléatoire des réserves égales.
    pub(crate) fn foraging_destination(
        &self,
        x: usize,
        y: usize,
        maintenance: &[f32],
        individual_rate: f32,
        rng: &mut impl rand::Rng,
    ) -> Option<(usize, usize)> {
        use rand::seq::SliceRandom;
        let mut neighbors = [
            ((x + 1) % self.size, y),
            ((x + self.size - 1) % self.size, y),
            (x, (y + 1) % self.size),
            (x, (y + self.size - 1) % self.size),
        ];
        neighbors.shuffle(rng);
        // Budget du prochain intervalle de décision (une seconde), par unité d'entretien.
        let score = |nx: usize, ny: usize, demand: f32| {
            let renewal = if matches!(self.get_entity(nx, ny), Entity::Food(_)) {
                self.definition.food_regeneration_per_second()
            } else {
                0.0
            };
            (self.food_energy(nx, ny) + renewal) / demand.max(individual_rate)
        };
        // Chaque candidat n'est évalué qu'une fois ; à score égal, le dernier de l'ordre
        // mélangé l'emporte, comme `Iterator::max_by`.
        let mut best: Option<((usize, usize), f32)> = None;
        for (nx, ny) in neighbors {
            if (nx, ny) == (x, y) || !self.is_passable(nx, ny) {
                continue;
            }
            let value = score(nx, ny, maintenance[ny * self.size + nx] + individual_rate);
            if best.is_none_or(|(_, current)| value.total_cmp(&current).is_ge()) {
                best = Some(((nx, ny), value));
            }
        }
        let (destination, destination_score) = best?;
        let current_score = score(x, y, maintenance[y * self.size + x]);
        (current_score < 1.0 || destination_score > current_score).then_some(destination)
    }

    pub(crate) fn is_passable(&self, x: usize, y: usize) -> bool {
        match self.get_entity(x, y) {
            Entity::Obstacle(terrain) => terrain.is_passable(),
            _ => true,
        }
    }

    /// Point de départ accessible le plus proche du centre, sans modifier le terrain.
    pub(crate) fn initial_position(&self) -> Option<(usize, usize)> {
        let center = self.size / 2;
        (0..self.size)
            .flat_map(|y| (0..self.size).map(move |x| (x, y)))
            .filter(|&(x, y)| self.is_passable(x, y))
            .min_by_key(|&(x, y)| {
                let dx = x.abs_diff(center);
                let dy = y.abs_diff(center);
                dx.min(self.size - dx) + dy.min(self.size - dy)
            })
    }

    /// Retire seulement l'énergie effectivement ingérée ; le site reste renouvelable.
    pub(crate) fn consume_food(&mut self, x: usize, y: usize, requested: f32) -> f32 {
        if let Entity::Food(energy) = &mut self.entities[y][x] {
            let consumed = energy.min(requested.max(0.0));
            *energy -= consumed;
            consumed
        } else {
            0.0
        }
    }

    pub(crate) fn food_energy(&self, x: usize, y: usize) -> f32 {
        match self.get_entity(x, y) {
            Entity::Food(energy) => *energy,
            _ => 0.0,
        }
    }

    pub(crate) fn get_temperature(&self, x: usize, y: usize) -> f32 {
        self.temperature_map.get(x, y)
    }

    pub(crate) fn get_entity(&self, x: usize, y: usize) -> &Entity {
        &self.entities[y][x]
    }

    pub(crate) fn get_environment_type(&self, x: usize, y: usize) -> Environment {
        if self.water_level > 0.5 || matches!(self.entities[y][x], Entity::Water(_)) {
            Environment::Water
        } else {
            Environment::Land
        }
    }
}
