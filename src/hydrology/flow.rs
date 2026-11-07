use crate::hydrology::water_physics::BODY_DEPTH;

/// Courant de surface d'un site, en sites par seconde, et épaisseur d'eau qui le porte.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Flow {
    east: f32,
    south: f32,
    depth: f32,
}

impl Flow {
    pub(crate) fn new(east: f32, south: f32, depth: f32) -> Self {
        Self { east, south, depth }
    }

    pub(crate) fn east(&self) -> f32 {
        self.east
    }

    pub(crate) fn south(&self) -> f32 {
        self.south
    }

    pub(crate) fn depth(&self) -> f32 {
        self.depth
    }

    pub(crate) fn speed(&self) -> f32 {
        self.east.hypot(self.south)
    }

    /// Part du corps d'un organisme immergée : une lame mince ne l'entraîne que partiellement.
    pub(crate) fn wetted(&self) -> f32 {
        (self.depth / BODY_DEPTH).clamp(0.0, 1.0)
    }
}
