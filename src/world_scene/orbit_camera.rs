use crate::application::CellLab;
use fyrox::{
    core::{
        algebra::{UnitQuaternion, Vector3},
        reflect::prelude::*,
        type_traits::prelude::*,
        visitor::prelude::*,
    },
    plugin::error::GameResult,
    script::{ScriptContext, ScriptTrait},
};

/// Camera behaviour is a native Fyrox script; persistent settings live once in the session.
#[derive(Visit, Reflect, Clone, Default, Debug, TypeUuidProvider, ComponentProvider)]
#[type_uuid(id = "019a59d8-0000-7000-8000-000000000040")]
pub(crate) struct OrbitCamera;
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
