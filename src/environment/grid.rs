use crate::environment::Entity;
use crate::environment::TemperatureMap;
use rand::RngExt;
use std::time::Instant;
use crate::environment::Terrain;

pub(crate) struct Grid {
    size: usize,
    entities: Vec<Vec<Entity>>,
    temperature_map: TemperatureMap,
    last_update: Instant,
    water_level: f32, // Niveau d'eau global (diminue progressivement)
}

impl Grid {
    pub(crate) fn new(size: usize) -> Self {
        let mut grid = Self {
            size,
            entities: vec![vec![Entity::Empty; size]; size],
            temperature_map: TemperatureMap::new(size),
            last_update: Instant::now(),
            water_level: 1.0, // Commence complètement submergé
        };

        grid.initialize_environment();
        grid
    }

    pub(crate) fn size(&self) -> usize {
        self.size
    }

    pub(crate) fn entities(&self) -> &Vec<Vec<Entity>> {
        &self.entities
    }

    pub(crate) fn temperature_map(&self) -> &TemperatureMap {
        &self.temperature_map
    }

    pub(crate) fn last_update(&self) -> Instant {
        self.last_update
    }

    pub(crate) fn water_level(&self) -> f32 {
        self.water_level
    }

    fn initialize_environment(&mut self) {
        let mut rng = rand::rng();

        // Crée le terrain de base
        for y in 0..self.size {
            for x in 0..self.size {
                if rng.random_bool(0.1) {
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
        for _ in 0..3 {
            let x = rng.random_range(0..self.size);
            let y = rng.random_range(0..self.size);
            if let Entity::Empty = self.entities[y][x] {
                let intensity = rng.random_range(5.0..15.0);
                self.entities[y][x] = Entity::HeatSource(intensity);
                self.temperature_map.apply_heat_source(x, y, intensity);
            }
        }

        // Ajoute des ressources
        self.add_random_resources();
    }

    fn add_random_resources(&mut self) {
        let mut rng = rand::rng();

        // Ajoute de la nourriture
        for _ in 0..5 {
            let x = rng.random_range(0..self.size);
            let y = rng.random_range(0..self.size);
            if let Entity::Empty = self.entities[y][x] {
                self.entities[y][x] = Entity::Food(20.0);
            }
        }

        // Ajoute de l'oxygène et des minéraux
        for y in 0..self.size {
            for x in 0..self.size {
                if let Entity::Empty = self.entities[y][x] {
                    if rng.random_bool(0.05) {
                        self.entities[y][x] = Entity::Oxygen(rng.random_range(0.5..1.0));
                    } else if rng.random_bool(0.05) {
                        self.entities[y][x] = Entity::Water(rng.random_range(0.5..1.0));
                    }
                }
            }
        }
    }

    pub(crate) fn update(&mut self) {
        if self.last_update.elapsed().as_secs_f32() >= 1.0 {
            // Diminue progressivement le niveau d'eau (simulation géologique)
            self.water_level *= 0.9999; // Très lente diminution
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
            let mut rng = rand::rng();
            if rng.random_bool(0.1) {
                self.add_random_resources();
            }
            self.last_update = Instant::now();
        }
    }

    pub(crate) fn get_temperature(&self, x: usize, y: usize) -> f32 {
        self.temperature_map.get(x, y)
    }

    pub(crate) fn get_entity(&self, x: usize, y: usize) -> &Entity {
        &self.entities[y][x]
    }

    pub(crate) fn remove_entity(&mut self, x: usize, y: usize) {
        self.entities[y][x] = Entity::Empty;
    }

    pub(crate) fn get_environment_type(&self, x: usize, y: usize) -> &str {
        if self.water_level > 0.5 {
            "water"
        } else {
            match self.entities[y][x] {
                Entity::Water(_) => "water",
                _ => "land",
            }
        }
    }
}
