use crate::biology::Cell;
use crate::habitat::Environment;
use rand::{SeedableRng, rngs::StdRng};
use std::time::Duration;

#[test]
fn resting_metabolism_is_measured_per_simulated_second() {
    let mut cell = Cell::new(
        0,
        0,
        crate::definitions::Definitions::builtin().species().clone(),
    );
    for _ in 0..625 {
        cell.update(20.0, Environment::Water, 1.0, Duration::from_millis(16));
    }
    assert_eq!(cell.age(), Duration::from_secs(10));
    assert!((cell.energy() - 90.0).abs() < 0.01);
}

#[test]
fn division_requires_maturity_and_energy_and_conserves_energy() {
    let mut cell = Cell::new(
        0,
        0,
        crate::definitions::Definitions::builtin().species().clone(),
    );
    let mut rng = StdRng::seed_from_u64(0);
    assert!(cell.try_divide(&mut rng).is_none());
    for _ in 0..1876 {
        cell.update(20.0, Environment::Water, 1.0, Duration::from_millis(16));
    }
    // Une cellule mature doit encore trouver de la nourriture pour atteindre le seuil.
    assert!(cell.try_divide(&mut rng).is_none());
    cell.feed(20.0);
    let before = cell.energy();
    let daughter = cell
        .try_divide(&mut rng)
        .expect("mature et suffisamment nourrie");
    assert!((cell.energy() + daughter.energy() - before).abs() < 0.001);
    assert_eq!(daughter.age(), Duration::ZERO);
    cell.feed(100.0);
    assert!(cell.try_divide(&mut rng).is_none());
}

#[test]
fn starvation_still_depletes_energy() {
    let mut cell = Cell::new(
        0,
        0,
        crate::definitions::Definitions::builtin().species().clone(),
    );
    for _ in 0..6313 {
        cell.update(20.0, Environment::Water, 1.0, Duration::from_millis(16));
    }
    assert!(cell.energy() <= 0.0);
}

#[test]
fn hunger_triggers_foraging_even_in_cold_water_at_one_move_per_second() {
    let mut cell = Cell::new(
        0,
        0,
        crate::definitions::Definitions::builtin().species().clone(),
    );
    for _ in 0..40 {
        cell.update(20.0, Environment::Water, 20.0, Duration::from_secs(1));
    }
    let mut moves = 0;
    for _ in 0..125 {
        if cell.update(20.0, Environment::Water, 0.0, Duration::from_millis(16)) {
            moves += 1;
        }
    }
    assert_eq!(moves, 2);
}

#[test]
fn available_food_avoids_unnecessary_movement() {
    let mut cell = Cell::new(
        0,
        0,
        crate::definitions::Definitions::builtin().species().clone(),
    );
    for _ in 0..40 {
        cell.update(20.0, Environment::Water, 20.0, Duration::from_secs(1));
    }
    assert!(!cell.update(40.0, Environment::Water, 20.0, Duration::from_secs(1)));
}

#[test]
fn adaptation_changes_metabolism_without_changing_age() {
    let mut water = Cell::new(
        0,
        0,
        crate::definitions::Definitions::builtin().species().clone(),
    );
    let mut land = Cell::new(
        0,
        0,
        crate::definitions::Definitions::builtin().species().clone(),
    );
    water.update(20.0, Environment::Water, 20.0, Duration::from_secs(1));
    land.update(20.0, Environment::Land, 20.0, Duration::from_secs(1));
    assert!((water.energy() - 99.0).abs() < 0.001);
    assert!((land.energy() - 98.1).abs() < 0.001);
    assert_eq!(water.age(), land.age());
}

#[test]
fn feeding_stops_at_storage_capacity() {
    let mut cell = Cell::new(
        0,
        0,
        crate::definitions::Definitions::builtin().species().clone(),
    );
    cell.update(20.0, Environment::Water, 20.0, Duration::from_secs(10));
    assert_eq!(cell.energy_deficit(), 10.0);
    cell.feed(20.0);
    assert_eq!(cell.energy(), 100.0);
    assert_eq!(cell.energy_deficit(), 0.0);
}

#[test]
fn maturity_boundary_is_inclusive_and_daughter_must_mature_again() {
    let mut cell = Cell::new(
        0,
        0,
        crate::definitions::Definitions::builtin().species().clone(),
    );
    let mut rng = StdRng::seed_from_u64(0);
    cell.update(
        20.0,
        Environment::Water,
        20.0,
        Duration::from_millis(29_999),
    );
    cell.feed(100.0);
    assert!(cell.try_divide(&mut rng).is_none());
    cell.update(20.0, Environment::Water, 20.0, Duration::from_millis(1));
    let mut daughter = cell.try_divide(&mut rng).unwrap();
    daughter.feed(100.0);
    assert!(daughter.try_divide(&mut rng).is_none());
    assert_eq!(daughter.age(), Duration::ZERO);
}

#[test]
fn movement_cost_depends_on_travel_and_metabolism() {
    let mut cell = Cell::new(
        0,
        0,
        crate::definitions::Definitions::builtin().species().clone(),
    );
    cell.move_to(0, 0);
    assert_eq!(cell.energy(), 100.0);
    cell.move_to(1, 0);
    assert_eq!(cell.energy(), 99.0);
}

#[test]
fn division_keeps_maternal_network_unchanged_and_resets_daughter_activity() {
    let mut cell = Cell::new(
        0,
        0,
        crate::definitions::Definitions::builtin().species().clone(),
    );
    let mut rng = StdRng::seed_from_u64(0);
    cell.update(40.0, Environment::Water, 20.0, Duration::from_secs(30));
    cell.feed(100.0);
    let bias = cell.neuron().bias();
    let weight = cell.neuron().connections()[0].weight();
    let mut daughter = cell.try_divide(&mut rng).unwrap();
    assert_eq!(cell.neuron().bias(), bias);
    assert_eq!(cell.neuron().connections()[0].weight(), weight);
    assert_eq!(
        daughter.neuron().complexity(),
        daughter.genes().neuron_complexity()
    );
    assert_eq!(daughter.neuron().activation(), 0.0);
    assert_eq!((daughter.x(), daughter.y()), (cell.x(), cell.y()));
    daughter.update(0.0, Environment::Water, 20.0, Duration::from_millis(16));
    assert!(cell.neuron().activation() > 0.0);
}

#[test]
fn sustained_energy_surplus_allows_repeated_reproduction() {
    let mut cell = Cell::new(
        0,
        0,
        crate::definitions::Definitions::builtin().species().clone(),
    );
    let mut rng = StdRng::seed_from_u64(0);
    let mut births = 0;
    for _ in 0..3_750 {
        // 60 secondes, deux périodes de maturation.
        cell.update(20.0, Environment::Water, 20.0, Duration::from_millis(16));
        cell.feed(2.0 * 0.016);
        if cell.try_divide(&mut rng).is_some() {
            births += 1;
        }
    }
    assert_eq!(births, 2);
}

#[test]
fn founder_uses_the_shared_species_neural_definition() {
    let text = include_str!("../../../data/definitions.ron")
        .replace("neuron_complexity: 1", "neuron_complexity: 3")
        .replace("temperature_weight: 0.8", "temperature_weight: 0.4")
        .replace("bias: -20.0", "bias: -5.0");
    let definitions: crate::definitions::Definitions = ron::from_str(&text).unwrap();
    let cell = Cell::new(0, 0, definitions.species().clone());
    assert_eq!(cell.neuron().complexity(), 3);
    assert_eq!(cell.neuron().connections()[0].weight(), 0.4);
    assert_eq!(cell.neuron().bias(), -5.0);
}
