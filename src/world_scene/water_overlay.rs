use crate::hydrology::Hydrology;
use crate::persistence::ViewMode;
use crate::world_scene::{
    geometry, overlay_shader, surfaces::Surfaces, water_picture::WaterPicture,
};
use fyrox::{
    core::{algebra::Vector3, pool::Handle},
    material::{Material, MaterialResource},
    resource::texture::{
        Texture, TextureKind, TextureMagnificationFilter, TextureMinificationFilter,
        TexturePixelKind, TextureResource, TextureWrapMode,
    },
    scene::{
        graph::Graph,
        mesh::{Mesh, RenderPath},
    },
};
use std::time::{Duration, Instant};
use uuid::Uuid;

/// Altitude de la surcouche d'eau au-dessus du sol : assez pour ne pas scintiller avec lui.
const SURFACE_HEIGHT: f32 = 0.014;
/// Altitude du voile nuageux, en largeurs de site.
const CLOUD_HEIGHT: f32 = 2.6;
/// Délai minimal entre deux peintures : la météo évolue à l'échelle de la seconde simulée, pas
/// de l'image affichée.
const REPAINT: Duration = Duration::from_millis(250);

fn texture(side: u32) -> TextureResource {
    let mut texture = Texture::from_bytes(
        TextureKind::Rectangle {
            width: side,
            height: side,
        },
        TexturePixelKind::RGBA8,
        vec![0u8; (side * side * 4) as usize],
    )
    .expect("dimensions et octets cohérents");
    texture.set_s_wrap_mode(TextureWrapMode::ClampToEdge);
    texture.set_t_wrap_mode(TextureWrapMode::ClampToEdge);
    texture.set_magnification_filter(TextureMagnificationFilter::Linear);
    texture.set_minification_filter(TextureMinificationFilter::Linear);
    TextureResource::new_ok(Uuid::now_v7(), Default::default(), texture)
}

fn material(texture: &TextureResource) -> MaterialResource {
    let mut material = Material::from_shader(overlay_shader::shader());
    material.bind("diffuseTexture", texture.clone());
    MaterialResource::new_ok(Uuid::now_v7(), Default::default(), material)
}

fn write(texture: &TextureResource, pixels: &[u8]) {
    let mut resource = texture.data_ref();
    let mut data = resource.modify();
    data.data_mut().copy_from_slice(pixels);
}

/// Eau, humidité, relief ombré et nuages : deux plans translucides dont les textures suivent le
/// cycle de l'eau. Les champs sont interpolés, donc le monde reste continu.
pub(super) struct WaterOverlay {
    surface: Handle<Mesh>,
    sky: Handle<Mesh>,
    surface_texture: TextureResource,
    sky_texture: TextureResource,
    picture: WaterPicture,
    painted: Option<(u64, Instant)>,
}

impl WaterOverlay {
    pub(super) fn new(graph: &mut Graph, size: usize, extent: f32, spacing: f32, surfaces: &Surfaces) -> Self {
        let picture = WaterPicture::new(size);
        let surface_texture = texture(picture.side());
        let sky_texture = texture(picture.sky_side());
        let surface = geometry::mesh(
            graph,
            "Eau et relief",
            Uuid::now_v7(),
            Vector3::new(0.0, SURFACE_HEIGHT + extent * 0.0002, 0.0),
            Vector3::new(extent, 1.0, extent),
            surfaces.plane(),
            material(&surface_texture),
        );
        graph[surface].set_cast_shadows(false);
        graph[surface].set_render_path(RenderPath::Forward);
        let sky = geometry::mesh(
            graph,
            "Nuages",
            Uuid::now_v7(),
            Vector3::new(0.0, CLOUD_HEIGHT * spacing, 0.0),
            Vector3::new(extent, 1.0, extent),
            surfaces.plane(),
            material(&sky_texture),
        );
        graph[sky].set_cast_shadows(false);
        graph[sky].set_render_path(RenderPath::Forward);
        Self {
            surface,
            sky,
            surface_texture,
            sky_texture,
            picture,
            painted: None,
        }
    }

    /// Repeint les textures si le cycle de l'eau a avancé depuis la dernière peinture et si le
    /// délai minimal est écoulé (ou si `force`). Renvoie vrai si elles ont été repeintes.
    pub(super) fn refresh(&mut self, hydrology: &Hydrology, force: bool) -> bool {
        let now = Instant::now();
        if let Some((ticks, at)) = self.painted {
            if ticks == hydrology.ticks() || (!force && now.duration_since(at) < REPAINT) {
                return false;
            }
        }
        write(&self.surface_texture, &self.picture.surface(hydrology));
        write(&self.sky_texture, &self.picture.sky(hydrology));
        self.painted = Some((hydrology.ticks(), now));
        true
    }

    /// La vue thermique montre le sol nu ; les autres vues gardent l'eau et le ciel.
    pub(super) fn show(&self, graph: &mut Graph, mode: ViewMode) {
        let visible = mode != ViewMode::Temperature;
        graph[self.surface].set_visibility(visible);
        graph[self.sky].set_visibility(visible);
    }
}
