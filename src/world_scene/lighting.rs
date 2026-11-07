use fyrox::{
    core::{
        algebra::{UnitQuaternion, Vector3},
        color::Color,
    },
    scene::{
        base::BaseBuilder,
        graph::Graph,
        light::{BaseLightBuilder, directional::DirectionalLightBuilder, point::PointLightBuilder},
        transform::TransformBuilder,
    },
};

/// Éclairage du biotope : un soleil chaud rasant qui projette des ombres, et un contre-jour
/// froid proportionnel à la taille du monde, hors du plan de la grille. L'ambiance vient du
/// ciel et de la lumière ambiante de la scène.
pub(super) fn add(graph: &mut Graph, extent: f32) {
    DirectionalLightBuilder::new(
        BaseLightBuilder::new(
            BaseBuilder::new()
                .with_name("Lumière principale")
                .with_local_transform(
                    TransformBuilder::new()
                        .with_local_rotation(UnitQuaternion::from_euler_angles(-0.95, -0.55, 0.0))
                        .build(),
                ),
        )
        .with_color(Color::opaque(255, 226, 190))
        .with_scatter_enabled(false)
        .with_intensity(2.0),
    )
    .build(graph);
    PointLightBuilder::new(
        BaseLightBuilder::new(
            BaseBuilder::new()
                .with_name("Contre-jour froid")
                .with_local_transform(
                    TransformBuilder::new()
                        .with_local_position(Vector3::new(
                            -extent * 0.55,
                            extent * 0.45 + 2.0,
                            -extent * 0.40,
                        ))
                        .build(),
                ),
        )
        .with_color(Color::opaque(80, 160, 235))
        .with_scatter_enabled(false)
        .with_intensity(4.0),
    )
    .with_radius(extent * 2.4 + 8.0)
    .build(graph);
}
