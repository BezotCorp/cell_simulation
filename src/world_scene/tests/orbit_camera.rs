use crate::world_scene::OrbitCamera;
use fyrox::{
    core::algebra::{UnitQuaternion, Vector2, Vector3},
    scene::{base::BaseBuilder, camera::CameraBuilder, graph::Graph},
};

/// Projette `point` à l'écran d'une caméra placée en `eye` qui regarde `target`.
fn project(
    eye: Vector3<f32>,
    target: Vector3<f32>,
    point: Vector3<f32>,
    size: Vector2<f32>,
) -> Option<Vector2<f32>> {
    let mut graph = Graph::default();
    let camera = CameraBuilder::new(BaseBuilder::new())
        .with_projection(OrbitCamera::projection(2_000.0))
        .build(&mut graph);
    graph[camera]
        .local_transform_mut()
        .set_position(eye)
        .set_rotation(UnitQuaternion::face_towards(&(target - eye), &Vector3::y()));
    graph.update_hierarchical_data();
    graph[camera].calculate_matrices(size);
    let clip = graph[camera].view_projection_matrix() * point.push(1.0);
    (clip.w > 0.0).then(|| {
        Vector2::new(
            (clip.x / clip.w * 0.5 + 0.5) * size.x,
            (1.0 - (clip.y / clip.w * 0.5 + 0.5)) * size.y,
        )
    })
}

fn eye(target: [f32; 3], distance: f32, yaw: f32, pitch: f32) -> Vector3<f32> {
    Vector3::from(target)
        + Vector3::new(yaw.sin() * pitch.cos(), pitch.sin(), yaw.cos() * pitch.cos()) * distance
}

#[test]
fn framing_keeps_the_whole_biotope_in_view_for_any_window_shape() {
    let (yaw, pitch) = (-0.65_f32, 0.82_f32);
    for extent in [10.0_f32, 64.0, 128.0] {
        for (width, height) in [(1100.0, 780.0), (700.0, 900.0), (2000.0, 600.0), (1000.0, 1000.0)] {
            let size = Vector2::new(width, height);
            let (target, distance) = OrbitCamera::framing(extent, width / height, yaw, pitch);
            assert!(distance <= 300.0, "la caméra du jeu est limitée à 300 : {distance}");
            let (eye, target) = (eye(target, distance, yaw, pitch), Vector3::from(target));
            let half = extent / 2.0 + 0.35;
            for (x, z) in [(-half, -half), (half, -half), (-half, half), (half, half)] {
                let at = project(eye, target, Vector3::new(x, 0.0, z), size).expect("devant la caméra");
                assert!(
                    at.x >= 0.0 && at.x <= width && at.y >= 0.0 && at.y <= height,
                    "coin ({x}, {z}) hors champ : {at:?} dans {width}×{height} (étendue {extent})"
                );
            }
        }
    }
    // Plus la zone est étroite, plus la caméra doit reculer.
    assert!(
        OrbitCamera::framing(10.0, 0.6, yaw, pitch).1 > OrbitCamera::framing(10.0, 1.6, yaw, pitch).1
    );
}

#[test]
fn framing_is_tight_and_balanced() {
    let (yaw, pitch) = (-0.65_f32, 0.82_f32);
    let size = Vector2::new(1100.0, 780.0);
    let (target, distance) = OrbitCamera::framing(10.0, size.x / size.y, yaw, pitch);
    let (eye, target) = (eye(target, distance, yaw, pitch), Vector3::from(target));
    let corners: Vec<Vector2<f32>> = [(-5.35, -5.35), (5.35, -5.35), (-5.35, 5.35), (5.35, 5.35)]
        .iter()
        .map(|&(x, z)| project(eye, target, Vector3::new(x, 0.0, z), size).unwrap())
        .collect();
    let (top, bottom) = (
        corners.iter().map(|c| c.y).fold(f32::MAX, f32::min),
        corners.iter().map(|c| c.y).fold(f32::MIN, f32::max),
    );
    // Marges haute et basse comparables, et biotope occupant l'essentiel de la hauteur.
    assert!(((top) - (size.y - bottom)).abs() < 0.12 * size.y, "{top} / {bottom}");
    let (left, right) = (
        corners.iter().map(|c| c.x).fold(f32::MAX, f32::min),
        corners.iter().map(|c| c.x).fold(f32::MIN, f32::max),
    );
    let fill = ((bottom - top) / size.y).max((right - left) / size.x);
    assert!(fill > 0.5, "biotope trop petit à l'écran : {fill}");
}
