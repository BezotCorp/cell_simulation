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
    // Une seconde de métabolisme par seconde dans l'eau, majorée de l'inadaptation à la terre sinon.
    let penalty = match simulation.grid().get_environment_type(0, 0) {
        crate::habitat::Environment::Water => 0.0,
        crate::habitat::Environment::Land => 1.0 - cell.genes().land_adaptation(),
    };
    assert!((cell.energy() - (100.0 - 10.0 * (1.0 + penalty))).abs() < 0.01);
}

fn seeded(size: usize, seed: u64) -> Simulation {
    let text =
        include_str!("../../../data/definitions.ron").replace("size: 10", &format!("size: {size}"));
    let definitions: crate::definitions::Definitions = ron::from_str(&text).unwrap();
    Simulation::from_definitions(std::sync::Arc::new(definitions), seed).unwrap()
}

#[test]
fn offspring_spread_continuously_and_the_species_colonizes_distant_sites() {
    let mut simulation = seeded(24, 5);
    let start = simulation.cells()[0].position();
    let mut fine = std::collections::HashSet::new();
    for _ in 0..120_000 {
        simulation.step(Duration::from_millis(16));
    }
    assert!(!simulation.cells().is_empty());
    let size = simulation.grid().size() as f32;
    let mut farthest = 0.0_f32;
    for cell in simulation.cells() {
        let (x, y) = cell.position();
        assert!((0.0..size).contains(&x) && (0.0..size).contains(&y));
        let dx = (x - start.0).abs().min(size - (x - start.0).abs());
        let dy = (y - start.1).abs().min(size - (y - start.1).abs());
        farthest = farthest.max(dx.hypot(dy));
        assert!(simulation.grid().is_passable(cell.x(), cell.y()));
        fine.insert((cell.position().0.to_bits(), cell.position().1.to_bits()));
    }
    // Les positions ne sont pas ramenées à des centres de case.
    assert!(fine.len() * 2 > simulation.cells().len());
    assert!(farthest > 4.0, "la colonisation n'a pas dépassé {farthest}");
}

#[test]
fn a_current_carries_organisms_that_cannot_swim_against_it_and_spares_those_that_can() {
    let travelled = |east: f32| {
        let mut simulation = seeded(10, 3);
        let hydrology = simulation
            .grid()
            .hydrology()
            .clone()
            .with_uniform_current(east, 0.0, 1.0);
        simulation.restore_hydrology(hydrology).unwrap();
        let start = simulation.cells()[0].position();
        // Moins d'une seconde simulée : le courant installé n'est pas encore recalculé.
        for _ in 0..55 {
            simulation.step(Duration::from_millis(16));
        }
        let end = simulation.cells()[0].position();
        let dx = (end.0 - start.0 + 5.0).rem_euclid(10.0) - 5.0;
        let dy = (end.1 - start.1 + 5.0).rem_euclid(10.0) - 5.0;
        (dx, dy)
    };
    // La fondatrice nage à 0,12 site/s : un courant de 0,05 ne la déplace pas.
    let (gentle_x, gentle_y) = travelled(0.05);
    assert!(gentle_x.hypot(gentle_y) < 1.5, "{gentle_x} {gentle_y}");
    // Un courant d'un site/s l'emporte vers l'est sur près de 0,88 × 0,88 site.
    let (strong_x, strong_y) = travelled(1.0);
    assert!(strong_x > 0.6, "{strong_x}");
    assert!(strong_y.abs() < 1.0);
}

#[test]
#[ignore = "mesure d'interaction eau/biologie, lancée à la main"]
fn how_often_do_currents_exceed_the_swimming_capacity() {
    for seed in [3, 7, 42] {
        let mut simulation = seeded(10, seed);
        let (mut samples, mut swept, mut wet_cells, mut peak) = (0u64, 0u64, 0u64, 0.0_f32);
        for tick in 0..300_000u32 {
            simulation.step(Duration::from_millis(16));
            if tick % 62 == 0 {
                for cell in simulation.cells() {
                    let flow = simulation.grid().flow_at(cell.x(), cell.y());
                    samples += 1;
                    wet_cells += u64::from(flow.wetted() > 0.0);
                    let speed = flow.speed() * flow.wetted();
                    peak = peak.max(speed);
                    swept += u64::from(speed > cell.swim_capacity());
                }
            }
        }
        eprintln!(
            "graine {seed} : {} cellules, {:.1} % dans l'eau, {:.2} % emportées, courant maximal subi {peak:.2}",
            simulation.cells().len(),
            100.0 * wet_cells as f64 / samples as f64,
            100.0 * swept as f64 / samples as f64
        );
    }
}
