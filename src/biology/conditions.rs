use crate::genetics::Signals;
use crate::habitat::Environment;

/// Ce qu'un organisme perçoit et reçoit de son site pendant un pas : le monde extérieur, vu par
/// la biologie.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Conditions {
    temperature: f32,
    warmth: f32,
    light: f32,
    environment: Environment,
    food_available: f32,
    food_scale: f32,
    crowding: f32,
}

/// Excès de température rapporté à 1 pour le signal de chaleur.
const WARMTH_SCALE: f32 = 30.0;
/// Nombre de voisins dans un site rapporté à 1 pour le signal de densité.
const CROWDING_SCALE: f32 = 8.0;

impl Conditions {
    /// `warmth` est l'excès de température sur l'ambiante, `light` la lumière réellement reçue
    /// (déjà partagée entre les occupants du site), `food_scale` la réserve d'un site plein et
    /// `neighbors` le nombre d'autres organismes dans le site.
    pub(crate) fn new(
        temperature: f32,
        warmth: f32,
        light: f32,
        environment: Environment,
        food_available: f32,
        food_scale: f32,
        neighbors: f32,
    ) -> Self {
        Self {
            temperature,
            warmth,
            light,
            environment,
            food_available,
            food_scale,
            crowding: neighbors.max(0.0) / CROWDING_SCALE,
        }
    }

    pub(crate) fn temperature(&self) -> f32 {
        self.temperature
    }

    pub(crate) fn warmth(&self) -> f32 {
        self.warmth
    }

    pub(crate) fn light(&self) -> f32 {
        self.light
    }

    pub(crate) fn environment(&self) -> Environment {
        self.environment
    }

    pub(crate) fn food_available(&self) -> f32 {
        self.food_available
    }

    /// Signaux perçus par les gènes d'un organisme qui a `energy` (en part de sa capacité) et
    /// `maturity` (en part de sa maturité).
    pub(crate) fn signals(&self, energy: f32, maturity: f32) -> Signals {
        Signals::new([
            self.light,
            self.warmth / WARMTH_SCALE,
            if self.environment == Environment::Water { 1.0 } else { 0.0 },
            self.food_available / self.food_scale.max(f32::EPSILON),
            self.crowding,
            energy,
            maturity,
        ])
    }
}
