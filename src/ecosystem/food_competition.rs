use crate::habitat::Grid;

/// Répartition simultanée d'une réserve finie entre les déficits des occupants.
#[derive(Clone, Default)]
pub(crate) struct FoodCompetition {
    size: usize,
    demand: Vec<f32>,
    available: Vec<f32>,
}

impl FoodCompetition {
    /// Remet les demandes à zéro en réutilisant les tampons existants.
    pub(crate) fn reset(&mut self, size: usize) {
        self.size = size;
        for buffer in [&mut self.demand, &mut self.available] {
            buffer.clear();
            buffer.resize(size * size, 0.0);
        }
    }

    pub(crate) fn size(&self) -> usize {
        self.size
    }

    pub(crate) fn demand(&self, x: usize, y: usize) -> f32 {
        self.demand[y * self.size + x]
    }

    pub(crate) fn available(&self, x: usize, y: usize) -> f32 {
        self.available[y * self.size + x]
    }

    pub(crate) fn request(&mut self, x: usize, y: usize, deficit: f32) {
        self.demand[y * self.size + x] += deficit.max(0.0);
    }

    pub(crate) fn consume_from(&mut self, grid: &mut Grid) {
        for (index, demand) in self.demand.iter().copied().enumerate() {
            if demand > 0.0 {
                self.available[index] =
                    grid.consume_food(index % self.size(), index / self.size(), demand);
            }
        }
    }

    pub(crate) fn share(&self, x: usize, y: usize, deficit: f32) -> f32 {
        let demand = self.demand(x, y);
        if demand == 0.0 {
            return 0.0;
        }
        deficit.max(0.0) * (self.available(x, y) / demand).min(1.0)
    }
}
