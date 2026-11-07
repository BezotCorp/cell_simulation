use crate::definitions::{Appearance, MaterialKind};
use fyrox::{
    core::{algebra::Vector3, color::Color},
    material::{Material, MaterialResource},
    resource::texture::{Texture, TextureKind, TexturePixelKind, TextureResource},
};
use std::collections::{BTreeMap, HashMap};
use uuid::Uuid;

/// Nombre de paliers de couleur des dégradés : assez fin pour que l'énergie ou la
/// température se lisent comme une variation continue, assez grossier pour partager
/// les matériaux entre toutes les cellules.
pub(crate) const BANDS: usize = 64;

/// Grandeur biologique ou physique représentée par un dégradé.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Gradient {
    Energy,
    Temperature,
    Adaptation,
}

pub(crate) struct Materials {
    shared: BTreeMap<MaterialKind, MaterialResource>,
    energy: Vec<MaterialResource>,
    temperature: Vec<MaterialResource>,
    adaptation: Vec<MaterialResource>,
}

/// Textures 1×1 partagées : une par valeur de rugosité ou de métallicité distincte.
#[derive(Default)]
struct PixelCache(HashMap<u8, TextureResource>);

impl PixelCache {
    fn get(&mut self, value: f32) -> TextureResource {
        let byte = (value.clamp(0.0, 1.0) * 255.0) as u8;
        self.0
            .entry(byte)
            .or_insert_with(|| {
                let texture = Texture::from_bytes(
                    TextureKind::Rectangle {
                        width: 1,
                        height: 1,
                    },
                    TexturePixelKind::R8,
                    vec![byte],
                )
                .expect("une texture 1×1 est toujours valide");
                TextureResource::new_ok(Uuid::now_v7(), Default::default(), texture)
            })
            .clone()
    }
}

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
    material.bind("roughnessTexture", pixels.get(roughness));
    material.bind("metallicTexture", pixels.get(metallic));
    MaterialResource::new_ok(id, Default::default(), material)
}

/// Dégradé linéaire par morceaux entre des couleurs de référence.
fn gradient(pixels: &mut PixelCache, stops: &[[u8; 4]]) -> Vec<MaterialResource> {
    (0..BANDS)
        .map(|band| {
            let position = band as f32 / (BANDS - 1) as f32 * (stops.len() - 1) as f32;
            let lower = (position.floor() as usize).min(stops.len() - 2);
            let t = position - lower as f32;
            let channel = |index: usize| {
                (stops[lower][index] as f32 * (1.0 - t) + stops[lower + 1][index] as f32 * t) as u8
            };
            material(
                pixels,
                Uuid::now_v7(),
                Color::from_rgba(channel(0), channel(1), channel(2), 255),
                0.38,
                0.05,
                0.12,
            )
        })
        .collect()
}

impl Materials {
    pub(crate) fn new(appearance: &Appearance) -> Self {
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
        Self {
            shared,
            energy: gradient(
                &mut pixels,
                &[
                    appearance.materials()[&MaterialKind::Starving].color(),
                    appearance.materials()[&MaterialKind::Healthy].color(),
                ],
            ),
            temperature: gradient(
                &mut pixels,
                &[[24, 96, 168, 255], [74, 178, 150, 255], [246, 112, 55, 255]],
            ),
            adaptation: gradient(&mut pixels, &[[228, 176, 83, 255], [59, 188, 233, 255]]),
        }
    }

    pub(crate) fn get(&self, kind: MaterialKind) -> MaterialResource {
        self.shared[&kind].clone()
    }

    /// Palier d'un dégradé pour une valeur normalisée (hors bornes ou non finie : bord le plus proche).
    pub(crate) fn band(value: f32) -> u8 {
        if value.is_nan() {
            0
        } else {
            (value.clamp(0.0, 1.0) * (BANDS - 1) as f32).round() as u8
        }
    }

    pub(crate) fn gradient(&self, gradient: Gradient, band: u8) -> MaterialResource {
        let bands = match gradient {
            Gradient::Energy => &self.energy,
            Gradient::Temperature => &self.temperature,
            Gradient::Adaptation => &self.adaptation,
        };
        bands[(band as usize).min(BANDS - 1)].clone()
    }
}
