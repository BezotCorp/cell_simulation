use crate::definitions::Definitions;
use crate::genetics::{Gene, Genome, Levels, NEURAL_UNITS, Product, REGULATORS, Signal, Signals};
use rand::{SeedableRng, rngs::StdRng};

fn gene(bias: f32, signals: &[(Signal, f32)], products: &[(Product, f32)], effects: &[(Product, f32)]) -> Gene {
    let mut inputs = [0.0; REGULATORS];
    for (signal, weight) in signals {
        inputs[signal.index()] = *weight;
    }
    for (product, weight) in products {
        inputs[Signal::COUNT + product.index()] = *weight;
    }
    let mut all = [0.0; Product::COUNT];
    for (product, weight) in effects {
        all[product.index()] = *weight;
    }
    Gene::new(bias, inputs, all)
}

fn genome(genes: Vec<Gene>) -> Genome {
    Genome::new(genes, [1.0; NEURAL_UNITS], 0.0, 0)
}

fn level(levels: &Levels, product: Product) -> f32 {
    levels.get(product)
}

const NEUTRAL: [f32; Signal::COUNT] = [0.0; Signal::COUNT];

#[test]
fn constitutive_genes_add_their_effects_and_the_catalog_ancestor_is_recovered() {
    let ancestor = Definitions::builtin().species().genome().to_genome();
    let levels = ancestor.express(&Signals::new(NEUTRAL));
    for (product, expected) in [
        (Product::Metabolism, 0.5),
        (Product::Size, 0.5),
        (Product::Fecundity, 0.75),
        (Product::Repair, 0.25),
        (Product::Hydration, 1.0),
        (Product::Cuticle, 0.125),
        (Product::Digestive, 1.0),
        (Product::Motor, 1.0),
        (Product::Neural, 0.0),
        (Product::Pigment, 0.0),
    ] {
        assert!((level(&levels, product) - expected).abs() < 1e-6, "{product:?}");
    }
    assert!(levels.is_valid());
}

#[test]
fn one_product_comes_from_several_genes_and_one_gene_acts_on_several_products() {
    let genes = vec![
        gene(0.5, &[], &[], &[(Product::Motor, 0.25), (Product::Hydration, 0.25)]),
        gene(0.5, &[], &[], &[(Product::Motor, 0.25)]),
    ];
    let base = genome(genes.clone()).express(&Signals::new(NEUTRAL));
    assert_eq!(level(&base, Product::Motor), 0.5);
    assert_eq!(level(&base, Product::Hydration), 0.25);
    // Retirer le premier gène change deux produits à la fois ; le second ne change que le moteur.
    let without_first = genome(genes[1..].to_vec()).express(&Signals::new(NEUTRAL));
    assert_eq!(level(&without_first, Product::Motor), 0.25);
    assert_eq!(level(&without_first, Product::Hydration), 0.0);
}

#[test]
fn the_same_genome_expresses_differently_in_different_environments() {
    // Un gène de pigments qui ne s'allume qu'à la lumière.
    let pigment = gene(-0.5, &[(Signal::Light, 1.0)], &[], &[(Product::Pigment, 1.0)]);
    let genome = genome(vec![pigment]);
    let signals = |light| {
        let mut values = NEUTRAL;
        values[Signal::Light.index()] = light;
        Signals::new(values)
    };
    let dark = level(&genome.express(&signals(0.0)), Product::Pigment);
    let dim = level(&genome.express(&signals(0.5)), Product::Pigment);
    let bright = level(&genome.express(&signals(1.0)), Product::Pigment);
    assert_eq!(dark, 0.0);
    assert!(dim > dark && bright > dim, "{dark} {dim} {bright}");
}

#[test]
fn genes_regulate_each_other_through_the_products_they_make() {
    // Le gène A fabrique des enzymes digestives ; le gène B ne s'exprime que si elles existent.
    let a = gene(0.5, &[], &[], &[(Product::Digestive, 0.5)]);
    let b = gene(-0.5, &[], &[(Product::Digestive, 2.0)], &[(Product::Raptorial, 1.0)]);
    let with_a = genome(vec![a, b.clone()]).express(&Signals::new(NEUTRAL));
    let alone = genome(vec![b]).express(&Signals::new(NEUTRAL));
    assert_eq!(level(&alone, Product::Raptorial), 0.0);
    assert!(level(&with_a, Product::Raptorial) > 0.4);
}

#[test]
fn expression_is_deterministic_bounded_and_defined_for_any_genome() {
    let mut rng = StdRng::seed_from_u64(7);
    let mut genome = Definitions::builtin().species().genome().to_genome();
    for _ in 0..300 {
        genome = genome.reproduce(&mut rng, 0.0, 6.0);
        genome.validate().unwrap();
        let signals = Signals::new([0.3, 0.9, 1.0, 0.5, 0.1, 0.7, 1.0]);
        let (first, second) = (genome.express(&signals), genome.express(&signals));
        assert_eq!(first, second);
        assert!(first.is_valid());
    }
}

#[test]
fn mutation_opportunities_follow_genome_size_exposure_and_repair() {
    let ancestor = Definitions::builtin().species().genome().to_genome();
    let base = ancestor.expected_mutations(0.0, 1.0);
    assert!(base > 0.0);
    // Un génome deux fois plus long offre deux fois plus de cibles.
    let mut longer = ancestor.clone();
    let mut rng = StdRng::seed_from_u64(1);
    while longer.genes().len() < 12 {
        longer = longer.reproduce(&mut rng, 0.0, 40.0);
    }
    assert!(longer.expected_mutations(0.0, 1.0) > base);
    // Le rayonnement augmente le taux, la réparation le diminue ; aucun milieu ne l'annule seul.
    assert!(ancestor.expected_mutations(0.0, 2.0) > base);
    assert!(ancestor.expected_mutations(1.0, 1.0) < base);
    assert_eq!(ancestor.expected_mutations(0.0, 0.0), 0.0);
}

#[test]
fn a_larger_population_produces_more_mutations_through_its_reproductions() {
    let ancestor = Definitions::builtin().species().genome().to_genome();
    let mutated = |births: usize, seed: u64| {
        let mut rng = StdRng::seed_from_u64(seed);
        (0..births)
            .filter(|_| ancestor.reproduce(&mut rng, 0.25, 1.0) != ancestor)
            .count()
    };
    let (small, large) = (mutated(200, 3), mutated(4_000, 3));
    assert!(large > small * 10, "{small} contre {large}");
    // La fréquence par naissance reste celle que prédit le modèle (≈ λ pour λ petit).
    let lambda = ancestor.expected_mutations(0.25, 1.0);
    let measured = large as f64 / 4_000.0;
    assert!((measured - lambda).abs() < 0.08, "{measured} contre {lambda}");
}

#[test]
fn mutations_are_blind_changes_in_both_directions_never_aimed_at_usefulness() {
    // Génome à mi-course sur chaque produit : aucune saturation en 0 ou 1 ne fausse la comparaison.
    let ancestor = genome(vec![
        gene(0.0, &[], &[], &Product::ALL.map(|product| (product, 0.5))),
        gene(0.0, &[], &[], &[(Product::Motor, 0.25), (Product::Size, 0.25)]),
    ]);
    let mut rng = StdRng::seed_from_u64(11);
    let (mut up, mut down, mut grown, mut shrunk, mut novel) = (0, 0, 0, 0, 0);
    for _ in 0..20_000 {
        let daughter = ancestor.reproduce(&mut rng, 0.0, 8.0);
        let (before, after) = (ancestor.express(&Signals::new(NEUTRAL)), daughter.express(&Signals::new(NEUTRAL)));
        for product in Product::ALL {
            match level(&after, product).total_cmp(&level(&before, product)) {
                std::cmp::Ordering::Greater => up += 1,
                std::cmp::Ordering::Less => down += 1,
                _ => {}
            }
            novel += usize::from(level(&before, product) == 0.0 && level(&after, product) > 0.0);
        }
        grown += usize::from(daughter.genes().len() > ancestor.genes().len());
        shrunk += usize::from(daughter.genes().len() < ancestor.genes().len());
    }
    // Autant de hausses que de baisses (à 15 % près) : aucun biais vers un mieux-être.
    assert!((up as f64 / down as f64 - 1.0).abs() < 0.15, "{up} hausses, {down} baisses");
    assert!(grown > 0 && shrunk > 0, "le génome peut croître et décroître");
    let _ = novel;
}

#[test]
fn gene_count_stays_within_bounds_whatever_the_mutations() {
    let mut rng = StdRng::seed_from_u64(5);
    let mut genome = Definitions::builtin().species().genome().to_genome();
    for _ in 0..5_000 {
        genome = genome.reproduce(&mut rng, 0.0, 30.0);
        assert!((1..=crate::genetics::MAX_GENES).contains(&genome.genes().len()));
    }
}

#[test]
fn neural_activation_needs_units_and_follows_the_wiring() {
    let mut wiring = [1.0; NEURAL_UNITS];
    wiring[0] = 0.8;
    let genome = Genome::new(vec![gene(0.5, &[], &[], &[])], wiring, -20.0, 0);
    assert_eq!(genome.neural_activation(0, 40.0), 0.0);
    assert!(genome.neural_activation(1, 40.0) > 0.0);
    assert!(genome.neural_activation(1, 10.0) < 0.0);
    assert!(genome.neural_activation(3, 40.0).is_finite());
}

#[test]
fn new_products_and_new_structures_can_appear_from_the_catalog_ancestor() {
    let ancestor = Definitions::builtin().species().genome().to_genome();
    let before = ancestor.express(&Signals::new(NEUTRAL));
    let mut rng = StdRng::seed_from_u64(21);
    let mut appeared = std::collections::BTreeSet::new();
    for _ in 0..4_000 {
        let mut lineage = ancestor.clone();
        for _ in 0..12 {
            lineage = lineage.reproduce(&mut rng, 0.0, 6.0);
        }
        let after = lineage.express(&Signals::new(NEUTRAL));
        for product in Product::ALL {
            if level(&before, product) == 0.0 && level(&after, product) > 0.1 {
                appeared.insert(product);
            }
        }
    }
    // Pigments, thermosynthèse, prédation, adhésion, tissu nerveux : rien n'est exclu d'avance.
    for product in [Product::Pigment, Product::Thermal, Product::Raptorial, Product::Adhesin, Product::Neural] {
        assert!(appeared.contains(&product), "{product:?} n'apparaît jamais");
    }
}
