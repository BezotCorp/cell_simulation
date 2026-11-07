/// Vent régional, fonction déterministe du temps simulé et de la graine du monde : sa direction
/// dérive lentement et sa force varie. Il transporte la vapeur et les nuages.
#[derive(Debug, Clone, Copy)]
pub(super) struct Wind {
    heading: f32,
    swing: f32,
    strength: f32,
}

impl Wind {
    pub(super) fn new(seed: u64) -> Self {
        let unit = |salt: u64| {
            let mut z = seed.wrapping_add(salt.wrapping_mul(0x9E37_79B9_7F4A_7C15));
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            ((z ^ (z >> 31)) >> 40) as f32 / (1u64 << 24) as f32
        };
        Self {
            heading: unit(1) * std::f32::consts::TAU,
            swing: unit(3) * std::f32::consts::TAU,
            strength: unit(4) * std::f32::consts::TAU,
        }
    }

    /// Vitesse du vent (est, sud), en sites par seconde, à l'instant `seconds`.
    pub(super) fn at(&self, seconds: f32) -> (f32, f32) {
        let heading =
            self.heading + 0.0012 * seconds + 0.9 * (0.0031 * seconds + self.swing).sin();
        let speed = 0.16 + 0.10 * (0.5 + 0.5 * (0.0017 * seconds + self.strength).sin());
        (heading.cos() * speed, heading.sin() * speed)
    }
}
