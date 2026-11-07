use crate::application::CellLab;
use fyrox::{
    core::{
        algebra::{UnitQuaternion, Vector3},
        reflect::prelude::*,
        type_traits::prelude::*,
        visitor::prelude::*,
    },
    plugin::error::GameResult,
    scene::camera::{PerspectiveProjection, Projection},
    script::{ScriptContext, ScriptTrait},
};

/// Camera behaviour is a native Fyrox script; persistent settings live once in the session.
#[derive(Visit, Reflect, Clone, Default, Debug, TypeUuidProvider, ComponentProvider)]
#[type_uuid(id = "019a59d8-0000-7000-8000-000000000040")]
pub(crate) struct OrbitCamera;

impl OrbitCamera {
    /// Champ de vision vertical de la caméra du monde, en radians.
    pub(crate) const FIELD_OF_VIEW: f32 = 48.0 * std::f32::consts::PI / 180.0;

    /// Projection du monde : la caméra du moteur ignore `with_fov` et consorts, seule la
    /// projection explicite compte.
    pub(crate) fn projection(far: f32) -> Projection {
        // Le plan proche grandit avec le monde : la précision de profondeur reste suffisante pour
        // que les surcouches posées à quelques centièmes du sol ne scintillent pas au loin.
        let near = ((far - 400.0) / 6.0 * 0.004).clamp(0.05, 0.6);
        Projection::Perspective(PerspectiveProjection {
            fov: Self::FIELD_OF_VIEW,
            z_near: near,
            z_far: far,
        })
    }

    /// Cadrage du biotope carré de côté `extent` (socle compris) dans une zone d'affichage de
    /// proportions `aspect` (largeur / hauteur), vu depuis la direction `(yaw, pitch)`, avec une
    /// marge de 6 %. Renvoie la cible (sur le sol) et la distance. Le calcul projette les huit
    /// coins du socle et recentre la cible le long de l'axe de visée jusqu'à équilibrer les marges
    /// haute et basse : à angle rasant, viser le centre laisserait le coin proche coller au bord.
    pub(crate) fn framing(extent: f32, aspect: f32, yaw: f32, pitch: f32) -> ([f32; 3], f32) {
        const MARGIN: f32 = 0.94;
        let tan_vertical = (Self::FIELD_OF_VIEW / 2.0).tan() * MARGIN;
        let tan_horizontal = tan_vertical * aspect.max(0.2);
        let toward_camera =
            Vector3::new(yaw.sin() * pitch.cos(), pitch.sin(), yaw.cos() * pitch.cos());
        let forward = -toward_camera;
        let right = forward.cross(&Vector3::y()).normalize();
        let up = right.cross(&forward);
        let ground = Vector3::new(forward.x, 0.0, forward.z).normalize();
        let half = extent / 2.0 + 0.35;
        let corners: Vec<Vector3<f32>> = [-half, half]
            .iter()
            .flat_map(|&x| [-half, half].map(move |z| (x, z)))
            .flat_map(|(x, z)| [-0.7, 0.15].map(move |y| Vector3::new(x, y, z)))
            .collect();
        // Distance nécessaire pour une cible décalée de `shift` le long de la visée au sol.
        let required = |shift: f32| {
            corners.iter().fold(2.0_f32, |distance, corner| {
                let relative = corner - ground * shift;
                let depth = relative.dot(&forward);
                distance
                    .max(relative.dot(&right).abs() / tan_horizontal - depth)
                    .max(relative.dot(&up).abs() / tan_vertical - depth)
            })
        };
        // La fonction est convexe : recherche ternaire du décalage qui minimise la distance.
        let (mut low, mut high) = (-extent, extent);
        for _ in 0..60 {
            let (a, b) = (low + (high - low) / 3.0, high - (high - low) / 3.0);
            if required(a) < required(b) {
                high = b;
            } else {
                low = a;
            }
        }
        let shift = (low + high) / 2.0;
        let target = ground * shift;
        ([target.x, 0.0, target.z], required(shift))
    }
}

impl ScriptTrait for OrbitCamera {
    fn on_update(&mut self, context: &mut ScriptContext) -> GameResult {
        let game = context.plugins.get::<CellLab>();
        if let Some(session) = game.session() {
            let camera = &session.view().camera();
            let target = Vector3::from(camera.target());
            let eye = target
                + Vector3::new(
                    camera.yaw().sin() * camera.pitch().cos(),
                    camera.pitch().sin(),
                    camera.yaw().cos() * camera.pitch().cos(),
                ) * camera.distance();
            context.scene.graph[context.handle]
                .local_transform_mut()
                .set_position(eye)
                .set_rotation(UnitQuaternion::face_towards(&(target - eye), &Vector3::y()));
        }
        Ok(())
    }
}
