use crate::biology::Cell;
use crate::definitions::Definitions;
use crate::world_scene::{
    materials::Materials, organism_visual::OrganismVisual, surfaces::Surfaces,
};
use fyrox::core::algebra::Vector3;
use fyrox::scene::graph::Graph;

fn visual(graph: &mut Graph) -> OrganismVisual {
    let definitions = Definitions::builtin();
    let cell = Cell::new(0, 0, definitions.species().clone());
    let materials = Materials::new(definitions.appearance());
    OrganismVisual::new(
        graph,
        &cell,
        &Surfaces::new(),
        &materials,
        0.1,
        Vector3::zeros(),
        true,
    )
}

#[test]
fn smoothing_depends_on_elapsed_time_not_on_the_frame_rate() {
    let mut graph = Graph::default();
    let target = Vector3::new(3.0, 0.5, -2.0);
    let mut at_30 = visual(&mut graph);
    let mut at_144 = visual(&mut graph);
    for visual in [&mut at_30, &mut at_144] {
        visual.set_target(target, 0.2, 100.0);
    }
    for _ in 0..30 {
        at_30.animate(1.0 / 30.0);
    }
    for _ in 0..144 {
        at_144.animate(1.0 / 144.0);
    }
    assert!((at_30.position() - at_144.position()).norm() < 1e-4);
    assert!((at_30.radius() - at_144.radius()).abs() < 1e-5);
    // Après une seconde, la cible est atteinte sans dépassement ; ensuite plus rien ne bouge.
    assert!((at_30.position() - target).norm() < 1e-3);
    assert!(!at_30.animate(1.0 / 60.0) || (at_30.position() - target).norm() < 1e-3);
    for _ in 0..200 {
        at_30.animate(1.0 / 60.0);
    }
    assert!(
        !at_30.animate(1.0 / 60.0),
        "une cellule au repos ne réécrit plus sa transformation"
    );
}

#[test]
fn distant_targets_are_reached_without_sliding_across_the_grid() {
    let mut graph = Graph::default();
    let mut visual = visual(&mut graph);
    visual.set_target(Vector3::new(50.0, 0.3, 50.0), 0.1, 2.0);
    assert!((visual.position() - Vector3::new(50.0, 0.3, 50.0)).norm() < 1e-6);
}

#[test]
fn crowded_sites_shrink_their_organisms_smoothly_without_hiding_them() {
    assert_eq!(OrganismVisual::crowd_scale(0), 1.0);
    assert_eq!(OrganismVisual::crowd_scale(1), 1.0);
    let mut previous = 1.0;
    for count in [2, 3, 7, 40, 300, 5_000] {
        let scale = OrganismVisual::crowd_scale(count);
        assert!((0.35..=previous).contains(&scale), "{count} -> {scale}");
        previous = scale;
    }
}

#[test]
fn the_drawn_anatomy_follows_the_phenotype_and_the_level_of_detail() {
    use crate::biology::{Cell, PartKind};
    use crate::world_scene::{component_library::ComponentLibrary, detail_level::DetailLevel};
    // Un organisme à flagelles, cuticule et pigments, comme l'évolution peut en produire.
    let text = include_str!("../../../data/definitions.ron")
        .replace("{Digestive: 1.0}", "{Digestive: 1.0, Pigment: 0.75, Raptorial: 0.5}");
    let definitions: Definitions = ron::from_str(&text).unwrap();
    let cell = Cell::new(0, 0, definitions.species().clone());
    let anatomy = cell.morphology().anatomy();
    assert!(anatomy.parts().iter().any(|part| part.kind() == PartKind::Chloroplast));
    assert!(anatomy.parts().iter().any(|part| part.kind() == PartKind::Spine));
    assert!(anatomy.parts().iter().all(|part| part.kind() != PartKind::Ganglion));

    let mut scene = fyrox::scene::Scene::new();
    let materials = Materials::new(definitions.appearance());
    let surfaces = Surfaces::new();
    let mut visual = OrganismVisual::new(
        &mut scene.graph,
        &cell,
        &surfaces,
        &materials,
        0.1,
        Vector3::zeros(),
        true,
    );
    let bare = scene.graph.node_count();
    let library = ComponentLibrary::default();
    visual.set_detail(&mut scene, &cell, &library, &surfaces, &materials, DetailLevel::Principal);
    let principal = scene.graph.node_count() - bare;
    visual.set_detail(&mut scene, &cell, &library, &surfaces, &materials, DetailLevel::Full);
    let full = scene.graph.node_count() - bare;
    // Le noyau et deux familles, puis toute l'anatomie ; revenir à la membrane retire tout.
    assert!(principal > 1 && full > principal, "{principal} puis {full}");
    let expected: usize = 1 + anatomy.parts().iter().map(|part| usize::from(part.count())).sum::<usize>();
    assert_eq!(full as usize, expected.min(40));
    visual.set_detail(&mut scene, &cell, &library, &surfaces, &materials, DetailLevel::Bare);
    assert_eq!(scene.graph.node_count(), bare);
}
