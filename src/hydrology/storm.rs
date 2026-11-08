use crate::hydrology::{Hydrology, Strike, water_physics::*};
use serde::{Deserialize, Serialize};

/// Électrification de l'atmosphère. Les nuages se chargent par les collisions que provoque la
/// précipitation (charge ∝ pluie × masse de nuage), la charge voyage avec eux et fuit — beaucoup
/// plus vite dans l'air clair. Quand elle dépasse le seuil de claquage, une décharge relie le nuage
/// au point le plus haut du voisinage : le tonnerre n'est que la conséquence de cet éclair.
#[derive(Clone, Default, Serialize, Deserialize)]
pub(crate) struct Storm {
    charge: Vec<f32>,
}

fn unit(seed: u64, second: u64, site: u64) -> f32 {
    let mut z = seed
        ^ second.wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ site.wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    ((z ^ (z >> 31)) >> 40) as f32 / (1u64 << 24) as f32
}

impl Storm {
    pub(crate) fn new(size: usize) -> Self {
        Self {
            charge: vec![0.0; size * size],
        }
    }

    pub(crate) fn charge(&self, x: usize, y: usize, size: usize) -> f32 {
        self.charge.get(y * size + x).copied().unwrap_or(0.0)
    }

    pub(crate) fn verify(&self, size: usize) -> Result<(), String> {
        if self.charge.len() != size * size
            || self.charge.iter().any(|value| !value.is_finite() || *value < 0.0)
        {
            return Err("État orageux invalide.".into());
        }
        Ok(())
    }

    /// Ajoute de la charge à un site (intervention) : `factor` fois le seuil de claquage.
    pub(crate) fn overcharge(&mut self, x: usize, y: usize, size: usize, factor: f32) {
        if self.charge.len() != size * size {
            self.charge = vec![0.0; size * size];
        }
        self.charge[y * size + x] += BREAKDOWN * factor.max(0.0);
    }

    pub(crate) fn total(&self) -> f32 {
        self.charge.iter().sum()
    }

    /// Une seconde d'orage, d'après l'état du cycle de l'eau déjà avancé. Les décharges sont
    /// tirées d'une fonction de hachage de la graine, de l'instant et du site : le rejeu est exact
    /// sans toucher au générateur de la simulation.
    pub(crate) fn step(&mut self, hydrology: &Hydrology) -> Vec<Strike> {
        let size = hydrology.size();
        let len = size * size;
        if self.charge.len() != len {
            self.charge = vec![0.0; len];
        }
        for index in 0..len {
            let (x, y) = (index % size, index / size);
            self.charge[index] += CHARGE_RATE * hydrology.rain(x, y) * hydrology.cloud(x, y);
        }
        self.charge = hydrology.carry(&self.charge);

        let mut strikes = Vec::new();
        let second = hydrology.ticks();
        for index in 0..len {
            let (x, y) = (index % size, index / size);
            let cloud = hydrology.cloud(x, y);
            let leak = CHARGE_LEAK + if cloud < CLOUD_CRITICAL_FOR_LEAK { CLEAR_AIR_LEAK } else { 0.0 };
            let charge = self.charge[index];
            if charge > BREAKDOWN {
                let chance = ((charge - BREAKDOWN) / BREAKDOWN).min(1.0) * STRIKE_PROBABILITY;
                if unit(hydrology.seed(), second, index as u64) < chance {
                    let (tx, ty) = Self::target(hydrology, x, y);
                    let released = charge * STRIKE_RELEASE;
                    self.charge[index] = charge - released;
                    strikes.push(Strike::new(tx, ty, released, second));
                    continue;
                }
            }
            self.charge[index] = charge * (1.0 - leak.min(1.0));
        }
        strikes
    }

    /// La foudre frappe le point le plus élevé autour du nuage : les reliefs attirent l'éclair.
    fn target(hydrology: &Hydrology, x: usize, y: usize) -> (usize, usize) {
        let size = hydrology.size();
        let mut best = (x, y);
        let mut height = f32::MIN;
        for dy in [size - 1, 0, 1] {
            for dx in [size - 1, 0, 1] {
                let (nx, ny) = ((x + dx) % size, (y + dy) % size);
                let elevation = hydrology.elevation(nx, ny) + hydrology.depth(nx, ny);
                if elevation > height {
                    height = elevation;
                    best = (nx, ny);
                }
            }
        }
        best
    }
}
