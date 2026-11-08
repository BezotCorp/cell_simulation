use crate::definitions::Definitions;
use crate::ecosystem::Simulation;
use std::collections::HashSet;

fn evolved(seed: u64, steps: u64) -> Simulation {
    let mut simulation = Simulation::from_definitions(Definitions::builtin(), seed).unwrap();
    run(&mut simulation, steps);
    simulation
}

#[test]
fn lineage_is_complete_indexed_and_consistent_with_the_living_population() {
    let simulation = evolved(42, 150_000);
    let records = simulation.lineage();
    // Une fondatrice et une naissance par division : aucun enregistrement n'est jamais purgé.
    assert_eq!(records.len() as u64, simulation.generation() as u64 + 1);
    let summary = simulation.summary();
    assert!(summary.deaths() > 0, "l'expérience doit avoir des décès");
    assert_eq!(
        summary.deaths(),
        records
            .iter()
            .filter(|r| r.died_at_tick().is_some())
            .count() as u64
    );
    let living: HashSet<_> = simulation.cells().iter().map(|cell| cell.id()).collect();
    for record in records {
        assert_eq!(
            simulation.lineage_record(record.id()).unwrap().id(),
            record.id()
        );
        assert_eq!(
            simulation.is_alive(record.id()),
            living.contains(&record.id())
        );
        assert_eq!(
            record.died_at_tick().is_none(),
            living.contains(&record.id())
        );
    }
}

#[test]
fn rebinding_rebuilds_the_derived_indexes_without_changing_the_history() {
    let mut simulation = evolved(7, 60_000);
    let before: Vec<_> = simulation
        .lineage()
        .iter()
        .map(|r| (r.id(), r.parent(), r.born_at_tick(), r.died_at_tick()))
        .collect();
    let bytes =
        bincode_next::serde::encode_to_vec(&simulation, bincode_next::config::standard()).unwrap();
    let (mut restored, _): (Simulation, usize) =
        bincode_next::serde::decode_from_slice(&bytes, bincode_next::config::standard()).unwrap();
    restored.bind_definitions(Definitions::builtin()).unwrap();
    let after: Vec<_> = restored
        .lineage()
        .iter()
        .map(|r| (r.id(), r.parent(), r.born_at_tick(), r.died_at_tick()))
        .collect();
    assert_eq!(before, after);
    assert_eq!(restored.summary().deaths(), simulation.summary().deaths());
    for cell in simulation.cells() {
        assert!(restored.is_alive(cell.id()));
    }
    // L'état restauré poursuit exactement comme l'original.
    run(&mut simulation, 2_000);
    run(&mut restored, 2_000);
    assert_eq!(simulation.summary(), restored.summary());
}

#[test]
fn failed_binding_leaves_the_simulation_exactly_as_it_was() {
    let mut simulation = evolved(1, 20_000);
    let snapshot =
        bincode_next::serde::encode_to_vec(&simulation, bincode_next::config::standard()).unwrap();
    let original = simulation.definitions().clone();
    // Monde plus petit que l'état restauré : la validation doit échouer.
    let text = include_str!("../../../data/definitions.ron").replace("size: 10", "size: 9");
    let incompatible: Definitions = ron::from_str(&text).unwrap();
    assert!(
        simulation
            .bind_definitions(std::sync::Arc::new(incompatible))
            .is_err()
    );
    assert!(std::sync::Arc::ptr_eq(simulation.definitions(), &original));
    assert_eq!(
        snapshot,
        bincode_next::serde::encode_to_vec(&simulation, bincode_next::config::standard()).unwrap()
    );
    for cell in simulation.cells() {
        assert!(simulation.is_alive(cell.id()));
    }
}

fn run(simulation: &mut Simulation, steps: u64) {
    for _ in 0..steps {
        simulation.step(crate::ecosystem::FIXED_STEP);
    }
}
