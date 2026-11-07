use crate::persistence::CameraState;

#[test]
fn dragging_pans_the_ground_like_grabbing_it() {
    // Caméra au sud (+z) regardant vers le nord (-z) : la droite de l'écran est +x.
    let mut camera = CameraState::default();
    camera.focus([0.0; 3], 20.0);
    camera.orbit(camera.yaw() / 0.006, 0.0);
    assert!(camera.yaw().abs() < 1e-5);
    camera.pan(10.0, 0.0);
    assert!(camera.target()[0] < 0.0, "glisser à droite : le monde suit, la cible va à gauche");
    camera.focus([0.0; 3], 20.0);
    camera.pan(0.0, 10.0);
    assert!(camera.target()[2] < 0.0, "glisser vers le bas : la cible avance vers le nord");
    assert_eq!(camera.target()[1], 0.0, "le déplacement reste au sol");
    // Quel que soit le cap, un glissement vertical ne déplace jamais latéralement.
    for yaw in [-2.0_f32, -0.65, 1.1, 2.6] {
        let mut camera = CameraState::default();
        camera.focus([0.0; 3], 20.0);
        camera.orbit((camera.yaw() - yaw) / 0.006, 0.0);
        let forward = [-camera.yaw().sin(), -camera.yaw().cos()];
        camera.pan(0.0, 25.0);
        let moved = [camera.target()[0], camera.target()[2]];
        let along = moved[0] * forward[0] + moved[1] * forward[1];
        let across = moved[0] * forward[1] - moved[1] * forward[0];
        assert!(along > 0.0 && across.abs() < 1e-4 * along, "{yaw}: {moved:?}");
    }
}
