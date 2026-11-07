use crate::world_scene::noise::fbm3;
use fyrox::{
    core::{
        algebra::{Matrix4, Vector2, Vector3},
        math::TriangleDefinition,
    },
    scene::mesh::{
        buffer::{TriangleBuffer, VertexBuffer},
        surface::{SurfaceData, SurfaceResource},
        vertex::StaticVertex,
    },
};

/// Sphère déformée par du bruit : membranes, roches, baies. `amplitude` est la part du rayon
/// que le relief peut gagner ou perdre ; les normales sont calculées sur la forme exacte,
/// sans couture ni facettes.
fn blob(seed: u32, amplitude: f32, rings: usize, segments: usize) -> SurfaceData {
    let position = |theta: f32, phi: f32| -> Vector3<f32> {
        let direction = Vector3::new(theta.sin() * phi.cos(), theta.cos(), theta.sin() * phi.sin());
        let noise = fbm3(
            direction.x * 1.7 + 11.0,
            direction.y * 1.7 + 5.0,
            direction.z * 1.7 + 3.0,
            seed,
            3,
        );
        direction * (1.0 + amplitude * (2.0 * noise - 1.0))
    };
    let epsilon = 1e-3;
    let mut vertices = Vec::with_capacity((rings + 1) * (segments + 1));
    for i in 0..=rings {
        for j in 0..=segments {
            let theta = std::f32::consts::PI * i as f32 / rings as f32;
            let phi = std::f32::consts::TAU * j as f32 / segments as f32;
            let at = position(theta, phi);
            let along_theta = position(theta + epsilon, phi) - position(theta - epsilon, phi);
            let along_phi = position(theta, phi + epsilon) - position(theta, phi - epsilon);
            let normal = along_phi
                .cross(&along_theta)
                .try_normalize(1e-9)
                .unwrap_or_else(|| at.normalize());
            vertices.push(StaticVertex::from_pos_uv_normal(
                at,
                Vector2::new(j as f32 / segments as f32, i as f32 / rings as f32),
                normal,
            ));
        }
    }
    let mut triangles = Vec::with_capacity(rings * segments * 2);
    for i in 0..rings {
        for j in 0..segments {
            let a = (i * (segments + 1) + j) as u32;
            let (b, c) = (a + 1, a + segments as u32 + 1);
            let d = c + 1;
            // Le sens d'enroulement est choisi d'après la normale pour rester orienté vers l'extérieur.
            for tri in [[a, c, b], [b, c, d]] {
                let p = |k: u32| vertices[k as usize].position;
                let face = (p(tri[1]) - p(tri[0])).cross(&(p(tri[2]) - p(tri[0])));
                let outward = vertices[tri[0] as usize].normal;
                if face.norm_squared() > 1e-14 {
                    triangles.push(if face.dot(&outward) >= 0.0 {
                        TriangleDefinition(tri)
                    } else {
                        TriangleDefinition([tri[0], tri[2], tri[1]])
                    });
                }
            }
        }
    }
    let mut data = SurfaceData::new(
        VertexBuffer::new(vertices.len(), vertices).expect("sommets valides"),
        TriangleBuffer::new(triangles),
    );
    data.calculate_tangents().expect("tangentes calculables");
    data
}

/// Cristal à facettes : bipyramide à six faces, plus haute que large, normales par facette.
fn crystal() -> SurfaceData {
    let ring: Vec<Vector3<f32>> = (0..6)
        .map(|i| {
            let angle = std::f32::consts::TAU * i as f32 / 6.0;
            Vector3::new(angle.cos() * 0.5, 0.0, angle.sin() * 0.5)
        })
        .collect();
    let (top, bottom) = (Vector3::new(0.0, 1.0, 0.0), Vector3::new(0.0, -0.45, 0.0));
    let mut vertices = Vec::new();
    let mut triangles = Vec::new();
    for i in 0..6 {
        let (a, b) = (ring[i], ring[(i + 1) % 6]);
        for corners in [[top, b, a], [bottom, a, b]] {
            let normal = (corners[1] - corners[0])
                .cross(&(corners[2] - corners[0]))
                .normalize();
            let base = vertices.len() as u32;
            for (corner, uv) in corners.iter().zip([(0.5, 0.0), (1.0, 1.0), (0.0, 1.0)]) {
                vertices.push(StaticVertex::from_pos_uv_normal(
                    *corner,
                    Vector2::new(uv.0, uv.1),
                    normal,
                ));
            }
            triangles.push(TriangleDefinition([base, base + 1, base + 2]));
        }
    }
    let mut data = SurfaceData::new(
        VertexBuffer::new(vertices.len(), vertices).expect("sommets valides"),
        TriangleBuffer::new(triangles),
    );
    data.calculate_tangents().expect("tangentes calculables");
    data
}

/// Plan horizontal unitaire (de -0,5 à 0,5), normale vers le haut, coordonnées de texture
/// de (0, 0) au coin (-x, -z) à (1, 1) au coin (+x, +z). Les triangles sont doublés dans les
/// deux sens : il reste visible quelle que soit la face vue ou le mode de rejet du moteur.
fn plane() -> SurfaceData {
    let corners = [(-0.5, -0.5), (-0.5, 0.5), (0.5, 0.5), (0.5, -0.5)];
    let vertices: Vec<StaticVertex> = corners
        .iter()
        .map(|&(x, z)| {
            StaticVertex::from_pos_uv_normal(
                Vector3::new(x, 0.0, z),
                Vector2::new(x + 0.5, z + 0.5),
                Vector3::y(),
            )
        })
        .collect();
    let mut data = SurfaceData::new(
        VertexBuffer::new(vertices.len(), vertices).expect("sommets valides"),
        TriangleBuffer::new(vec![
            TriangleDefinition([0, 1, 2]),
            TriangleDefinition([0, 2, 3]),
            TriangleDefinition([0, 2, 1]),
            TriangleDefinition([0, 3, 2]),
        ]),
    );
    data.calculate_tangents().expect("tangentes calculables");
    data
}

/// Disque horizontal de rayon 1, normale vers le haut, doublé dans les deux sens : flaques et mares.
/// Jeu de formes partagées par toute la scène : chaque forme n'existe qu'une fois en mémoire
/// et les instances identiques sont regroupées par le moteur de rendu.
pub(super) struct Surfaces {
    cells: [SurfaceResource; 3],
    rocks: [SurfaceResource; 4],
    sphere: SurfaceResource,
    crystal: SurfaceResource,
    ring: SurfaceResource,
    plane: SurfaceResource,
}

impl Surfaces {
    pub(super) fn new() -> Self {
        let embed = SurfaceResource::new_embedded;
        // Anneau couché dans le plan horizontal (le tore natif est dans le plan XY).
        let flat = Matrix4::from_axis_angle(&Vector3::x_axis(), std::f32::consts::FRAC_PI_2);
        Self {
            cells: [10, 20, 30].map(|seed| embed(blob(seed, 0.06, 20, 28))),
            rocks: [1, 2, 3, 4].map(|seed| embed(blob(seed, 0.30, 12, 16))),
            sphere: embed(blob(0, 0.0, 14, 20)),
            crystal: embed(crystal()),
            ring: embed(SurfaceData::make_torus(1.0, 0.12, 10, 36, &flat)),
            plane: embed(plane()),
        }
    }

    /// Membrane cellulaire ; la variante est choisie par `variant` (par exemple l'identité).
    pub(super) fn cell(&self, variant: usize) -> SurfaceResource {
        self.cells[variant % self.cells.len()].clone()
    }

    pub(super) fn rock(&self, variant: usize) -> SurfaceResource {
        self.rocks[variant % self.rocks.len()].clone()
    }

    pub(super) fn sphere(&self) -> SurfaceResource {
        self.sphere.clone()
    }

    pub(super) fn crystal(&self) -> SurfaceResource {
        self.crystal.clone()
    }

    pub(super) fn plane(&self) -> SurfaceResource {
        self.plane.clone()
    }


    pub(super) fn ring(&self) -> SurfaceResource {
        self.ring.clone()
    }
}
