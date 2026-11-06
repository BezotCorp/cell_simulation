use crate::cells::Cell;
use crate::environment::{Entity, Grid};
use crate::simulation_engine::SimulationStats;
use std::vec::Vec;

pub(crate) struct Simulation {
    grid: Grid,
    cells: Vec<Cell>,
    generation: u32,
}

impl Simulation {
    pub(crate) fn new(size: usize) -> Self {
        let mut cells = Vec::new();
        cells.push(Cell::new(size / 2, size / 2));

        Self {
            grid: Grid::new(size),
            cells,
            generation: 0,
        }
    }

    pub(crate) fn cells(&self) -> &Vec<Cell> {
        &self.cells
    }

    pub(crate) fn grid(&self) -> &Grid {
        &self.grid
    }

    pub(crate) fn generation(&self) -> u32 {
        self.generation
    }

    pub(crate) fn update(&mut self) {
        // Met à jour l'environnement
        self.grid.update();

        // Collecte les états de l'environnement dont nous aurons besoin
        let mut env_states: Vec<(String, f32)> = Vec::new();
        for cell in &self.cells {
            let env_type = self
                .grid
                .get_environment_type(cell.x(), cell.y())
                .to_string();
            let temp = self.grid.get_temperature(cell.x(), cell.y());
            env_states.push((env_type, temp));
        }

        // Collecte les positions avec de la nourriture
        let mut food_positions: Vec<(usize, usize, usize)> = Vec::new();
        for (i, cell) in self.cells.iter().enumerate() {
            if let Entity::Food(_) = self.grid.get_entity(cell.x(), cell.y()) {
                food_positions.push((i, cell.x(), cell.y()));
            }
        }

        // Applique la consommation de nourriture
        for (cell_idx, x, y) in food_positions {
            if let Some(cell) = self.cells.get_mut(cell_idx) {
                if let Entity::Food(energy) = self.grid.get_entity(x, y) {
                    cell.feed(*energy);
                    self.grid.remove_entity(x, y);
                }
            }
        }

        // Met à jour les cellules avec leurs états d'environnement respectifs
        let mut new_cells = Vec::new();
        for (cell, (env_type, temp)) in self.cells.iter_mut().zip(env_states.iter()) {
            cell.update(temp.clone(), &env_type, self.grid.size());

            // Essaie la division cellulaire
            if let Some(new_cell) = cell.try_divide() {
                new_cells.push(new_cell);
                self.generation += 1;
            }
        }

        // Ajoute les nouvelles cellules
        self.cells.extend(new_cells);

        // Supprime les cellules mortes
        self.cells.retain(|cell| cell.energy() > 0.0);
    }

    pub(crate) fn get_stats(&self) -> SimulationStats {
        SimulationStats::new(
            self.cells.len(),
            self.generation,
            self.grid.water_level(),
            if self.cells.is_empty() {
                0.0
            } else {
                self.cells.iter().map(|c| c.energy()).sum::<f32>() / self.cells.len() as f32
            },
        )
    }
}
