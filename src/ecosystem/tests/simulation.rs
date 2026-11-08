use crate::definitions::Definitions;
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
        // Débit maximal par site : 2 unités/s de nourriture, 3 organismes de photosynthèse pleine
        // sous la plus forte lumière (10,8 unités/s) et 2 de thermosynthèse (7,2 unités/s) ; entretien
        // minimal d'un organisme (métabolisme 0,5, socle de taille 0,4) : 0,2 unité/s. Cette enveloppe de régression ne constitue pas une limite imposée
        // au moteur : elle détecte un emballement, pas une capacité.
        let population_envelope = 8_500;
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
        // Le génome a divergé de celui de l'ancêtre dans au moins un organisme.
        let ancestor = Definitions::builtin().species().ancestor_genome();
        neural_variation |= simulation.cells().iter().any(|cell| *cell.genome() != ancestor);
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
        assert_eq!(a.genome(), b.genome());
        assert_eq!(a.levels(), b.levels());
        assert_eq!(a.morphology(), b.morphology());
        assert_eq!(a.physiology(), b.physiology());
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
        crate::habitat::Environment::Land => 1.0 - cell.physiology().land_adaptation(),
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
        let simulation = seeded(10, 3);
        let hydrology = simulation
            .grid()
            .hydrology()
            .clone()
            .with_uniform_current(east, 0.0, 1.0);
        let mut simulation = simulation.with_hydrology(hydrology);
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

#[test]
#[ignore = "observation de l'évolution, lancée à la main"]
fn what_do_the_lineages_evolve() {
    use crate::genetics::Product;
    for seed in 0..8 {
        let mut simulation = seeded(10, seed);
        for tick in 0..450_000u32 {
            simulation.step(Duration::from_millis(16));
            if tick % 150_000 == 149_999 || tick == 449_999 {
                let counts = simulation.census(0.2);
                let cells = simulation.cells();
                let genes: usize = cells.iter().map(|c| c.genome_length()).sum();
                let sizes: f32 = cells.iter().map(|c| c.morphology().size()).sum();
                eprintln!(
                    "graine {seed} t={:>6}s cellules {:>4} séquences/org {:.1} taille {:.2} photo {:>4} thermo {:>4} prédation {:>4} neurones {:>4} adhésion {:>4} sessiles {:>4} liaisons {:>3}",
                    tick / 62,
                    cells.len(),
                    genes as f32 / cells.len().max(1) as f32,
                    sizes / cells.len().max(1) as f32,
                    counts[Product::Pigment.index()],
                    counts[Product::Thermal.index()],
                    counts[Product::Raptorial.index()],
                    counts[Product::Neural.index()],
                    counts[Product::Adhesin.index()],
                    cells.iter().filter(|c| c.is_sessile()).count(),
                    simulation.bonds().len(),
                );
            }
        }
    }
}

/// Simulation dont l'ancêtre est celui du catalogue après ajout et retrait de séquences porteuses de motifs.
fn founded_with(edits: (&[&str], &[&str]), seed: u64) -> Simulation {
    Simulation::from_definitions(crate::test_support::catalog(edits.0, edits.1), seed).unwrap()
}

/// Ancêtre qui vit de lumière (pigments à la place de la digestion), sait vivre hors de l'eau
/// (cuticule développée) et dont le métabolisme est bas : il doit tenir une nuit sur ses réserves.
const PHOTOTROPH: (&[&str], &[&str]) = (
    &["pig_a", "pig_a", "cuticle", "cuticle", "cuticle"],
    &["digest", "digest", "metab"],
);
/// Ancêtre dont des séquences identiques portent une protéine d'adhésion (une duplication).
const ADHESIVE: (&[&str], &[&str]) = (&["adh_a", "adh_a", "adh_a"], &[]);

#[test]
fn a_lineage_can_live_on_light_alone_and_light_bounds_its_growth() {
    // Ni ingestion ni nourriture : seule la lumière, partagée entre les occupants de chaque site.
    let mut simulation = founded_with(PHOTOTROPH, 2);
    let sites = (simulation.grid().size() * simulation.grid().size()) as f32;
    let mut peak = 0;
    for _ in 0..100_000 {
        simulation.step(Duration::from_millis(16));
        peak = peak.max(simulation.cells().len());
    }
    assert!(simulation.cells().len() > 50, "la lignée doit subsister : {}", simulation.cells().len());
    assert!(simulation.generation() > 100);
    // Trois organismes à photosynthèse pleine par site au plus à midi : l'énergie lumineuse est finie.
    assert!((peak as f32) < sites * 3.0 * 3.6 / 0.5, "croissance sans frein : {peak}");
}

#[test]
fn adhesive_lineages_form_colonies_that_stay_together_and_alive() {
    let mut simulation = founded_with(ADHESIVE, 1);
    let mut colonies_seen = false;
    for tick in 0..60_000u32 {
        simulation.step(Duration::from_millis(16));
        if tick % 300 != 0 {
            continue;
        }
        let size = simulation.grid().size() as f32;
        let living: std::collections::HashMap<_, _> = simulation
            .cells()
            .iter()
            .map(|cell| (cell.id(), cell.position()))
            .collect();
        for (a, b) in simulation.bonds().pairs() {
            let (pa, pb) = (living[a], living[b]);
            let wrap = |d: f32| {
                let d = d.abs();
                d.min(size - d)
            };
            assert!(wrap(pa.0 - pb.0).hypot(wrap(pa.1 - pb.1)) <= 1.8 + 0.05, "liaison trop étirée");
            colonies_seen = true;
        }
        for cell in simulation.cells() {
            assert!(simulation.bonds().degree(cell.id()) <= 4);
        }
    }
    assert!(colonies_seen, "des cellules adhésives doivent rester liées");
}

#[test]
fn lightning_strikes_where_the_charge_is_and_fertilizes_the_ground() {
    let simulation = seeded(10, 4);
    // Un site nu et traversable, loin du fondateur, sert de cible.
    let (x, y) = (0..100)
        .map(|i| (i % 10, i / 10))
        .find(|&(x, y)| {
            matches!(simulation.grid().get_entity(x, y), crate::habitat::Entity::Empty)
                && simulation.grid().is_passable(x, y)
        })
        .expect("le monde contient un site nu");
    let mut storm_state = vec![0.0_f32; 100];
    storm_state[y * 10 + x] = 5.0;
    let storm: crate::hydrology::Storm =
        bincode_next::serde::decode_from_slice(
            &bincode_next::serde::encode_to_vec(&storm_state, bincode_next::config::standard()).unwrap(),
            bincode_next::config::standard(),
        )
        .unwrap()
        .0;
    let mut simulation = simulation.with_storm(storm);
    for _ in 0..3_000 {
        simulation.step(Duration::from_millis(16));
        if !simulation.strikes().is_empty() {
            break;
        }
    }
    let strike = simulation.strikes().first().expect("l'éclair doit tomber");
    assert!(strike.energy() > 1.0);
    let (sx, sy) = (strike.x(), strike.y());
    // Le point le plus haut du voisinage est touché ; il reçoit un apport nutritif durable.
    assert!(sx.abs_diff(x).min(10 - sx.abs_diff(x)) <= 1 && sy.abs_diff(y).min(10 - sy.abs_diff(y)) <= 1);
    // La foudre fixe l'azote : un site nu devient site alimentaire ; un solide ou une source restent tels quels.
    assert!(!matches!(simulation.grid().get_entity(sx, sy), crate::habitat::Entity::Empty));
    assert!(simulation.grid().storm().charge(x, y, 10) < 5.0 * 0.5);
}
