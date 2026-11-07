use crate::definitions::MaterialKind;
use crate::world_scene::{
    active_bolt::ActiveBolt,
    geometry::{self, mesh},
    materials::Materials,
    surfaces::Surfaces,
};
use fyrox::{
    core::{
        algebra::{UnitQuaternion, Vector3},
        color::Color,
        pool::Handle,
    },
    scene::{
        base::BaseBuilder,
        graph::Graph,
        light::{BaseLightBuilder, point::{PointLight, PointLightBuilder}},
        mesh::Mesh,
        transform::TransformBuilder,
    },
};
use uuid::Uuid;

/// Éclairs affichés en même temps, segments par éclair et durées d'affichage (secondes réelles).
const BOLTS: usize = 4;
const SEGMENTS: usize = 12;
const BOLT_LIFE: f32 = 0.45;
const FLASH_LIFE: f32 = 0.18;
const THUNDER_LIFE: f32 = 1.8;
/// Vitesse apparente de l'onde du tonnerre, en largeurs de site par seconde.
const THUNDER_SPEED: f32 = 2.2;

fn unit(seed: u32) -> f32 {
    let mut z = seed.wrapping_mul(0x9E37_79B1).wrapping_add(0x7F4A_7C15);
    z = (z ^ (z >> 15)).wrapping_mul(0x85EB_CA77);
    z = (z ^ (z >> 13)).wrapping_mul(0xC2B2_AE3D);
    ((z ^ (z >> 16)) >> 8) as f32 / (1u32 << 24) as f32
}

/// Foudre : un tracé brisé du nuage au sol, un éclair de lumière, puis l'onde du tonnerre qui
/// s'étend depuis le point d'impact. L'affichage suit les éclairs de la simulation, il n'en invente pas.
pub(super) struct LightningShow {
    segments: Vec<Handle<Mesh>>,
    thunder: Vec<Handle<Mesh>>,
    flash: Vec<Handle<PointLight>>,
    active: Vec<Option<ActiveBolt>>,
    next: usize,
    spacing: f32,
    sky: f32,
}

impl LightningShow {
    pub(super) fn new(
        graph: &mut Graph,
        materials: &Materials,
        surfaces: &Surfaces,
        spacing: f32,
        sky: f32,
    ) -> Self {
        let segments = (0..BOLTS * SEGMENTS)
            .map(|_| {
                let node = mesh(
                    graph,
                    "Éclair",
                    Uuid::now_v7(),
                    Vector3::zeros(),
                    Vector3::new(0.03, 0.3, 0.03),
                    geometry::cube(),
                    materials.get(MaterialKind::Selection),
                );
                graph[node].set_cast_shadows(false);
                graph[node].set_visibility(false);
                node
            })
            .collect();
        let thunder = (0..BOLTS)
            .map(|_| {
                let node = mesh(
                    graph,
                    "Tonnerre",
                    Uuid::now_v7(),
                    Vector3::zeros(),
                    Vector3::repeat(0.1),
                    surfaces.ring(),
                    materials.get(MaterialKind::Selection),
                );
                graph[node].set_cast_shadows(false);
                graph[node].set_visibility(false);
                node
            })
            .collect();
        let flash = (0..BOLTS)
            .map(|_| {
                let light = PointLightBuilder::new(
                    BaseLightBuilder::new(
                        BaseBuilder::new()
                            .with_name("Lueur de l'éclair")
                            .with_visibility(false)
                            .with_local_transform(TransformBuilder::new().build()),
                    )
                    .with_color(Color::opaque(214, 226, 255))
                    .with_scatter_enabled(false)
                    .with_intensity(7.0),
                )
                .with_radius(7.0 * spacing)
                .build(graph);
                graph[light].set_visibility(false);
                light
            })
            .collect();
        Self {
            segments,
            thunder,
            flash,
            active: vec![None; BOLTS],
            next: 0,
            spacing,
            sky,
        }
    }

    /// Déclenche l'affichage d'un éclair tombé en `(east, south)` (plan du monde) à l'instant `now`.
    pub(super) fn strike(&mut self, east: f32, south: f32, now: f32, seed: u32) {
        self.active[self.next] = Some(ActiveBolt::new(east, south, now, seed));
        self.next = (self.next + 1) % BOLTS;
    }

    pub(super) fn animate(&mut self, graph: &mut Graph, now: f32, visible: bool) {
        for slot in 0..BOLTS {
            let bolt = self.active[slot].filter(|bolt| visible && now - bolt.born() < THUNDER_LIFE);
            if bolt.is_none() {
                self.active[slot] = None;
            }
            let age = bolt.map_or(f32::MAX, |bolt| now - bolt.born());
            let alive = bolt.is_some() && age < BOLT_LIFE;
            for part in 0..SEGMENTS {
                let node = self.segments[slot * SEGMENTS + part];
                // Le tracé vacille : il s'éteint et se rallume brièvement avant de disparaître.
                let lit = alive && (age < 0.12 || unit((age * 40.0) as u32 + slot as u32) > 0.35);
                graph[node].set_visibility(lit);
                if let (true, Some(bolt)) = (lit, bolt) {
                    let point = |index: usize| {
                        let t = index as f32 / SEGMENTS as f32;
                        let sway = (1.0 - (2.0 * t - 1.0).abs()) * 0.45 * self.spacing;
                        let jitter = |axis: u32| {
                            (unit(bolt.seed().wrapping_add(index as u32 * 31 + axis)) - 0.5) * 2.0 * sway
                        };
                        Vector3::new(
                            bolt.east() + jitter(1),
                            self.sky * (1.0 - t) + 0.04,
                            bolt.south() + jitter(2),
                        )
                    };
                    let (top, bottom) = (point(part), point(part + 1));
                    let offset = bottom - top;
                    let fade = 1.0 - age / BOLT_LIFE;
                    graph[node]
                        .local_transform_mut()
                        .set_position((top + bottom) * 0.5)
                        .set_rotation(
                            UnitQuaternion::rotation_between(&Vector3::y(), &offset)
                                .unwrap_or_else(UnitQuaternion::identity),
                        )
                        .set_scale(Vector3::new(
                            0.035 * self.spacing * (0.4 + fade),
                            offset.norm(),
                            0.035 * self.spacing * (0.4 + fade),
                        ));
                }
            }
            let glowing = bolt.is_some() && age < FLASH_LIFE;
            graph[self.flash[slot]].set_visibility(glowing);
            if let (true, Some(bolt)) = (glowing, bolt) {
                graph[self.flash[slot]]
                    .local_transform_mut()
                    .set_position(Vector3::new(bolt.east(), 1.4 * self.spacing, bolt.south()));
            }
            let spreading = bolt.is_some() && age < THUNDER_LIFE;
            graph[self.thunder[slot]].set_visibility(spreading);
            if let (true, Some(bolt)) = (spreading, bolt) {
                graph[self.thunder[slot]]
                    .local_transform_mut()
                    .set_position(Vector3::new(bolt.east(), 0.035, bolt.south()))
                    // Anneau plat qui s'étend puis s'amincit : une onde, pas un volume.
                    .set_scale(Vector3::new(
                        (age * THUNDER_SPEED * self.spacing).max(0.05),
                        0.02 * (1.0 - age / THUNDER_LIFE).max(0.0),
                        (age * THUNDER_SPEED * self.spacing).max(0.05),
                    ));
            }
        }
    }
}
