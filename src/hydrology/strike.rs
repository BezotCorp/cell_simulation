/// Un éclair : le site touché, l'énergie libérée et l'instant (en secondes simulées).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Strike {
    x: usize,
    y: usize,
    energy: f32,
    second: u64,
}

impl Strike {
    pub(super) fn new(x: usize, y: usize, energy: f32, second: u64) -> Self {
        Self {
            x,
            y,
            energy,
            second,
        }
    }

    pub(crate) fn x(&self) -> usize {
        self.x
    }

    pub(crate) fn y(&self) -> usize {
        self.y
    }

    /// Énergie libérée, en unités de charge.
    pub(crate) fn energy(&self) -> f32 {
        self.energy
    }

    pub(crate) fn second(&self) -> u64 {
        self.second
    }
}
