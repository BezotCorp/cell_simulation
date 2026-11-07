/// Un éclair en cours d'affichage : son point d'impact (plan du monde), l'instant de sa
/// naissance en secondes d'affichage et la graine de son tracé.
#[derive(Debug, Clone, Copy)]
pub(super) struct ActiveBolt {
    east: f32,
    south: f32,
    born: f32,
    seed: u32,
}

impl ActiveBolt {
    pub(super) fn new(east: f32, south: f32, born: f32, seed: u32) -> Self {
        Self {
            east,
            south,
            born,
            seed,
        }
    }

    pub(super) fn east(&self) -> f32 {
        self.east
    }

    pub(super) fn south(&self) -> f32 {
        self.south
    }

    pub(super) fn born(&self) -> f32 {
        self.born
    }

    pub(super) fn seed(&self) -> u32 {
        self.seed
    }
}
