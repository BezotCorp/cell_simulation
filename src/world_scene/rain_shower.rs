use crate::definitions::MaterialKind;
use crate::hydrology::Hydrology;
use crate::world_scene::{
    geometry::{self, mesh},
    materials::Materials,
};
use fyrox::{
    core::{algebra::Vector3, pool::Handle},
    scene::{graph::Graph, mesh::Mesh},
};
use uuid::Uuid;

/// Nombre de gouttes disponibles pour toute la scène.
const DROPS: usize = 640;
/// Au plus ce nombre d'entrées dans la table des sites pluvieux.
const MAX_SITES: usize = 4_096;
/// Hauteur de chute, en largeurs de site.
const FALL_HEIGHT: f32 = 4.0;

fn unit(seed: u32) -> f32 {
    let mut z = seed.wrapping_mul(0x9E37_79B1).wrapping_add(0x7F4A_7C15);
    z = (z ^ (z >> 15)).wrapping_mul(0x85EB_CA77);
    z = (z ^ (z >> 13)).wrapping_mul(0xC2B2_AE3D);
    ((z ^ (z >> 16)) >> 8) as f32 / (1u32 << 24) as f32
}

/// Pluie locale : un lot de gouttes réutilisées qui tombent uniquement sur les sites où le cycle
/// de l'eau précipite, en proportion de l'intensité.
pub(super) struct RainShower {
    drops: Vec<Handle<Mesh>>,
    /// Sites pluvieux, répétés selon l'intensité.
    sites: Vec<u32>,
    size: usize,
    spacing: f32,
    shown: bool,
}

impl RainShower {
    pub(super) fn new(graph: &mut Graph, materials: &Materials, size: usize, spacing: f32) -> Self {
        let drops = (0..DROPS)
            .map(|_| {
                let drop = mesh(
                    graph,
                    "Goutte",
                    Uuid::now_v7(),
                    Vector3::zeros(),
                    Vector3::new(0.006, 0.16, 0.006) * spacing,
                    geometry::cube(),
                    materials.get(MaterialKind::Oxygen),
                );
                graph[drop].set_cast_shadows(false);
                graph[drop].set_visibility(false);
                drop
            })
            .collect();
        Self {
            drops,
            sites: Vec::new(),
            size,
            spacing,
            shown: false,
        }
    }

    /// Recense les sites sous la pluie d'après le dernier pas hydrologique.
    pub(super) fn refresh(&mut self, hydrology: &Hydrology) {
        self.sites.clear();
        for index in 0..self.size * self.size {
            let rain = hydrology.rain(index % self.size, index / self.size);
            if rain > 2e-4 {
                let weight = ((rain / 0.002).ceil() as usize).clamp(1, 6);
                for _ in 0..weight {
                    if self.sites.len() < MAX_SITES {
                        self.sites.push(index as u32);
                    }
                }
            }
        }
    }

    /// Fait tomber les gouttes ; `visible` est faux dans la vue thermique.
    pub(super) fn animate(&mut self, graph: &mut Graph, time: f32, visible: bool) {
        let active = visible && !self.sites.is_empty();
        if !active {
            if self.shown {
                for drop in &self.drops {
                    graph[*drop].set_visibility(false);
                }
                self.shown = false;
            }
            return;
        }
        self.shown = true;
        let half = self.size as f32 / 2.0;
        let height = FALL_HEIGHT * self.spacing;
        for (index, drop) in self.drops.iter().enumerate() {
            let speed = 1.1 + 0.5 * unit(index as u32 * 7 + 1);
            let clock = time * speed + unit(index as u32 * 13 + 2);
            let cycle = clock.floor() as u32;
            let fall = clock.fract();
            let seed = (index as u32).wrapping_mul(2_654_435_761).wrapping_add(cycle.wrapping_mul(40_503));
            let site = self.sites[(unit(seed) * self.sites.len() as f32) as usize % self.sites.len()] as usize;
            let (x, y) = (
                (site % self.size) as f32 + unit(seed ^ 0x55),
                (site / self.size) as f32 + unit(seed ^ 0xAA),
            );
            let node = &mut graph[*drop];
            node.set_visibility(true);
            node.local_transform_mut().set_position(Vector3::new(
                (x - half) * self.spacing,
                height * (1.0 - fall) + 0.05,
                (y - half) * self.spacing,
            ));
        }
    }
}
