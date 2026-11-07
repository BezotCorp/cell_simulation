use crate::habitat::Environment;

/// Habitat réel observé autour d'un point : l'étendue continue de sites de même milieu et de
/// même traversabilité. La grille de calcul ne le délimite pas, elle sert seulement à le mesurer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct HabitatRegion {
    sites: usize,
    environment: Environment,
    passable: bool,
    mean_temperature: f32,
}

impl HabitatRegion {
    pub(crate) fn new(
        sites: usize,
        environment: Environment,
        passable: bool,
        mean_temperature: f32,
    ) -> Self {
        Self {
            sites,
            environment,
            passable,
            mean_temperature,
        }
    }

    pub(crate) fn sites(&self) -> usize {
        self.sites
    }

    pub(crate) fn environment(&self) -> Environment {
        self.environment
    }

    pub(crate) fn passable(&self) -> bool {
        self.passable
    }

    pub(crate) fn mean_temperature(&self) -> f32 {
        self.mean_temperature
    }
}
