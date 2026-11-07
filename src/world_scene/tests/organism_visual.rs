use crate::biology::Cell;
use crate::definitions::Definitions;
use crate::world_scene::{materials::Materials, organism_visual::OrganismVisual};
use fyrox::core::algebra::Vector3;
use fyrox::scene::graph::Graph;

fn visual(graph: &mut Graph) -> OrganismVisual {
    let definitions = Definitions::builtin();
    let cell = Cell::new(0, 0, definitions.species().clone());
    let materials = Materials::new(definitions.appearance());
    OrganismVisual::new(graph, &cell, 0.1, Vector3::zeros(), &materials)
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
fn crowded_organisms_never_overlap_and_stay_inside_their_habitat() {
    for spacing in [1.0_f32, 2.5] {
        for count in [2, 3, 7, 40, 300, 5_000] {
            let slots: Vec<_> = (0..count)
                .map(|index| OrganismVisual::slot(index, count, spacing))
                .collect();
            let radius = slots[0].1;
            assert!(radius.is_finite() && radius > 0.0);
            assert!(slots.iter().all(|slot| slot.1 == radius));
            for (offset, _) in &slots {
                assert!(offset.x.hypot(offset.z) <= 0.40 * spacing + 1e-5);
            }
            // La disposition en spirale de Vogel garantit un écart minimal entre voisins.
            let mut closest = f32::MAX;
            for (i, (a, _)) in slots.iter().enumerate() {
                for (b, _) in slots.iter().skip(i + 1).take(300) {
                    closest = closest.min((a - b).norm());
                }
            }
            assert!(
                closest >= 2.0 * radius,
                "recouvrement avec {count} occupants : écart {closest}, rayon {radius}"
            );
        }
    }
    let (offset, radius) = OrganismVisual::slot(0, 1, 1.0);
    assert_eq!(offset.norm(), 0.0);
    assert!(
        radius.is_infinite(),
        "un occupant isolé garde sa taille normale"
    );
}
