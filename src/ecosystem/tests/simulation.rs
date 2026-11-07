use crate::ecosystem::Simulation;
use std::time::Duration;

#[test]
fn population_remains_viable_with_finite_resources() {
    let mut late_births = 0;
    let mut neural_variation = false;
    for seed in 0..16 {
        let mut simulation = seeded(10, seed);
        let mut peak = 1;
        let mut births_at_one_hour = 0;
        // Débit maximal : 100 sites × 2 unités/s ; entretien minimal : 0,5 unité/s.
        // Cette enveloppe de régression ne constitue pas une limite imposée au moteur.
        let population_envelope = 400;
        for tick in 0..450_000 {
            // Deux heures simulées au pas fixe de 16 ms.
            simulation.step(Duration::from_millis(16));
            let count = simulation.cells().len();
            assert!(count > 0, "extinction : seed={seed}, tick={tick}");
            assert!(
                count <= population_envelope,
                "croissance excessive : seed={seed}, tick={tick}"
            );
            peak = peak.max(count);
            if tick == 224_999 {
                births_at_one_hour = simulation.generation();
            }
        }
        let births_in_second_hour = simulation.generation() - births_at_one_hour;
        late_births += births_in_second_hour;
        neural_variation |= simulation.cells().iter().any(|cell| {
            cell.neuron().bias() != -20.0 || cell.neuron().connections()[0].weight() != 0.8
        });
        eprintln!(
            "seed={seed} final={} peak={peak} births={} second_hour={births_in_second_hour}",
            simulation.cells().len(),
            simulation.generation()
        );
    }
    // Un habitat à l'équilibre peut ne plus avoir de surplus pour une naissance.
    // L'ensemble conserve cependant une activité reproductive et une variation héritée.
    assert!(late_births > 0);
    assert!(neural_variation);
}

#[test]
fn starvation_removes_cells_without_forced_repopulation() {
    // Une grille d'une case est entièrement occupée par une source ou un obstacle.
    let mut simulation = seeded(1, 0);
    assert_eq!(simulation.cells().len(), 1);
    assert!(!matches!(
        simulation.grid().get_entity(0, 0),
        crate::habitat::Entity::Food(_)
    ));
    for _ in 0..12_500 {
        simulation.step(Duration::from_millis(16));
    }
    assert!(simulation.cells().is_empty());
    assert_eq!(simulation.generation(), 0);
}

#[test]
fn same_seed_replays_the_same_population_and_mutations() {
    let mut first = seeded(10, 42);
    let mut second = seeded(10, 42);
    for _ in 0..10_000 {
        first.step(Duration::from_millis(16));
        second.step(Duration::from_millis(16));
    }
    assert!(first.generation() > 0);
    assert!(!first.cells().is_empty());
    assert_eq!(first.generation(), second.generation());
    assert_eq!(first.cells().len(), second.cells().len());
    for (a, b) in first.cells().iter().zip(second.cells()) {
        assert_eq!(
            (a.x(), a.y(), a.age(), a.energy()),
            (b.x(), b.y(), b.age(), b.energy())
        );
        assert_eq!(a.genes().metabolism_rate(), b.genes().metabolism_rate());
        assert_eq!(
            a.genes().division_threshold(),
            b.genes().division_threshold()
        );
        assert_eq!(a.genes().water_adaptation(), b.genes().water_adaptation());
        assert_eq!(a.neuron().bias(), b.neuron().bias());
        assert_eq!(a.neuron().complexity(), b.neuron().complexity());
        assert_eq!(a.neuron().activation(), b.neuron().activation());
        let weights = |cell: &crate::biology::Cell| {
            cell.neuron()
                .connections()
                .iter()
                .map(|c| (c.from_id(), c.to_id(), c.weight()))
                .collect::<Vec<_>>()
        };
        assert_eq!(weights(a), weights(b));
    }
}

#[test]
fn cells_and_descendants_remain_on_accessible_habitats() {
    let mut births = 0;
    for seed in 0..8 {
        let mut simulation = seeded(10, seed);
        for tick in 0..10_000 {
            simulation.step(Duration::from_millis(16));
            if tick % 62 == 0 {
                for cell in simulation.cells() {
                    assert!(simulation.grid().is_passable(cell.x(), cell.y()));
                }
            }
        }
        births += simulation.generation();
    }
    assert!(births > 0);
}

#[test]
fn blocked_movement_spends_only_maintenance_energy() {
    let mut simulation = seeded(1, 0);
    assert_eq!(simulation.cells().len(), 1);
    for _ in 0..625 {
        simulation.step(Duration::from_millis(16));
    }
    let cell = &simulation.cells()[0];
    assert_eq!((cell.x(), cell.y()), (0, 0));
    assert_eq!(cell.age(), Duration::from_secs(10));
    assert!((cell.energy() - 90.0).abs() < 0.01);
}

fn seeded(size: usize, seed: u64) -> Simulation {
    let text =
        include_str!("../../../data/definitions.ron").replace("size: 10", &format!("size: {size}"));
    let definitions: crate::definitions::Definitions = ron::from_str(&text).unwrap();
    Simulation::from_definitions(std::sync::Arc::new(definitions), seed).unwrap()
}
