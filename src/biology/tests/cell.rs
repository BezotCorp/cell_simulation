use crate::biology::{Cell, Conditions};
use crate::habitat::Environment;
use rand::{SeedableRng, rngs::StdRng};
use std::time::Duration;

/// Ancêtre dont le génome compte en plus un gène de tissu nerveux (une unité de calcul).
fn neural_cell() -> Cell {
    let text = include_str!("../../../data/definitions.ron")
        .replace("genes: [", "genes: [\n                (bias: 0.5, effects: {Neural: 0.125}),");
    let definitions: crate::definitions::Definitions = ron::from_str(&text).unwrap();
    Cell::new(0, 0, definitions.species().clone())
}

/// Un pas dans un site sans lumière ni chaleur particulières.
fn tick(
    cell: &mut Cell,
    temperature: f32,
    environment: Environment,
    food: f32,
    dt: Duration,
) -> bool {
    cell.update(&Conditions::new(temperature, 0.0, 0.0, environment, food, 20.0, 0.0), dt)
}

#[test]
fn resting_metabolism_is_measured_per_simulated_second() {
    let mut cell = Cell::new(
        0,
        0,
        crate::definitions::Definitions::builtin().species().clone(),
    );
    for _ in 0..625 {
        tick(&mut cell, 20.0, Environment::Water, 1.0, Duration::from_millis(16));
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
    assert!(cell.try_divide(&mut rng, 1.0).is_none());
    for _ in 0..1876 {
        tick(&mut cell, 20.0, Environment::Water, 1.0, Duration::from_millis(16));
    }
    // Une cellule mature doit encore trouver de la nourriture pour atteindre le seuil.
    assert!(cell.try_divide(&mut rng, 1.0).is_none());
    cell.feed(20.0);
    let before = cell.energy();
    let daughter = cell
        .try_divide(&mut rng, 1.0)
        .expect("mature et suffisamment nourrie");
    assert!((cell.energy() + daughter.energy() - before).abs() < 0.001);
    assert_eq!(daughter.age(), Duration::ZERO);
    cell.feed(100.0);
    assert!(cell.try_divide(&mut rng, 1.0).is_none());
}

#[test]
fn starvation_still_depletes_energy() {
    let mut cell = Cell::new(
        0,
        0,
        crate::definitions::Definitions::builtin().species().clone(),
    );
    for _ in 0..6313 {
        tick(&mut cell, 20.0, Environment::Water, 1.0, Duration::from_millis(16));
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
        tick(&mut cell, 20.0, Environment::Water, 20.0, Duration::from_secs(1));
    }
    let mut moves = 0;
    for _ in 0..125 {
        if tick(&mut cell, 20.0, Environment::Water, 0.0, Duration::from_millis(16)) {
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
        tick(&mut cell, 20.0, Environment::Water, 20.0, Duration::from_secs(1));
    }
    assert!(!tick(&mut cell, 40.0, Environment::Water, 20.0, Duration::from_secs(1)));
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
    tick(&mut water, 20.0, Environment::Water, 20.0, Duration::from_secs(1));
    tick(&mut land, 20.0, Environment::Land, 20.0, Duration::from_secs(1));
    assert!((water.energy() - 99.0).abs() < 0.001);
    assert!((land.energy() - 98.125).abs() < 0.001);
    assert_eq!(water.age(), land.age());
}

#[test]
fn feeding_stops_at_storage_capacity() {
    let mut cell = Cell::new(
        0,
        0,
        crate::definitions::Definitions::builtin().species().clone(),
    );
    tick(&mut cell, 20.0, Environment::Water, 20.0, Duration::from_secs(10));
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
    tick(&mut cell, 
        20.0,
        Environment::Water,
        20.0,
        Duration::from_millis(29_999),
    );
    cell.feed(100.0);
    assert!(cell.try_divide(&mut rng, 1.0).is_none());
    tick(&mut cell, 20.0, Environment::Water, 20.0, Duration::from_millis(1));
    let mut daughter = cell.try_divide(&mut rng, 1.0).unwrap();
    daughter.feed(100.0);
    assert!(daughter.try_divide(&mut rng, 1.0).is_none());
    assert_eq!(daughter.age(), Duration::ZERO);
}

#[test]
fn movement_cost_depends_on_travel_and_metabolism() {
    let mut cell = Cell::new(
        0,
        0,
        crate::definitions::Definitions::builtin().species().clone(),
    );
    cell.move_to(cell.position(), 10);
    assert_eq!(cell.energy(), 100.0);
    cell.move_to((1.2, 0.5), 10);
    assert_eq!(cell.energy(), 99.0);
}

#[test]
fn position_is_continuous_and_wraps_around_the_torus() {
    let mut cell = Cell::new(
        0,
        0,
        crate::definitions::Definitions::builtin().species().clone(),
    );
    assert_eq!(cell.position(), (0.5, 0.5));
    cell.move_to((3.25, 7.75), 10);
    assert_eq!((cell.x(), cell.y()), (3, 7));
    assert_eq!(cell.position(), (3.25, 7.75));
    cell.move_to((-0.25, 10.5), 10);
    assert_eq!((cell.x(), cell.y()), (9, 0));
    assert_eq!(cell.position(), (9.75, 0.5));
    // Une valeur tout juste négative ne doit jamais produire un site hors du monde.
    cell.settle((-1e-9, -1e-9), 10);
    assert!(cell.x() < 10 && cell.y() < 10);
    let (east, south) = cell.offsets();
    assert!(Cell::offsets_are_valid(east, south));
}

#[test]
fn legacy_offsets_are_deterministic_and_inside_the_site() {
    let mut cell = Cell::new(
        2,
        3,
        crate::definitions::Definitions::builtin().species().clone(),
    );
    cell.settle_legacy_offsets();
    let first = cell.offsets();
    cell.settle_legacy_offsets();
    assert_eq!(cell.offsets(), first);
    assert!((0.15..=0.85).contains(&first.0) && (0.15..=0.85).contains(&first.1));
    assert_eq!((cell.x(), cell.y()), (2, 3));
}

#[test]
fn a_daughter_inherits_the_genome_exactly_when_nothing_exposes_it_to_mutation() {
    let mut cell = neural_cell();
    let mut rng = StdRng::seed_from_u64(0);
    tick(&mut cell, 40.0, Environment::Water, 20.0, Duration::from_secs(30));
    cell.feed(100.0);
    let mut daughter = cell.try_divide(&mut rng, 0.0).unwrap();
    assert_eq!(daughter.genome(), cell.genome());
    assert_eq!(daughter.activation(), 0.0);
    assert_eq!((daughter.x(), daughter.y()), (cell.x(), cell.y()));
    assert_eq!(daughter.morphology().neural_units(), 1);
    // Le tissu nerveux du fondateur lit la température : au-dessus de 25 °C, il s'active.
    tick(&mut daughter, 40.0, Environment::Water, 20.0, Duration::from_millis(16));
    assert!(daughter.activation() > 0.0);
    tick(&mut daughter, 10.0, Environment::Water, 20.0, Duration::from_millis(16));
    assert!(daughter.activation() < 0.0);
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
        tick(&mut cell, 20.0, Environment::Water, 20.0, Duration::from_millis(16));
        cell.feed(2.0 * 0.016);
        if cell.try_divide(&mut rng, 1.0).is_some() {
            births += 1;
        }
    }
    assert_eq!(births, 2);
}

#[test]
fn founder_uses_the_shared_species_genome_and_has_no_nervous_tissue() {
    let founder = Cell::new(0, 0, crate::definitions::Definitions::builtin().species().clone());
    assert_eq!(founder.morphology().neural_units(), 0);
    assert_eq!(founder.genome().genes().len(), 6);
    assert_eq!(founder.genome().wiring()[0], 0.8);
    assert_eq!(founder.genome().neural_bias(), -20.0);
    // Une mutation fabriquant du tissu nerveux le câble avec les poids du catalogue.
    assert_eq!(neural_cell().morphology().neural_units(), 1);
    assert_eq!(neural_cell().genome().wiring()[0], 0.8);
}

fn swimmer(water_adaptation: &str) -> Cell {
    let half = water_adaptation.parse::<f32>().unwrap() / 2.0;
    let text = include_str!("../../../data/definitions.ron")
        .replace("Hydration: 0.5, Cuticle: 0.125", &format!("Hydration: {half}, Cuticle: 0.125"))
        .replace("Hydration: 0.5, Motor: 0.5", &format!("Hydration: {half}, Motor: 0.5"));
    let definitions: crate::definitions::Definitions = ron::from_str(&text).unwrap();
    Cell::new(3, 3, definitions.species().clone())
}

#[test]
fn swimming_capacity_follows_the_aquatic_adaptation_of_the_genes() {
    let (aquatic, terrestrial) = (swimmer("1.0"), swimmer("0.1"));
    assert!(aquatic.swim_capacity() > 9.0 * terrestrial.swim_capacity());
    assert_eq!(swimmer("0.0").swim_capacity(), 0.0);
    // Un organisme sans adaptation à l'eau est emporté par le moindre courant.
    use crate::hydrology::Flow;
    let mut helpless = swimmer("0.0");
    let drift = helpless
        .swim_against(Flow::new(0.2, 0.0, 1.0), Duration::from_secs(1))
        .expect("emporté");
    assert!((drift.0 - 0.2).abs() < 1e-6);
}

#[test]
fn a_current_beyond_the_swimming_capacity_carries_the_organism_the_rest_is_resisted() {
    use crate::hydrology::Flow;
    let mut cell = swimmer("1.0");
    let capacity = cell.swim_capacity();
    assert!(capacity > 0.0);
    let dt = Duration::from_secs(1);
    // Courant faible : l'organisme tient sur place, au prix d'un effort.
    let before = cell.energy();
    assert_eq!(cell.swim_against(Flow::new(capacity * 0.5, 0.0, 1.0), dt), None);
    assert!(cell.energy() < before);
    // Courant fort : il dérive de l'excédent, dans le sens de l'eau.
    let (east, south) = cell
        .swim_against(Flow::new(capacity * 3.0, 0.0, 1.0), dt)
        .expect("emporté");
    assert!((east - capacity * 2.0).abs() < 1e-5 && south == 0.0);
}

#[test]
fn shallow_water_and_still_water_do_not_move_organisms() {
    use crate::hydrology::Flow;
    let mut cell = swimmer("1.0");
    let energy = cell.energy();
    let dt = Duration::from_secs(1);
    assert_eq!(cell.swim_against(Flow::new(0.0, 0.0, 1.0), dt), None);
    assert_eq!(cell.energy(), energy);
    // Une lame d'eau très mince n'enveloppe pas l'organisme : il n'est que partiellement emporté.
    let thin = cell.swim_against(Flow::new(10.0, 0.0, 0.0), dt);
    assert_eq!(thin, None);
    let partial = cell.swim_against(Flow::new(10.0, 0.0, 0.03), dt).expect("emporté").0;
    let deep = cell.swim_against(Flow::new(10.0, 0.0, 1.0), dt).expect("emporté").0;
    assert!(partial < deep);
}
