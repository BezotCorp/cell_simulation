use crate::definitions::MaterialKind;
use crate::hydrology::Hydrology;
use crate::world_scene::{
    geometry::{self, mesh},
    materials::Materials,
};
use fyrox::{
    core::{
        algebra::{UnitQuaternion, Vector3},
        pool::Handle,
    },
    scene::{graph::Graph, mesh::Mesh},
};
use uuid::Uuid;

/// Nombre de traînées disponibles pour toute la scène.
const STREAKS: usize = 220;
/// Courant minimal (sites par seconde) et eau minimale pour qu'un site en porte.
const MIN_SPEED: f32 = 0.025;
const MIN_DEPTH: f32 = 0.03;
/// Exagération de la vitesse d'affichage : un courant lent reste lisible à l'écran.
const DISPLAY_GAIN: f32 = 4.0;
/// Durée de vie d'une traînée, en secondes réelles.
const LIFETIME: f32 = 3.0;

fn unit(seed: u32) -> f32 {
    let mut z = seed.wrapping_mul(0x9E37_79B1).wrapping_add(0x7F4A_7C15);
    z = (z ^ (z >> 15)).wrapping_mul(0x85EB_CA77);
    z = (z ^ (z >> 13)).wrapping_mul(0xC2B2_AE3D);
    ((z ^ (z >> 16)) >> 8) as f32 / (1u32 << 24) as f32
}

/// Traînées claires qui dérivent le long des courants de surface. Elles montrent où l'eau
/// s'écoule et à quelle vitesse ; seul le dessin les utilise, jamais la biologie.
pub(super) struct FlowStreaks {
    nodes: Vec<Handle<Mesh>>,
    /// Position (en sites) et âge de chaque traînée.
    state: Vec<(f32, f32, f32)>,
    /// Courant est, sud et profondeur de chaque site, copiés du dernier pas hydrologique.
    east: Vec<f32>,
    south: Vec<f32>,
    depth: Vec<f32>,
    /// Sites qui portent un courant.
    flowing: Vec<u32>,
    size: usize,
    spacing: f32,
    spawned: u32,
    shown: bool,
}

impl FlowStreaks {
    pub(super) fn new(graph: &mut Graph, materials: &Materials, size: usize, spacing: f32) -> Self {
        let nodes = (0..STREAKS)
            .map(|_| {
                let node = mesh(
                    graph,
                    "Courant",
                    Uuid::now_v7(),
                    Vector3::zeros(),
                    Vector3::new(0.22, 0.004, 0.022) * spacing,
                    geometry::cube(),
                    materials.get(MaterialKind::Oxygen),
                );
                graph[node].set_cast_shadows(false);
                graph[node].set_visibility(false);
                node
            })
            .collect();
        Self {
            nodes,
            state: vec![(0.0, 0.0, f32::MAX); STREAKS],
            east: Vec::new(),
            south: Vec::new(),
            depth: Vec::new(),
            flowing: Vec::new(),
            size,
            spacing,
            spawned: 0,
            shown: false,
        }
    }

    pub(super) fn refresh(&mut self, hydrology: &Hydrology) {
        let len = self.size * self.size;
        self.east.clear();
        self.south.clear();
        self.depth.clear();
        self.flowing.clear();
        for index in 0..len {
            let flow = hydrology.flow_at(index % self.size, index / self.size);
            self.east.push(flow.east());
            self.south.push(flow.south());
            self.depth.push(flow.depth());
            if flow.speed() > MIN_SPEED && flow.depth() > MIN_DEPTH {
                self.flowing.push(index as u32);
            }
        }
    }

    pub(super) fn animate(&mut self, graph: &mut Graph, dt: f32, visible: bool) {
        if !visible || self.flowing.is_empty() {
            if self.shown {
                for node in &self.nodes {
                    graph[*node].set_visibility(false);
                }
                self.shown = false;
            }
            return;
        }
        self.shown = true;
        let half = self.size as f32 / 2.0;
        let extent = self.size as f32;
        for (slot, node) in self.nodes.iter().enumerate() {
            let (mut x, mut y, mut age) = self.state[slot];
            let site = |x: f32, y: f32| {
                (y.rem_euclid(extent) as usize).min(self.size - 1) * self.size
                    + (x.rem_euclid(extent) as usize).min(self.size - 1)
            };
            let mut here = site(x, y);
            if age >= LIFETIME
                || self.depth[here] <= MIN_DEPTH
                || self.east[here].hypot(self.south[here]) <= MIN_SPEED * 0.5
            {
                self.spawned = self.spawned.wrapping_add(1);
                let seed = self.spawned.wrapping_mul(2_246_822_519) ^ slot as u32;
                let pick = self.flowing[(unit(seed) * self.flowing.len() as f32) as usize % self.flowing.len()] as usize;
                x = (pick % self.size) as f32 + unit(seed ^ 0x33);
                y = (pick / self.size) as f32 + unit(seed ^ 0x77);
                age = unit(seed ^ 0x99) * LIFETIME * 0.5;
                here = site(x, y);
            }
            let (east, south) = (self.east[here], self.south[here]);
            x += east * DISPLAY_GAIN * dt;
            y += south * DISPLAY_GAIN * dt;
            self.state[slot] = (x, y, age + dt);
            let fade = (age / 0.4).min(1.0) * ((LIFETIME - age) / 0.4).clamp(0.0, 1.0);
            let transform = graph[*node].local_transform_mut();
            transform
                .set_position(Vector3::new(
                    (x - half) * self.spacing,
                    0.03,
                    (y - half) * self.spacing,
                ))
                .set_rotation(UnitQuaternion::from_axis_angle(&Vector3::y_axis(), -south.atan2(east)))
                .set_scale(Vector3::new(0.22, 0.004, 0.022) * self.spacing * fade.max(0.01));
            graph[*node].set_visibility(true);
        }
    }
}
