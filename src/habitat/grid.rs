use crate::definitions::{Definitions, WorldDefinition};
use std::sync::Arc;
use uuid::Uuid;
fn default_world() -> Arc<WorldDefinition> {
    Definitions::builtin().world().clone()
}
use crate::habitat::Entity;
use crate::habitat::Environment;
use crate::habitat::HabitatRegion;
use crate::hydrology::{Flow, GroundKind, Hydrology, INITIAL_FILL, Storm, Strike};
use crate::habitat::TemperatureMap;
use crate::habitat::Terrain;
use rand::RngExt;
use std::collections::HashMap;
use std::time::Duration;

/// Portée d'un déplacement actif, en largeurs de site.
const EXPLORATION_REACH: f32 = 1.0;
/// Distance maximale entre une mère et sa fille à la naissance, en largeurs de site.
const BIRTH_REACH: f32 = 0.5;

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
    /// Cycle de l'eau.
    hydrology: Hydrology,
    /// Électrification de l'atmosphère.
    storm: Storm,
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
            hydrology: Hydrology::default(),
            storm: Storm::new(size),
        };

        grid.initialize_environment(rng);
        grid.rebuild_indexes();
        let seed = rng.random::<u64>();
        let (ground, puddles) = grid.ground_and_puddles();
        grid.hydrology = Hydrology::new(size, seed, ground, INITIAL_FILL, &puddles);
        grid.water_level = grid.hydrology.summary().flooded();
        grid
    }

    /// Reconstruit les index dérivés de l'état persistant (identités, sites alimentaires).
    fn rebuild_indexes(&mut self) {
        let (ground, _) = self.ground_and_puddles();
        self.hydrology.set_ground(ground);
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

    /// Nature du sol de chaque site et eau stagnante initiale des sites « Water ».
    fn ground_and_puddles(&self) -> (Vec<GroundKind>, Vec<f32>) {
        let entities = self.entities.iter().flatten();
        (
            entities.clone().map(GroundKind::of).collect(),
            entities
                .map(|entity| match entity {
                    Entity::Water(depth) => depth * 0.3,
                    _ => 0.0,
                })
                .collect(),
        )
    }

    pub(crate) fn hydrology(&self) -> &Hydrology {
        &self.hydrology
    }

    /// Réserve d'un site alimentaire plein.
    pub(crate) fn food_capacity(&self) -> f32 {
        self.definition.food_capacity()
    }

    #[cfg(test)]
    pub(crate) fn with_hydrology(mut self, mut hydrology: Hydrology) -> Self {
        hydrology.set_ground(self.ground_and_puddles().0);
        self.water_level = hydrology.summary().flooded();
        self.hydrology = hydrology;
        self
    }

    #[cfg(test)]
    pub(crate) fn with_storm(mut self, storm: Storm) -> Self {
        self.storm = storm;
        self
    }

    pub(crate) fn storm(&self) -> &Storm {
        &self.storm
    }

    /// Éclairement du site `(x, y)`.
    pub(crate) fn light_at(&self, x: usize, y: usize) -> f32 {
        self.hydrology.light(x, y)
    }

    /// Courant de surface et épaisseur d'eau du site `(x, y)`.
    pub(crate) fn flow_at(&self, x: usize, y: usize) -> Flow {
        self.hydrology.flow_at(x, y)
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
        self.hydrology.verify(self.size)?;
        self.storm.verify(self.size)?;
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

    /// Avance l'environnement ; renvoie les éclairs tombés pendant le pas.
    pub(crate) fn update(&mut self, dt: Duration, rng: &mut impl rand::Rng) -> Vec<Strike> {
        let mut strikes = Vec::new();
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
            // Met à jour les températures
            self.temperature_map.reset_to_ambient();
            for y in 0..self.size {
                for x in 0..self.size {
                    if let Entity::HeatSource(intensity) = self.entities[y][x] {
                        self.temperature_map.apply_heat_source(x, y, intensity);
                    }
                }
            }
            // Cycle de l'eau : la perte lente du catalogue s'ajoute aux échanges physiques.
            self.hydrology
                .step(&self.temperature_map, self.definition.water_decay_per_second());
            self.water_level = self.hydrology.summary().flooded();
            let fresh = self.storm.step(&self.hydrology);
            for strike in &fresh {
                self.hydrology.boil(strike.x(), strike.y(), strike.energy());
                self.fertilize(strike.x(), strike.y(), strike.energy());
            }
            strikes.extend(fresh);
            // Ajoute de nouvelles ressources
            if rng.random_bool(self.definition.resource_spread_probability()) {
                self.add_random_resources(rng);
            }
        }
        strikes
    }

    /// La foudre fixe l'azote de l'air : un site nu devient un site alimentaire renouvelable, un site
    /// alimentaire est regarni. Sans cet apport extérieur la matière vivante resterait confinée aux
    /// sites du monde initial ; avec lui, les orages ouvrent de nouveaux terrains à coloniser.
    fn fertilize(&mut self, x: usize, y: usize, energy: f32) {
        let capacity = self.definition.food_capacity();
        match &mut self.entities[y][x] {
            Entity::Empty => {
                self.entities[y][x] = Entity::Food((energy * capacity).min(capacity));
                self.food_sites.push(y * self.size + x);
            }
            Entity::Food(food) => *food = (*food + energy * capacity).min(capacity),
            _ => {}
        }
    }

    /// Site de la grille technique qui contient `point` (unités de site, monde torique).
    pub(crate) fn site_at(&self, point: (f32, f32)) -> (usize, usize) {
        let extent = self.size as f32;
        let site = |value: f32| (value.rem_euclid(extent).floor() as usize).min(self.size - 1);
        (site(point.0), site(point.1))
    }

    /// Température au point exact : interpolation bilinéaire entre les centres de sites, sans
    /// palier à la frontière de deux sites.
    pub(crate) fn temperature_at(&self, point: (f32, f32)) -> f32 {
        let extent = self.size as f32;
        let (u, v) = (point.0 - 0.5, point.1 - 0.5);
        let (fx, fy) = (u.floor(), v.floor());
        let (bx, by) = (u - fx, v - fy);
        let wrap = |value: f32| (value.rem_euclid(extent) as usize).min(self.size - 1);
        let (x0, y0) = (wrap(fx), wrap(fy));
        let (x1, y1) = (wrap(fx + 1.0), wrap(fy + 1.0));
        self.get_temperature(x0, y0) * (1.0 - bx) * (1.0 - by)
            + self.get_temperature(x1, y0) * bx * (1.0 - by)
            + self.get_temperature(x0, y1) * (1.0 - bx) * by
            + self.get_temperature(x1, y1) * bx * by
    }

    /// Un déplacement ne traverse jamais un solide : le point d'arrivée et le milieu du trajet
    /// doivent être dans des sites traversables.
    pub(crate) fn can_travel(&self, from: (f32, f32), to: (f32, f32)) -> bool {
        let middle = ((from.0 + to.0) / 2.0, (from.1 + to.1) / 2.0);
        [middle, to].into_iter().all(|point| {
            let (x, y) = self.site_at(point);
            self.is_passable(x, y)
        })
    }

    /// Choisit où un organisme affamé tente sa chance : huit cap régulièrement répartis, d'orientation
    /// aléatoire, à une portée de site. Aucune direction n'est privilégiée ni interdite ; seuls les
    /// solides arrêtent le mouvement. À score égal, le hasard départage.
    pub(crate) fn exploration_destination(
        &self,
        origin: (f32, f32),
        maintenance: &[f32],
        individual_rate: f32,
        rng: &mut impl rand::Rng,
    ) -> Option<(f32, f32)> {
        use std::f32::consts::{FRAC_PI_4, TAU};
        let (x, y) = self.site_at(origin);
        let phase = rng.random_range(0.0..FRAC_PI_4);
        let first = rng.random_range(0..8);
        // Budget du prochain intervalle de décision (une seconde), par unité d'entretien.
        let score = |nx: usize, ny: usize, demand: f32| {
            let renewal = if matches!(self.get_entity(nx, ny), Entity::Food(_)) {
                self.definition.food_regeneration_per_second()
            } else {
                0.0
            };
            (self.food_energy(nx, ny) + renewal) / demand.max(individual_rate)
        };
        let mut best: Option<((f32, f32), f32)> = None;
        for turn in 0..8 {
            let angle = phase + ((first + turn) % 8) as f32 * (TAU / 8.0);
            let target = (
                (origin.0 + angle.cos() * EXPLORATION_REACH).rem_euclid(self.size as f32),
                (origin.1 + angle.sin() * EXPLORATION_REACH).rem_euclid(self.size as f32),
            );
            let (nx, ny) = self.site_at(target);
            if (nx, ny) == (x, y) || !self.can_travel(origin, target) {
                continue;
            }
            let value = score(nx, ny, maintenance[ny * self.size + nx] + individual_rate);
            if best.is_none_or(|(_, current)| value.total_cmp(&current).is_ge()) {
                best = Some((target, value));
            }
        }
        let (destination, destination_score) = best?;
        let current_score = score(x, y, maintenance[y * self.size + x]);
        (current_score < 1.0 || destination_score > current_score).then_some(destination)
    }

    /// Lieu de naissance d'une fille : à portée de sa mère, dans n'importe quelle direction. Le
    /// site voisin n'est pas interdit ; un solide ramène la fille auprès de sa mère.
    pub(crate) fn birth_point(&self, mother: (f32, f32), rng: &mut impl rand::Rng) -> (f32, f32) {
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let distance = rng.random_range(0.0..BIRTH_REACH);
        let point = (
            (mother.0 + angle.cos() * distance).rem_euclid(self.size as f32),
            (mother.1 + angle.sin() * distance).rem_euclid(self.size as f32),
        );
        if self.can_travel(mother, point) { point } else { mother }
    }

    /// Habitat réel contenant le site `(x, y)` : parcours en largeur des sites voisins (huit
    /// directions, monde torique) de même milieu et de même traversabilité.
    pub(crate) fn region_at(&self, x: usize, y: usize) -> HabitatRegion {
        let environment = self.get_environment_type(x, y);
        let passable = self.is_passable(x, y);
        let mut seen = vec![false; self.size * self.size];
        let mut queue = vec![y * self.size + x];
        seen[queue[0]] = true;
        let (mut sites, mut heat) = (0, 0.0);
        while let Some(index) = queue.pop() {
            let (cx, cy) = (index % self.size, index / self.size);
            sites += 1;
            heat += self.get_temperature(cx, cy);
            for (dx, dy) in [(0, 1), (0, 2), (1, 0), (1, 1), (1, 2), (2, 0), (2, 1), (2, 2)] {
                let nx = (cx + self.size + dx - 1) % self.size;
                let ny = (cy + self.size + dy - 1) % self.size;
                let next = ny * self.size + nx;
                if !seen[next]
                    && self.get_environment_type(nx, ny) == environment
                    && self.is_passable(nx, ny) == passable
                {
                    seen[next] = true;
                    queue.push(next);
                }
            }
        }
        HabitatRegion::new(sites, environment, passable, heat / sites as f32)
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
                // L'ancêtre est aquatique : il apparaît dans l'eau si le monde en contient.
                (
                    !self.hydrology.is_submerged(x, y),
                    dx.min(self.size - dx) + dy.min(self.size - dy),
                )
            })
    }

    /// Pose `entity` sur un site libre, ou déjà occupé par une ressource remplaçable ; renvoie faux
    /// pour un obstacle ou une source de chaleur (intervention de l'expérimentateur).
    pub(crate) fn place(&mut self, x: usize, y: usize, entity: Entity) -> bool {
        if !matches!(self.entities[y][x], Entity::Empty | Entity::Food(_) | Entity::Oxygen(_) | Entity::Water(_)) {
            return false;
        }
        self.entities[y][x] = entity;
        self.rebuild_indexes();
        true
    }

    /// Remplit toutes les réserves alimentaires.
    pub(crate) fn fill_food(&mut self) {
        let capacity = self.definition.food_capacity();
        for entity in self.entities.iter_mut().flatten() {
            if let Entity::Food(energy) = entity {
                *energy = capacity;
            }
        }
    }

    /// Ramène les réserves alimentaires à `capacity` au plus.
    pub(crate) fn clamp_food(&mut self, capacity: f32) {
        for entity in self.entities.iter_mut().flatten() {
            if let Entity::Food(energy) = entity {
                *energy = energy.min(capacity);
            }
        }
    }

    pub(crate) fn storm_mut(&mut self) -> &mut Storm {
        &mut self.storm
    }

    pub(crate) fn hydrology_mut(&mut self) -> &mut Hydrology {
        &mut self.hydrology
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
        if self.hydrology.is_submerged(x, y) {
            Environment::Water
        } else {
            Environment::Land
        }
    }
}
