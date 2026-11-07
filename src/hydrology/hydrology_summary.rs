/// Bilan instantané du cycle de l'eau, pour l'interface et les vérifications.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct HydrologySummary {
    flooded: f32,
    raining: f32,
    cloud_cover: f32,
    mean_depth: f32,
    max_speed: f32,
    water: f64,
    solids: f64,
}

impl HydrologySummary {
    pub(super) fn new(
        flooded: f32,
        raining: f32,
        cloud_cover: f32,
        mean_depth: f32,
        max_speed: f32,
        water: f64,
        solids: f64,
    ) -> Self {
        Self {
            flooded,
            raining,
            cloud_cover,
            mean_depth,
            max_speed,
            water,
            solids,
        }
    }

    /// Part des sites submergés.
    pub(crate) fn flooded(&self) -> f32 {
        self.flooded
    }

    /// Part des sites sous la pluie.
    pub(crate) fn raining(&self) -> f32 {
        self.raining
    }

    /// Part des sites couverts de nuages.
    pub(crate) fn cloud_cover(&self) -> f32 {
        self.cloud_cover
    }

    pub(crate) fn mean_depth(&self) -> f32 {
        self.mean_depth
    }

    /// Plus fort courant de surface, en sites par seconde.
    pub(crate) fn max_speed(&self) -> f32 {
        self.max_speed
    }

    /// Eau totale (surface, sol, vapeur, nuages), en hauteur d'eau cumulée sur le monde.
    pub(crate) fn water(&self) -> f64 {
        self.water
    }

    #[cfg(test)]
    /// Matière solide totale (relief et sédiments en suspension), conservée par le transport.
    pub(crate) fn solids(&self) -> f64 {
        self.solids
    }
}
