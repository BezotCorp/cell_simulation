use crate::definitions::{Definitions, FounderDefinition};
use crate::ecosystem::{Intervention, KillTarget, ResourceKind, Simulation};
use crate::test_support::edited;
use std::time::Duration;

fn simulation(definitions: std::sync::Arc<Definitions>) -> Simulation {
    Simulation::from_definitions(definitions, 3).expect("simulation")
}

fn coherent(simulation: &Simulation) {
    simulation.verify(simulation.definitions()).expect("expérience cohérente");
}

fn group(count: u32) -> FounderDefinition {
    FounderDefinition { count, ..FounderDefinition::default() }
}

#[test]
fn the_catalog_chooses_how_many_cells_start_and_what_they_carry() {
    let definitions = edited(|d| {
        let founders = d.species_mut().founders_mut();
        founders.push(FounderDefinition { count: 6, ..FounderDefinition::default() });
        founders.push(FounderDefinition {
            count: 4,
            energy: 0.5,
            add_motifs: vec!["pig_a".into(), "pig_b".into()],
            scatter: true,
            ..FounderDefinition::default()
        });
    });
    let simulation = simulation(definitions);
    assert_eq!(simulation.cells().len(), 10);
    let genomes: std::collections::HashSet<_> = simulation.cells().iter().map(|c| c.genome().id()).collect();
    assert_eq!(genomes.len(), 2);
    assert!(simulation.cells().iter().filter(|c| c.energy() < c.capacity() * 0.6).count() == 4);
    coherent(&simulation);
}

#[test]
fn creating_and_killing_cells_keeps_the_experiment_coherent() {
    let mut simulation = simulation(Definitions::builtin());
    let mut carrying = group(20);
    carrying.add_motifs = vec!["neural_a".into()];
    carrying.scatter = true;
    simulation.intervene(&Intervention::Spawn { group: carrying, at: None }).unwrap();
    assert_eq!(simulation.cells().len(), 21);
    coherent(&simulation);

    simulation.intervene(&Intervention::Kill(KillTarget::Count(5))).unwrap();
    assert_eq!(simulation.cells().len(), 16);
    simulation.intervene(&Intervention::Kill(KillTarget::Weakest(3))).unwrap();
    assert_eq!(simulation.cells().len(), 13);
    simulation.intervene(&Intervention::Kill(KillTarget::Fraction(1.0))).unwrap();
    assert!(simulation.cells().is_empty());
    coherent(&simulation);
    // Une expérience vidée peut être repeuplée et continue d'avancer.
    simulation.intervene(&Intervention::Spawn { group: group(3), at: Some((4, 4)) }).unwrap();
    for _ in 0..200 {
        simulation.step(Duration::from_millis(16));
    }
    assert!(simulation.cells().len() >= 3);
    coherent(&simulation);
}

#[test]
fn killing_around_a_site_spares_the_distant_cells() {
    let mut simulation = simulation(Definitions::builtin());
    let mut far = group(5);
    far.scatter = true;
    simulation.intervene(&Intervention::Spawn { group: far, at: None }).unwrap();
    let total = simulation.cells().len();
    simulation
        .intervene(&Intervention::Kill(KillTarget::Around { x: 0, y: 0, radius: 0.0 }))
        .unwrap();
    assert_eq!(simulation.cells().len(), total);
    simulation
        .intervene(&Intervention::Kill(KillTarget::Around { x: 5, y: 5, radius: 20.0 }))
        .unwrap();
    assert!(simulation.cells().is_empty());
    coherent(&simulation);
}

#[test]
fn weather_and_resources_can_be_added_to_the_running_world() {
    let mut simulation = simulation(Definitions::builtin());
    let charge = simulation.grid().storm().total();
    simulation.intervene(&Intervention::Storm { sites: 5, intensity: 3.0 }).unwrap();
    assert!(simulation.grid().storm().total() > charge + 5.0);
    let clouds: f32 = (0..100).map(|i| simulation.grid().hydrology().cloud(i % 10, i / 10)).sum();
    simulation.intervene(&Intervention::Rain { sites: 4, amount: 1.0 }).unwrap();
    let after: f32 = (0..100).map(|i| simulation.grid().hydrology().cloud(i % 10, i / 10)).sum();
    assert!(after > clouds + 3.0);
    let sources = |s: &Simulation| (0..100).filter(|i| matches!(s.grid().get_entity(i % 10, i / 10), crate::habitat::Entity::HeatSource(_))).count();
    let before = sources(&simulation);
    simulation
        .intervene(&Intervention::Scatter { resource: ResourceKind::HeatSource, count: 3, amount: 8.0 })
        .unwrap();
    assert!(sources(&simulation) > before);
    for _ in 0..300 {
        simulation.step(Duration::from_millis(16));
    }
    coherent(&simulation);
}

#[test]
fn redefining_the_catalog_changes_the_living_cells_without_losing_them() {
    let mut simulation = simulation(edited(|d| d.species_mut().founders_mut().push(group(8))));
    let genomes: Vec<u64> = simulation.cells().iter().map(|c| c.genome().id()).collect();
    let before = simulation.cells()[0].levels().get(crate::genetics::Product::Metabolism);
    let faster = edited(|d| {
        d.species_mut().founders_mut().push(group(8));
        for capability in &mut d.species_mut().genetics_mut().capabilities {
            if capability.product == crate::genetics::Product::Metabolism {
                capability.contributions.iter_mut().for_each(|c| c.amount = (c.amount * 0.5).min(1.0));
            }
        }
    });
    simulation.redefine(faster).expect("redéfinition");
    // Les génomes sont intacts ; l'expression change à la prochaine seconde de vie.
    assert_eq!(simulation.cells().iter().map(|c| c.genome().id()).collect::<Vec<_>>(), genomes);
    for _ in 0..80 {
        simulation.step(Duration::from_millis(16));
    }
    let after = simulation.cells()[0].levels().get(crate::genetics::Product::Metabolism);
    assert!(after < before, "{before} puis {after}");
    coherent(&simulation);
}

#[test]
fn an_invalid_redefinition_leaves_the_experiment_untouched() {
    let mut simulation = simulation(Definitions::builtin());
    let bigger = edited(|d| d.world_mut().size = 12);
    assert!(simulation.redefine(bigger).unwrap_err().contains("taille du monde"));
    let mut broken = (*Definitions::builtin()).clone();
    broken.species_mut().genetics_mut().capabilities[0].contributions[0].when =
        vec![crate::definitions::ConditionDefinition::Count { motif: "inconnu".into(), at_least: 1, at_most: None }];
    assert!(broken.validate().is_err());
    assert!(simulation.redefine(std::sync::Arc::new(broken)).is_err());
    coherent(&simulation);
    assert_eq!(simulation.cells().len(), 1);
}

#[test]
fn a_part_appears_only_when_the_medium_satisfies_its_conditions() {
    use crate::definitions::SignalRange;
    use crate::genetics::{Signal, Signals};
    let definitions = edited(|d| {
        let part = d.species_mut().parts_mut().iter_mut().find(|p| p.name == "vacuole").unwrap();
        part.when = vec![SignalRange { signal: Signal::Water, at_least: 0.5, at_most: None }];
    });
    let mut cell = crate::biology::Cell::new(0, 0, definitions.species().clone());
    let has_vacuole = |cell: &crate::biology::Cell| {
        cell.morphology().anatomy().parts().iter().any(|p| definitions.species().parts()[p.part()].name == "vacuole")
    };
    let mut values = [0.0; Signal::COUNT];
    cell.develop(Signals::new(values));
    assert!(!has_vacuole(&cell));
    values[Signal::Water.index()] = 1.0;
    cell.develop(Signals::new(values));
    assert!(has_vacuole(&cell));
}
