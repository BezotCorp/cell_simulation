use crate::definitions::Definitions;
use crate::ecosystem::{FIXED_STEP, Simulation};
use crate::persistence::{SessionView, ViewMode};
use crate::world_scene::{OrbitCamera, Picked, WorldScene};
use fyrox::{
    core::algebra::{UnitQuaternion, Vector2, Vector3},
    scene::Scene,
};

const FRAME: (u32, u32) = (1100, 780);

fn simulation(steps: u64) -> Simulation {
    let mut simulation = Simulation::from_definitions(Definitions::builtin(), 42).unwrap();
    for _ in 0..steps {
        simulation.step(FIXED_STEP);
    }
    simulation
}

/// Scène synchronisée et posée : caméra cadrée sur le biotope, formes à leur place.
fn staged(simulation: &Simulation) -> (WorldScene, Scene, Vector3<f32>) {
    let (mut world, mut scene) = WorldScene::compose(simulation, FRAME, Default::default());
    let view = SessionView::default();
    world.sync(simulation, &view, &mut scene);
    for _ in 0..40 {
        world.animate(0.05, &mut scene);
    }
    let size = Vector2::new(FRAME.0 as f32, FRAME.1 as f32);
    let extent = simulation.grid().size() as f32;
    let (yaw, pitch) = (-0.65_f32, 0.82_f32);
    let (target, distance) = OrbitCamera::framing(extent, size.x / size.y, yaw, pitch);
    let target = Vector3::from(target);
    let eye = target
        + Vector3::new(yaw.sin() * pitch.cos(), pitch.sin(), yaw.cos() * pitch.cos()) * distance;
    let camera = world.camera_handle();
    scene.graph[camera]
        .local_transform_mut()
        .set_position(eye)
        .set_rotation(UnitQuaternion::face_towards(&(target - eye), &Vector3::y()));
    scene.graph.update_hierarchical_data();
    scene.graph[camera].calculate_matrices(size);
    (world, scene, eye)
}

fn project(scene: &Scene, world: &WorldScene, point: Vector3<f32>) -> Vector2<f32> {
    let size = Vector2::new(FRAME.0 as f32, FRAME.1 as f32);
    let clip = scene.graph[world.camera_handle()].view_projection_matrix() * point.push(1.0);
    Vector2::new(
        (clip.x / clip.w * 0.5 + 0.5) * size.x,
        (1.0 - (clip.y / clip.w * 0.5 + 0.5)) * size.y,
    )
}

#[test]
fn clicking_a_cell_or_a_habitat_selects_exactly_that_one() {
    let simulation = simulation(60_000);
    assert!(simulation.cells().len() > 20, "il faut une foule pour ce test");
    let (world, scene, _) = staged(&simulation);

    let (mut organisms, mut exact) = (0, 0);
    for cell in simulation.cells() {
        let position = world.selected_position(cell.id()).expect("visuel créé");
        let at = project(&scene, &world, Vector3::from(position));
        match world.pick(&scene, &simulation, at) {
            // Un individu devant celui visé peut légitimement l'emporter : jamais un habitat.
            Some(Picked::Organism(found)) => {
                organisms += 1;
                if found == cell.id() {
                    exact += 1;
                } else {
                    assert!(simulation.is_alive(found));
                }
            }
            other => panic!("clic au centre d'une cellule : {other:?}"),
        }
    }
    assert_eq!(organisms, simulation.cells().len());
    assert!(
        exact * 10 >= organisms * 7,
        "seulement {exact} sélections exactes sur {organisms}"
    );

    // Un habitat sans occupant : le clic sur son centre le désigne.
    let size = simulation.grid().size();
    let occupied: std::collections::HashSet<_> =
        simulation.cells().iter().map(|c| (c.x(), c.y())).collect();
    let mut found = 0;
    let free = size * size - occupied.len();
    for y in 0..size {
        for x in 0..size {
            if occupied.contains(&(x, y)) {
                continue;
            }
            let offset = (size as f32 - 1.0) / 2.0;
            let center = Vector3::new(x as f32 - offset, 0.0, y as f32 - offset);
            let at = project(&scene, &world, center);
            match world.pick(&scene, &simulation, at) {
                Some(Picked::Habitat(id)) => {
                    assert_eq!(id, simulation.grid().habitat_id(x, y));
                    found += 1;
                }
                // Une cellule d'une autre case peut masquer celle-ci en vue oblique.
                Some(Picked::Organism(_)) => {}
                None => panic!("habitat ({x}, {y}) introuvable"),
            }
        }
    }
    assert!(found * 2 >= free, "{found} habitats trouvés sur {free} libres");
    // Hors du biotope : rien.
    assert!(world.pick(&scene, &simulation, Vector2::new(2.0, 2.0)).is_none());
}

#[test]
fn views_share_one_ground_and_synchronise_without_growing_the_scene() {
    let simulation = simulation(20_000);
    let (mut world, mut scene) = WorldScene::compose(&simulation, FRAME, Default::default());
    let mut view = SessionView::default();
    world.sync(&simulation, &view, &mut scene);
    let nodes = scene.graph.node_count();
    // Changer de vue ou resynchroniser un état identique ne crée aucun nœud : le sol est unique.
    for _ in 0..4 {
        view.cycle_mode();
        world.sync(&simulation, &view, &mut scene);
        world.animate(0.016, &mut scene);
    }
    assert_eq!(view.mode(), ViewMode::Ecosystem);
    assert_eq!(scene.graph.node_count(), nodes);
}

#[test]
fn the_asset_library_format_is_strict_and_optional() {
    use crate::world_scene::library_definition::LibraryDefinition;
    let library: LibraryDefinition =
        ron::from_str(r#"(slots: {"flagellum": ["a.glb", "b.glb"], "spine": ["s.glb"]})"#).unwrap();
    assert_eq!(library.slots["flagellum"].len(), 2);
    // Un fichier vide ou sans emplacement est valide : tout reste procédural.
    assert!(ron::from_str::<LibraryDefinition>("()").unwrap().slots.is_empty());
    assert!(ron::from_str::<LibraryDefinition>("(slot: {})").is_err());
    // Le fichier fourni avec le projet est valide.
    let shipped = include_str!("../../../data/assets/library.ron");
    assert!(ron::from_str::<LibraryDefinition>(shipped).is_ok());
}
