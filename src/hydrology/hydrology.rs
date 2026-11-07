use crate::habitat::TemperatureMap;
use crate::hydrology::{Flow, GroundKind, HydrologySummary, water_physics::*, wind::Wind};
use serde::{Deserialize, Serialize};

/// Cycle de l'eau et modelage du relief. Les champs sont portés par les sites de la grille
/// technique, mais les phénomènes (pluie, courant, érosion) sont continus dans l'espace : ils
/// ne dépendent que des différences de hauteur, de température et de saturation entre voisins.
///
/// Toutes les quantités d'eau sont des hauteurs d'eau équivalentes par site. Le système est
/// fermé, sauf la perte géologique lente demandée par le catalogue (`retention`) : l'eau se
/// conserve entre surface, sol, vapeur et nuages, la matière entre relief et sédiments.
#[derive(Clone, Default, Serialize, Deserialize)]
pub(crate) struct Hydrology {
    size: usize,
    seed: u64,
    ticks: u64,
    elevation: Vec<f32>,
    depth: Vec<f32>,
    moisture: Vec<f32>,
    sediment: Vec<f32>,
    vapor: Vec<f32>,
    cloud: Vec<f32>,
    current_east: Vec<f32>,
    current_south: Vec<f32>,
    /// Intensité de pluie du dernier pas : dérivée, utilisée par le rendu seulement.
    #[serde(skip)]
    rain: Vec<f32>,
    /// Nature du sol : dérivée du contenu des sites, réinstallée à la restauration.
    #[serde(skip)]
    ground: Vec<GroundKind>,
}

fn mix(seed: u64, a: i64, b: i64, octave: u32) -> f32 {
    let mut z = seed
        ^ (a as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (b as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
        ^ u64::from(octave).wrapping_mul(0x1656_67B1_9E37_79F9);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    ((z ^ (z >> 31)) >> 40) as f32 / (1u64 << 24) as f32
}

/// Bruit de valeur périodique : le relief se raccorde sur lui-même, comme le monde torique.
fn periodic_noise(seed: u64, octave: u32, cells: usize, u: f32, v: f32) -> f32 {
    let (x, y) = (u * cells as f32, v * cells as f32);
    let (fx, fy) = (x.floor(), y.floor());
    let smooth = |t: f32| t * t * (3.0 - 2.0 * t);
    let (tx, ty) = (smooth(x - fx), smooth(y - fy));
    let wrap = |value: f32| (value as i64).rem_euclid(cells as i64);
    let at = |dx: f32, dy: f32| mix(seed, wrap(fx + dx), wrap(fy + dy), octave);
    let top = at(0.0, 0.0) * (1.0 - tx) + at(1.0, 0.0) * tx;
    let bottom = at(0.0, 1.0) * (1.0 - tx) + at(1.0, 1.0) * tx;
    top * (1.0 - ty) + bottom * ty
}

fn saturation(temperature: f32) -> f32 {
    SATURATION_REFERENCE * ((temperature - 20.0) / SATURATION_SCALE).exp()
}

impl Hydrology {
    /// Monde neuf : relief périodique issu de la graine, eau à `fill` des sites non solides et
    /// `puddles` (hauteur d'eau ajoutée par site), puis quelques écoulements qui l'installent
    /// dans les bassins.
    pub(crate) fn new(
        size: usize,
        seed: u64,
        ground: Vec<GroundKind>,
        fill: f32,
        puddles: &[f32],
    ) -> Self {
        let len = size * size;
        let mut elevation = vec![0.0_f32; len];
        let mut octaves = 0;
        let mut cells = 2;
        let mut amplitude = 1.0;
        let mut total = 0.0;
        while octaves < 5 && (octaves == 0 || cells * 2 <= size.max(4)) {
            for (index, height) in elevation.iter_mut().enumerate() {
                let (u, v) = (
                    ((index % size) as f32 + 0.5) / size as f32,
                    ((index / size) as f32 + 0.5) / size as f32,
                );
                *height += amplitude * periodic_noise(seed, octaves, cells, u, v);
            }
            total += amplitude;
            amplitude *= 0.5;
            cells *= 2;
            octaves += 1;
        }
        let (low, high) = elevation
            .iter()
            .fold((f32::MAX, f32::MIN), |(l, h), e| (l.min(*e), h.max(*e)));
        for (height, kind) in elevation.iter_mut().zip(&ground) {
            *height = RELIEF * (*height - low) / (high - low).max(total * 1e-3);
            if kind.is_solid() {
                *height += SOLID_RAISE;
            }
        }

        let mut open: Vec<f32> = elevation
            .iter()
            .zip(&ground)
            .filter(|(_, kind)| !kind.is_solid())
            .map(|(height, _)| *height)
            .collect();
        open.sort_by(f32::total_cmp);
        let level = if open.is_empty() {
            0.0
        } else if fill >= 1.0 {
            open[open.len() - 1] + 0.02
        } else {
            open[((fill.clamp(0.0, 1.0) * open.len() as f32) as usize).min(open.len() - 1)]
        };
        let depth: Vec<f32> = elevation
            .iter()
            .zip(&ground)
            .enumerate()
            .map(|(index, (height, kind))| {
                let basin = if kind.is_solid() { 0.0 } else { (level - height).max(0.0) };
                basin + puddles.get(index).copied().unwrap_or(0.0)
            })
            .collect();
        let moisture = depth
            .iter()
            .zip(&ground)
            .map(|(d, kind)| kind.porosity() * if *d > WET_DEPTH { 1.0 } else { 0.4 })
            .collect();
        let vapor = vec![0.5 * saturation(20.0); len];

        let mut hydrology = Self {
            size,
            seed,
            ticks: 0,
            elevation,
            depth,
            moisture,
            sediment: vec![0.0; len],
            vapor,
            cloud: vec![0.0; len],
            current_east: vec![0.0; len],
            current_south: vec![0.0; len],
            rain: vec![0.0; len],
            ground,
        };
        for _ in 0..SETTLING_TICKS {
            hydrology.flow(false);
        }
        hydrology
    }

    /// Anciennes sauvegardes : un niveau d'eau global `level` (part des sites submergés) et la
    /// nature des sites servent à reconstruire un état hydrologique cohérent.
    pub(crate) fn from_legacy(
        size: usize,
        seed: u64,
        ground: Vec<GroundKind>,
        level: f32,
        puddles: &[f32],
    ) -> Self {
        Self::new(size, seed, ground, level, puddles)
    }

    pub(crate) fn size(&self) -> usize {
        self.size
    }

    pub(crate) fn ticks(&self) -> u64 {
        self.ticks
    }

    /// Réinstalle la nature des sites après une restauration.
    pub(crate) fn set_ground(&mut self, ground: Vec<GroundKind>) {
        self.ground = ground;
        if self.rain.len() != self.size * self.size {
            self.rain = vec![0.0; self.size * self.size];
        }
    }

    /// Contrôle d'un état restauré : dimensions et valeurs physiquement admissibles.
    pub(crate) fn verify(&self, size: usize) -> Result<(), String> {
        let len = size * size;
        let fields = [
            &self.elevation,
            &self.depth,
            &self.moisture,
            &self.sediment,
            &self.vapor,
            &self.cloud,
            &self.current_east,
            &self.current_south,
        ];
        if self.size != size || fields.iter().any(|field| field.len() != len) {
            return Err("État hydrologique invalide : dimensions.".into());
        }
        if fields.iter().any(|field| field.iter().any(|value| !value.is_finite())) {
            return Err("État hydrologique invalide : valeur non finie.".into());
        }
        if [&self.depth, &self.moisture, &self.sediment, &self.vapor, &self.cloud]
            .iter()
            .any(|field| field.iter().any(|value| *value < 0.0))
        {
            return Err("État hydrologique invalide : quantité négative.".into());
        }
        if self
            .current_east
            .iter()
            .zip(&self.current_south)
            .any(|(x, y)| x.hypot(*y) > MAX_SPEED * 1.001)
        {
            return Err("État hydrologique invalide : courant excessif.".into());
        }
        Ok(())
    }

    fn index(&self, x: usize, y: usize) -> usize {
        y * self.size + x
    }

    pub(crate) fn elevation(&self, x: usize, y: usize) -> f32 {
        self.elevation[self.index(x, y)]
    }

    pub(crate) fn depth(&self, x: usize, y: usize) -> f32 {
        self.depth[self.index(x, y)]
    }

    pub(crate) fn moisture(&self, x: usize, y: usize) -> f32 {
        self.moisture[self.index(x, y)]
    }

    pub(crate) fn sediment(&self, x: usize, y: usize) -> f32 {
        self.sediment[self.index(x, y)]
    }

    /// Saturation du sol en eau, de 0 (sec) à 1 (détrempé) ; 0 sur la roche.
    pub(crate) fn wetness(&self, x: usize, y: usize) -> f32 {
        let index = self.index(x, y);
        let porosity = self.ground.get(index).map_or(0.0, |kind| kind.porosity());
        if porosity > 0.0 { (self.moisture[index] / porosity).clamp(0.0, 1.0) } else { 0.0 }
    }

    /// Part de matière en suspension dans l'eau du site, de 0 (limpide) à 1.
    pub(crate) fn turbidity(&self, x: usize, y: usize) -> f32 {
        let index = self.index(x, y);
        (self.sediment[index] / (self.depth[index] + 0.05) * 30.0).clamp(0.0, 1.0)
    }

    /// Éclairement du site `(x, y)`, de 0 à 1 : le jour, voilé par les nuages, puis éteint par
    /// l'eau d'autant plus qu'elle est profonde et chargée de sédiments.
    pub(crate) fn light(&self, x: usize, y: usize) -> f32 {
        let index = self.index(x, y);
        let day = (0.65 + 0.9 * (std::f32::consts::TAU * self.ticks as f32 / DAY_SECONDS).sin())
            .clamp(0.0, 1.0);
        let veil = 1.0 - (self.cloud[index] * CLOUD_OPACITY).min(0.85);
        let murk = WATER_CLARITY + WATER_MUD * self.turbidity(x, y);
        day * veil * (-self.depth[index] * murk).exp()
    }

    pub(crate) fn seed(&self) -> u64 {
        self.seed
    }

    /// Un éclair vaporise l'eau de surface du site : l'eau passe dans l'air, rien n'est perdu.
    pub(crate) fn boil(&mut self, x: usize, y: usize, energy: f32) {
        let index = self.index(x, y);
        let boiled = (energy * FLASH_BOIL).min(self.depth[index]);
        self.depth[index] -= boiled;
        self.vapor[index] += boiled;
    }

    /// Transporte un champ (charges électriques, par exemple) avec le vent courant, comme les nuages.
    pub(crate) fn carry(&self, field: &[f32]) -> Vec<f32> {
        let (east, south) = Wind::new(self.seed).at(self.ticks as f32);
        self.advect(field, east, south)
    }

    pub(crate) fn cloud(&self, x: usize, y: usize) -> f32 {
        self.cloud[self.index(x, y)]
    }

    pub(crate) fn vapor(&self, x: usize, y: usize) -> f32 {
        self.vapor[self.index(x, y)]
    }

    pub(crate) fn rain(&self, x: usize, y: usize) -> f32 {
        self.rain.get(self.index(x, y)).copied().unwrap_or(0.0)
    }

    pub(crate) fn flow_at(&self, x: usize, y: usize) -> Flow {
        let index = self.index(x, y);
        Flow::new(
            self.current_east[index],
            self.current_south[index],
            self.depth[index],
        )
    }

    /// État d'essai : une lame d'eau de `depth` parcourue par un courant uniforme.
    #[cfg(test)]
    pub(crate) fn with_uniform_current(mut self, east: f32, south: f32, depth: f32) -> Self {
        self.depth.iter_mut().for_each(|d| *d = depth);
        self.current_east.iter_mut().for_each(|c| *c = east);
        self.current_south.iter_mut().for_each(|c| *c = south);
        self
    }

    /// Un site est submergé quand l'eau y dépasse [`WET_DEPTH`].
    pub(crate) fn is_submerged(&self, x: usize, y: usize) -> bool {
        self.depth[self.index(x, y)] >= WET_DEPTH
    }

    pub(crate) fn summary(&self) -> HydrologySummary {
        let len = (self.size * self.size).max(1) as f32;
        let count = |field: &[f32], threshold: f32| {
            field.iter().filter(|value| **value >= threshold).count() as f32 / len
        };
        let max_speed = self
            .current_east
            .iter()
            .zip(&self.current_south)
            .map(|(x, y)| x.hypot(*y))
            .fold(0.0, f32::max);
        let sum = |field: &[f32]| field.iter().map(|value| f64::from(*value)).sum::<f64>();
        HydrologySummary::new(
            count(&self.depth, WET_DEPTH),
            count(&self.rain, 1e-5),
            count(&self.cloud, CLOUD_CRITICAL * 0.5),
            sum(&self.depth) as f32 / len,
            max_speed,
            sum(&self.depth) + sum(&self.moisture) + sum(&self.vapor) + sum(&self.cloud),
            sum(&self.elevation) + sum(&self.sediment),
        )
    }

    fn neighbors(&self, index: usize) -> [usize; 4] {
        let n = self.size;
        let (x, y) = (index % n, index / n);
        [
            y * n + (x + 1) % n,
            y * n + (x + n - 1) % n,
            ((y + 1) % n) * n + x,
            ((y + n - 1) % n) * n + x,
        ]
    }

    /// Une seconde simulée : atmosphère, infiltration, écoulement et modelage du relief, puis la
    /// perte géologique `retention` (fraction conservée par seconde) sur l'eau de surface et du sol.
    pub(crate) fn step(&mut self, temperature: &TemperatureMap, retention: f32) {
        self.atmosphere(temperature);
        self.infiltrate();
        self.flow(true);
        for index in 0..self.depth.len() {
            // Les erreurs d'arrondi des transferts ne doivent jamais créer d'eau négative.
            self.depth[index] = (self.depth[index] * retention).max(0.0);
            self.moisture[index] = (self.moisture[index] * retention).max(0.0);
            self.sediment[index] = self.sediment[index].max(0.0);
        }
        self.ticks += 1;
    }

    /// Évaporation, transport par le vent, condensation et précipitations.
    fn atmosphere(&mut self, temperature: &TemperatureMap) {
        let n = self.size;
        let len = n * n;
        let (wind_east, wind_south) = Wind::new(self.seed).at(self.ticks as f32);
        let saturation: Vec<f32> = (0..len)
            .map(|index| {
                saturation(
                    temperature.get(index % n, index / n) - LAPSE_RATE * (self.elevation[index] - 0.5),
                )
            })
            .collect();

        for index in 0..len {
            let deficit = (saturation[index] - self.vapor[index]).max(0.0);
            let wet = (self.depth[index] / WET_DEPTH).min(1.0);
            let porosity = self.ground[index].porosity();
            let soaked = if porosity > 0.0 {
                (self.moisture[index] / porosity).min(1.0)
            } else {
                0.0
            };
            let from_surface = (EVAPORATION * deficit * wet).min(self.depth[index]);
            let from_soil = (EVAPORATION * SOIL_EVAPORATION * deficit * (1.0 - wet) * soaked)
                .min(self.moisture[index]);
            self.depth[index] -= from_surface;
            self.moisture[index] -= from_soil;
            self.vapor[index] += from_surface + from_soil;
        }

        self.vapor = self.advect(&self.vapor, wind_east, wind_south);
        self.cloud = self.advect(&self.cloud, wind_east, wind_south);

        self.rain.resize(len, 0.0);
        for index in 0..len {
            if self.vapor[index] > saturation[index] {
                let condensed = (self.vapor[index] - saturation[index]) * CONDENSATION;
                self.vapor[index] -= condensed;
                self.cloud[index] += condensed;
            } else {
                let evaporated =
                    ((saturation[index] - self.vapor[index]) * CLOUD_EVAPORATION).min(self.cloud[index]);
                self.cloud[index] -= evaporated;
                self.vapor[index] += evaporated;
            }
            // Les gouttes grossissent avec la densité du nuage : les gros nuages précipitent bien
            // plus vite que les voiles minces.
            let excess = (self.cloud[index] - CLOUD_CRITICAL).max(0.0);
            let rain = (RAIN_FRACTION * excess * (1.0 + 6.0 * excess / CLOUD_CRITICAL)).min(self.cloud[index]);
            self.cloud[index] -= rain;
            self.depth[index] += rain;
            self.rain[index] = rain;
        }
    }

    /// Transport amont conservatif par le vent, avec une légère diffusion vers les voisins.
    fn advect(&self, field: &[f32], east: f32, south: f32) -> Vec<f32> {
        let mut result = vec![0.0_f32; field.len()];
        let keep = 1.0 - east.abs() - south.abs() - 4.0 * AIR_DIFFUSION;
        for (index, value) in field.iter().enumerate() {
            let [e, w, s, nn] = self.neighbors(index);
            result[index] += value * keep;
            result[if east > 0.0 { e } else { w }] += value * east.abs();
            result[if south > 0.0 { s } else { nn }] += value * south.abs();
            for neighbor in [e, w, s, nn] {
                result[neighbor] += value * AIR_DIFFUSION;
            }
        }
        result
    }

    /// Infiltration de l'eau de surface dans le sol, et exfiltration de l'excès de saturation.
    fn infiltrate(&mut self) {
        for index in 0..self.depth.len() {
            let kind = self.ground[index];
            let capacity = kind.porosity();
            if capacity <= 0.0 {
                continue;
            }
            let room = (capacity - self.moisture[index]).max(0.0);
            let infiltrated = (INFILTRATION * kind.permeability() * room).min(self.depth[index]);
            self.depth[index] -= infiltrated;
            self.moisture[index] += infiltrated;
            if self.moisture[index] > capacity {
                let excess = (self.moisture[index] - capacity) * EXFILTRATION;
                self.moisture[index] -= excess;
                self.depth[index] += excess;
            }
        }
    }

    /// Écoulement de surface vers les voisins plus bas, transport des sédiments, arrachement et
    /// dépôt. La surface libre `élévation + profondeur` gouverne tout : le relief modifié
    /// redirige aussitôt l'eau.
    fn flow(&mut self, reshape: bool) {
        let len = self.depth.len();
        let surface: Vec<f32> = (0..len).map(|i| self.elevation[i] + self.depth[i]).collect();
        let mut outgoing = vec![[0.0_f32; 4]; len];
        for index in 0..len {
            if self.depth[index] <= DRY_DEPTH {
                continue;
            }
            let neighbors = self.neighbors(index);
            let mut wanted = [0.0_f32; 4];
            for (slot, neighbor) in neighbors.iter().enumerate() {
                wanted[slot] = FLOW_COEFFICIENT * (surface[index] - surface[*neighbor]).max(0.0);
            }
            let total: f32 = wanted.iter().sum();
            if total > 0.0 {
                let scale = (self.depth[index] / total).min(1.0);
                for slot in 0..4 {
                    outgoing[index][slot] = wanted[slot] * scale;
                }
            }
        }

        let mut depth = self.depth.clone();
        let mut sediment = self.sediment.clone();
        let mut net_east = vec![0.0_f32; len];
        let mut net_south = vec![0.0_f32; len];
        for index in 0..len {
            let neighbors = self.neighbors(index);
            let released: f32 = outgoing[index].iter().sum();
            if released <= 0.0 {
                continue;
            }
            // La concentration de sédiments suit l'eau qui part.
            let concentration = self.sediment[index] / self.depth[index];
            depth[index] -= released;
            sediment[index] -= concentration * released;
            for (slot, neighbor) in neighbors.iter().enumerate() {
                let amount = outgoing[index][slot];
                depth[*neighbor] += amount;
                sediment[*neighbor] += concentration * amount;
                match slot {
                    0 => net_east[index] += amount,
                    1 => net_east[*neighbor] -= amount,
                    2 => net_south[index] += amount,
                    _ => net_south[*neighbor] -= amount,
                }
            }
        }
        // Les arrondis des transferts ne doivent jamais créer d'eau ni de matière négative.
        self.depth = depth.into_iter().map(|value| value.max(0.0)).collect();
        self.sediment = sediment.into_iter().map(|value| value.max(0.0)).collect();

        for index in 0..len {
            let [_, west, _, north] = self.neighbors(index);
            let discharge_east = 0.5 * (net_east[index] + net_east[west]);
            let discharge_south = 0.5 * (net_south[index] + net_south[north]);
            let scale = 1.0 / (self.depth[index] + SPEED_DEPTH);
            let (mut east, mut south) = (discharge_east * scale, discharge_south * scale);
            let speed = east.hypot(south);
            if speed > MAX_SPEED {
                east *= MAX_SPEED / speed;
                south *= MAX_SPEED / speed;
            }
            self.current_east[index] = east;
            self.current_south[index] = south;
        }

        if reshape {
            self.reshape();
        }
    }

    /// Arrache de la matière là où le courant dépasse la capacité de transport, la dépose
    /// ailleurs. La somme `relief + sédiments` reste constante.
    fn reshape(&mut self) {
        for index in 0..self.depth.len() {
            let speed = self.current_east[index].hypot(self.current_south[index]);
            let capacity = if self.depth[index] > DRY_DEPTH {
                TRANSPORT_CAPACITY * speed * speed
            } else {
                0.0
            };
            let held = self.sediment[index];
            if held < capacity {
                let eroded = (EROSION_RATE * (capacity - held) * self.ground[index].erodibility())
                    .min(MAX_RELIEF_CHANGE);
                self.elevation[index] -= eroded;
                self.sediment[index] += eroded;
            } else {
                let deposited = if capacity == 0.0 {
                    held
                } else {
                    (DEPOSITION_RATE * (held - capacity)).min(held)
                };
                self.elevation[index] += deposited;
                self.sediment[index] -= deposited;
            }
        }
    }
}
