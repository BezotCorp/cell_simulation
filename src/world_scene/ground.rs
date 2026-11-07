use crate::ecosystem::Simulation;
use crate::persistence::ViewMode;
use crate::world_scene::{
    baked_ground::BakedGround, geometry, materials::Materials, surfaces::Surfaces,
};
use fyrox::{
    core::{algebra::Vector3, color::Color, pool::Handle},
    material::MaterialResource,
    resource::texture::{
        Texture, TextureKind, TexturePixelKind, TextureResource, TextureWrapMode,
    },
    scene::{graph::Graph, mesh::Mesh},
};
use uuid::Uuid;

/// Sol du biotope : un unique plan texturé. L'eau, les nuages et le relief qui évolue sont
/// dessinés par `WaterOverlay` au-dessus de lui.
pub(super) struct Ground {
    surface: Handle<Mesh>,
    ecosystem: MaterialResource,
    temperature: MaterialResource,
    shown: Option<ViewMode>,
}

fn texture(side: u32, pixel: TexturePixelKind, bytes: Vec<u8>) -> TextureResource {
    let mut texture = Texture::from_bytes(
        TextureKind::Rectangle {
            width: side,
            height: side,
        },
        pixel,
        bytes,
    )
    .expect("dimensions et octets cohérents");
    texture.set_s_wrap_mode(TextureWrapMode::ClampToEdge);
    texture.set_t_wrap_mode(TextureWrapMode::ClampToEdge);
    TextureResource::new_ok(Uuid::now_v7(), Default::default(), texture)
}

impl Ground {
    pub(super) fn new(
        graph: &mut Graph,
        simulation: &Simulation,
        materials: &mut Materials,
        surfaces: &Surfaces,
    ) -> Self {
        let appearance = simulation.definitions().appearance();
        let size = simulation.grid().size();
        let extent = size as f32 * appearance.habitat_spacing();
        let baked = BakedGround::bake(simulation);
        let side = baked.side();
        let normal = texture(side, TexturePixelKind::RGBA8, baked.normal().to_vec());
        let build = |materials: &mut Materials, pixels: &[u8]| {
            materials.textured(
                Some(texture(side, TexturePixelKind::SRGBA8, pixels.to_vec())),
                Color::WHITE,
                normal.clone(),
                0.82,
                0.0,
                None,
                1.0,
            )
        };
        let ecosystem = build(materials, baked.ecosystem());
        let temperature = build(materials, baked.temperature());

        let surface = geometry::mesh(
            graph,
            "Sol du biotope",
            Uuid::now_v7(),
            Vector3::zeros(),
            Vector3::new(extent, 1.0, extent),
            surfaces.plane(),
            ecosystem.clone(),
        );
        graph[surface].set_cast_shadows(false);
        Self {
            surface,
            ecosystem,
            temperature,
            shown: None,
        }
    }

    /// Applique la vue courante : couleurs du sol selon le mode.
    pub(super) fn show(&mut self, graph: &mut Graph, mode: ViewMode) {
        if self.shown == Some(mode) {
            return;
        }
        graph[self.surface].surfaces_mut()[0].set_material(if mode == ViewMode::Temperature {
            self.temperature.clone()
        } else {
            self.ecosystem.clone()
        });
        self.shown = Some(mode);
    }
}
