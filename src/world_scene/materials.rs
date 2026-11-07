use crate::definitions::{Appearance, MaterialKind};
use crate::world_scene::{gradient::Gradient, pixel_cache::PixelCache};
use fyrox::{
    core::{algebra::Vector3, color::Color},
    material::{Material, MaterialResource},
    resource::texture::TextureResource,
};
use std::collections::BTreeMap;
use uuid::Uuid;

/// Nombre de paliers de couleur des dégradés : assez fin pour que l'énergie ou la
/// température se lisent comme une variation continue, assez grossier pour partager
/// les matériaux entre toutes les cellules.
pub(super) const BANDS: usize = 64;

pub(super) struct Materials {
    shared: BTreeMap<MaterialKind, MaterialResource>,
    energy: Vec<MaterialResource>,
    temperature: Vec<MaterialResource>,
    adaptation: Vec<MaterialResource>,
    strategy: Vec<MaterialResource>,
    pixels: PixelCache,
}

/// Matériau standard : couleur, rugosité, métallicité, et émission si `emission > 0`.
fn material(
    pixels: &mut PixelCache,
    id: Uuid,
    color: Color,
    roughness: f32,
    metallic: f32,
    emission: f32,
) -> MaterialResource {
    let mut material = Material::standard();
    material.set_property("diffuseColor", color);
    material.set_property(
        "emissionStrength",
        Vector3::new(
            color.r as f32 / 255.0,
            color.g as f32 / 255.0,
            color.b as f32 / 255.0,
        ) * emission,
    );
    if emission > 0.0 {
        material.bind("emissionTexture", pixels.white());
    }
    material.bind("roughnessTexture", pixels.scalar(roughness));
    material.bind("metallicTexture", pixels.scalar(metallic));
    MaterialResource::new_ok(id, Default::default(), material)
}

/// Dégradé linéaire par morceaux entre des couleurs de référence ; l'émission croît avec la valeur
/// pour que les individus riches en énergie, ou chauds, brillent davantage.
fn gradient(pixels: &mut PixelCache, stops: &[[u8; 4]], glow: [f32; 2]) -> Vec<MaterialResource> {
    (0..BANDS)
        .map(|band| {
            let t = band as f32 / (BANDS - 1) as f32;
            let position = t * (stops.len() - 1) as f32;
            let lower = (position.floor() as usize).min(stops.len() - 2);
            let blend = position - lower as f32;
            let channel = |index: usize| {
                (stops[lower][index] as f32 * (1.0 - blend) + stops[lower + 1][index] as f32 * blend)
                    as u8
            };
            material(
                pixels,
                Uuid::now_v7(),
                Color::from_rgba(channel(0), channel(1), channel(2), 255),
                0.30,
                0.05,
                glow[0] + (glow[1] - glow[0]) * t,
            )
        })
        .collect()
}

/// Couleur d'un dégradé pour une valeur normalisée, utilisée pour cuire les textures de sol.
pub(super) fn temperature_color(value: f32) -> [f32; 3] {
    const STOPS: [[f32; 3]; 3] = [[24.0, 96.0, 168.0], [74.0, 178.0, 150.0], [246.0, 112.0, 55.0]];
    let position = value.clamp(0.0, 1.0) * 2.0;
    let lower = (position.floor() as usize).min(1);
    let blend = position - lower as f32;
    std::array::from_fn(|i| STOPS[lower][i] * (1.0 - blend) + STOPS[lower + 1][i] * blend)
}

impl Materials {
    pub(super) fn new(appearance: &Appearance) -> Self {
        let mut pixels = PixelCache::default();
        let shared = appearance
            .materials()
            .iter()
            .map(|(kind, def)| {
                (
                    *kind,
                    material(
                        &mut pixels,
                        def.id(),
                        Color::from_rgba(
                            def.color()[0],
                            def.color()[1],
                            def.color()[2],
                            def.color()[3],
                        ),
                        def.roughness(),
                        def.metallic(),
                        def.emission(),
                    ),
                )
            })
            .collect();
        let energy = gradient(
            &mut pixels,
            &[
                appearance.materials()[&MaterialKind::Starving].color(),
                appearance.materials()[&MaterialKind::Healthy].color(),
            ],
            [0.05, 0.55],
        );
        let temperature = gradient(
            &mut pixels,
            &[[24, 96, 168, 255], [74, 178, 150, 255], [246, 112, 55, 255]],
            [0.10, 0.45],
        );
        let adaptation = gradient(
            &mut pixels,
            &[[228, 176, 83, 255], [59, 188, 233, 255]],
            [0.20, 0.40],
        );
        // Une couleur par façon de se nourrir : ingestion, photosynthèse, thermosynthèse, prédation.
        let strategy = gradient(
            &mut pixels,
            &[
                [228, 176, 83, 255],
                [92, 200, 92, 255],
                [240, 110, 60, 255],
                [214, 60, 90, 255],
            ],
            [0.30, 0.30],
        );
        Self {
            shared,
            energy,
            temperature,
            adaptation,
            strategy,
            pixels,
        }
    }

    pub(super) fn get(&self, kind: MaterialKind) -> MaterialResource {
        self.shared[&kind].clone()
    }

    /// Palier d'un dégradé pour une valeur normalisée (hors bornes ou non finie : bord le plus proche).
    pub(super) fn band(value: f32) -> u8 {
        if value.is_nan() {
            0
        } else {
            (value.clamp(0.0, 1.0) * (BANDS - 1) as f32).round() as u8
        }
    }

    pub(super) fn gradient(&self, gradient: Gradient, band: u8) -> MaterialResource {
        let bands = match gradient {
            Gradient::Energy => &self.energy,
            Gradient::Temperature => &self.temperature,
            Gradient::Adaptation => &self.adaptation,
            Gradient::Strategy => &self.strategy,
        };
        bands[(band as usize).min(BANDS - 1)].clone()
    }

    /// Matériau texturé (couleur, relief) pour le sol et l'eau.
    pub(super) fn textured(
        &mut self,
        diffuse: Option<TextureResource>,
        color: Color,
        normal: TextureResource,
        roughness: f32,
        metallic: f32,
        emission: Option<Color>,
        tiling: f32,
    ) -> MaterialResource {
        let mut material = Material::standard();
        if let Some(diffuse) = diffuse {
            material.bind("diffuseTexture", diffuse);
        }
        material.set_property("diffuseColor", color);
        material.bind("normalTexture", normal);
        material.bind("roughnessTexture", self.pixels.scalar(roughness));
        material.bind("metallicTexture", self.pixels.scalar(metallic));
        material.set_property("texCoordScale", fyrox::core::algebra::Vector2::new(tiling, tiling));
        if let Some(color) = emission {
            material.bind("emissionTexture", self.pixels.white());
            material.set_property(
                "emissionStrength",
                Vector3::new(
                    color.r as f32 / 255.0,
                    color.g as f32 / 255.0,
                    color.b as f32 / 255.0,
                ),
            );
        }
        MaterialResource::new_ok(Uuid::now_v7(), Default::default(), material)
    }
}
