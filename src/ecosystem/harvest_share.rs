/// Partage d'une ressource captée sur place (lumière, chaleur) entre les occupants d'un site.
/// Chaque site offre une capacité finie, exprimée en « organismes à expression pleine » : au-delà,
/// les occupants se font de l'ombre et chacun reçoit la part proportionnelle à sa demande. C'est ce
/// qui borne la croissance des organismes qui ne se nourrissent pas de réserves.
#[derive(Clone, Default)]
pub(crate) struct HarvestShare {
    size: usize,
    demand: Vec<f32>,
    capacity: f32,
}

impl HarvestShare {
    pub(crate) fn new(capacity: f32) -> Self {
        Self {
            size: 0,
            demand: Vec::new(),
            capacity,
        }
    }

    pub(crate) fn reset(&mut self, size: usize) {
        self.size = size;
        self.demand.clear();
        self.demand.resize(size * size, 0.0);
    }

    pub(crate) fn request(&mut self, x: usize, y: usize, expression: f32) {
        self.demand[y * self.size + x] += expression.max(0.0);
    }

    /// Fraction de sa ressource que reçoit chaque occupant du site `(x, y)`.
    pub(crate) fn scale(&self, x: usize, y: usize) -> f32 {
        let demand = self.demand[y * self.size + x];
        if demand <= self.capacity {
            1.0
        } else {
            self.capacity / demand
        }
    }
}
