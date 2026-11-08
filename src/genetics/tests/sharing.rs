use crate::definitions::Definitions;
use crate::genetics::{Genetics, Genome, Product, Signal, Signals};
use rand::{SeedableRng, rngs::StdRng};
use std::sync::Arc;

fn catalog() -> (Arc<Genetics>, Genome) {
    let species = Definitions::builtin().species().clone();
    (species.genetics().clone(), species.ancestor_genome())
}

fn neutral() -> Signals {
    Signals::new([0.0; Signal::COUNT])
}

#[test]
fn clones_and_equal_fragments_are_shared_in_memory_and_edits_create_new_versions() {
    let (genetics, ancestor) = catalog();
    assert_eq!(ancestor.chunks().len(), 2);
    // Un clone est le même objet ; un génome reconstruit des mêmes séquences réutilise les fragments.
    assert!(ancestor.clone().is_same_object(&ancestor));
    let again = Genome::from_bytes(&genetics, &ancestor.bytes());
    assert_eq!(again.shared_chunks(&ancestor), 2);
    assert_eq!(again.id(), ancestor.id());
    // Une erreur de copie dans une séquence ne recrée que le fragment concerné.
    let mut bytes = ancestor.bytes();
    bytes[3] ^= 0b0000_0100;
    let mutated = ancestor.splice(&genetics, 3, 4, &[bytes[3]]);
    assert_eq!(mutated.shared_chunks(&ancestor), ancestor.chunks().len() - 1);
    assert_ne!(mutated.id(), ancestor.id());
    assert_eq!(mutated.bytes(), bytes);
    // Le génome d'origine n'a pas bougé : les modifications créent des versions.
    assert_eq!(ancestor.len(), 24);
    assert_ne!(ancestor.bytes(), mutated.bytes());
}

#[test]
fn an_edit_in_the_middle_keeps_every_untouched_fragment_even_after_insertions_and_deletions() {
    let (genetics, _) = catalog();
    let bytes: Vec<u8> = (0..96u8).map(|b| b.wrapping_mul(37).wrapping_add(11)).collect();
    let genome = Genome::from_bytes(&genetics, &bytes);
    assert_eq!(genome.chunks().len(), 8);
    let longer = genome.splice(&genetics, 40, 40, &[1, 2, 3, 4, 5]);
    assert_eq!(longer.len(), 101);
    assert!(longer.shared_chunks(&genome) >= 6, "{}", longer.shared_chunks(&genome));
    let shorter = genome.splice(&genetics, 40, 44, &[]);
    assert_eq!(shorter.len(), 92);
    assert!(shorter.shared_chunks(&genome) >= 6);
    // Retirer puis réinsérer le même fragment redonne le même génome, fragments inclus.
    let restored = shorter.splice(&genetics, 40, 40, &genome.slice(40, 44));
    assert_eq!(restored, genome);
    assert_eq!(restored.id(), genome.id());
}

#[test]
fn interpretation_is_computed_once_per_genome_and_shared_by_every_clone() {
    let (genetics, ancestor) = catalog();
    let clone = ancestor.clone();
    assert!(!clone.is_interpreted());
    let _ = ancestor.interpretation(&genetics);
    assert!(clone.is_interpreted(), "le résultat est partagé avec les copies");
    // Deux interprétations d'un même génome sont le même objet : rien n'est recalculé.
    let first = ancestor.interpretation(&genetics);
    let second = clone.interpretation(&genetics);
    assert!(std::ptr::eq(&*first, &*second));
}

#[test]
fn interpreting_many_genomes_in_parallel_gives_exactly_the_sequential_result() {
    let (genetics, ancestor) = catalog();
    let mut rng = StdRng::seed_from_u64(9);
    let genomes: Vec<Genome> = (0..300)
        .map(|_| genetics.reproduce(&ancestor, &mut rng, 0.0, 12.0))
        .collect();
    let references: Vec<&Genome> = genomes.iter().collect();
    // Copies indépendantes (mêmes octets, objets distincts) pour comparer les deux voies.
    let sequential: Vec<Genome> = genomes.iter().map(|g| Genome::from_bytes(&Genetics::compile(Definitions::builtin().species().genetics_definition()).unwrap(), &g.bytes())).collect();
    assert_eq!(genetics.interpret_all(&references, 4), genomes.len());
    assert_eq!(genetics.interpret_all(&references, 4), 0, "tout est déjà interprété");
    for (parallel, copy) in genomes.iter().zip(&sequential) {
        let other = Genetics::compile(Definitions::builtin().species().genetics_definition()).unwrap();
        let a = genetics.express(parallel, &neutral());
        let b = other.express(copy, &neutral());
        assert_eq!(a, b);
    }
}

#[test]
fn genomes_survive_serialization_and_reenter_the_shared_registry() {
    let (genetics, ancestor) = catalog();
    let mut rng = StdRng::seed_from_u64(3);
    let genome = genetics.reproduce(&ancestor, &mut rng, 0.0, 30.0);
    let configuration = bincode_next::config::standard();
    let encoded = bincode_next::serde::encode_to_vec(&genome, configuration).unwrap();
    let (decoded, _): (Genome, usize) = bincode_next::serde::decode_from_slice(&encoded, configuration).unwrap();
    assert_eq!(decoded, genome);
    assert_eq!(decoded.id(), genome.id());
    // Lu dans un fichier, un génome n'est pas encore dans le registre ; il y revient, et tous les
    // génomes ainsi relus partagent leurs fragments identiques.
    assert_eq!(decoded.shared_chunks(&genome), 0);
    let (first, second) = (decoded.interned(&genetics), genome.interned(&genetics));
    assert_eq!(first.shared_chunks(&second), first.chunks().len());
    assert_eq!(first.id(), genome.id(), "l'identité ne dépend pas du découpage en fragments");
}

#[test]
fn mutation_opportunities_follow_length_exposure_and_repair() {
    let (genetics, ancestor) = catalog();
    let base = genetics.expected_mutations(&ancestor, 0.0, 1.0);
    assert!(base > 0.0);
    assert_eq!(genetics.expected_mutations(&ancestor, 0.0, 0.0), 0.0);
    assert!(genetics.expected_mutations(&ancestor, 0.0, 2.0) > base);
    assert!(genetics.expected_mutations(&ancestor, 1.0, 1.0) < base);
    // Un génome plus long offre plus de bits à copier de travers.
    let longer = ancestor.splice(&genetics, 0, 0, &ancestor.bytes());
    assert_eq!(longer.len(), 48);
    assert!(genetics.expected_mutations(&longer, 0.0, 1.0) > base);
}

#[test]
fn a_larger_population_makes_more_mutations_and_the_measured_rate_matches_the_model() {
    let (genetics, ancestor) = catalog();
    let mutated = |births: usize, seed: u64| {
        let mut rng = StdRng::seed_from_u64(seed);
        (0..births)
            .filter(|_| genetics.reproduce(&ancestor, &mut rng, 0.25, 1.0) != ancestor)
            .count()
    };
    let (small, large) = (mutated(200, 3), mutated(4_000, 3));
    assert!(large > small * 10, "{small} contre {large}");
    let lambda = genetics.expected_mutations(&ancestor, 0.25, 1.0);
    // La part de filles mutées est 1 − e^−λ (arrondie ici à λ pour de petits taux, sans les événements nuls).
    let measured = large as f64 / 4_000.0;
    assert!((measured - lambda).abs() < 0.12, "{measured} contre {lambda}");
}

#[test]
fn without_exposure_the_daughter_is_the_very_same_genome_in_memory() {
    let (genetics, ancestor) = catalog();
    let mut rng = StdRng::seed_from_u64(1);
    assert!(genetics.reproduce(&ancestor, &mut rng, 0.0, 0.0).is_same_object(&ancestor));
}

#[test]
fn copy_errors_flip_bits_in_both_directions_and_never_aim_at_usefulness() {
    let (genetics, ancestor) = catalog();
    let mut rng = StdRng::seed_from_u64(11);
    let (mut zero_to_one, mut one_to_zero) = (0u32, 0u32);
    let reference = ancestor.bytes();
    for _ in 0..4_000 {
        let daughter = genetics.reproduce(&ancestor, &mut rng, 0.0, 6.0);
        if daughter.len() == ancestor.len() {
            for (before, after) in reference.iter().zip(daughter.bytes()) {
                zero_to_one += (!before & after).count_ones();
                one_to_zero += (before & !after).count_ones();
            }
        }
    }
    assert!(zero_to_one > 100 && one_to_zero > 100);
    let ratio = f64::from(zero_to_one) / f64::from(one_to_zero);
    assert!((0.8..1.25).contains(&ratio), "{zero_to_one} apparitions de 1, {one_to_zero} pertes");
}

/// Règles du catalogue dont seuls les événements de `mutation` (texte RON de taux) sont possibles.
fn only(changes: impl Fn(&mut crate::definitions::MutationDefinition)) -> Genetics {
    let mut definition = Definitions::builtin().species().genetics_definition().clone();
    definition.mutation.bit_flip = 0.0;
    definition.mutation.duplication = 0.0;
    definition.mutation.deletion = 0.0;
    definition.mutation.insertion = 0.0;
    definition.mutation.inversion = 0.0;
    definition.mutation.translocation = 0.0;
    changes(&mut definition.mutation);
    Genetics::compile(&definition).unwrap()
}

#[test]
fn each_kind_of_reorganization_does_what_it_says_and_stays_within_the_length_bounds() {
    let ancestor = Definitions::builtin().species().ancestor_genome();
    let bytes = ancestor.bytes();
    let mut sorted = bytes.clone();
    sorted.sort_unstable();
    let mut rng = StdRng::seed_from_u64(5);
    let run = |genetics: &Genetics, rng: &mut StdRng| {
        let genome = Genome::from_bytes(genetics, &bytes);
        genetics.reproduce(&genome, rng, 0.0, 1.0)
    };
    let duplicating = only(|m| m.duplication = 2.0);
    let deleting = only(|m| m.deletion = 2.0);
    let inserting = only(|m| m.insertion = 2.0);
    let inverting = only(|m| m.inversion = 2.0);
    let moving = only(|m| m.translocation = 2.0);
    let (mut grew, mut shrank, mut inserted, mut inverted, mut moved) = (0, 0, 0, 0, 0);
    for _ in 0..400 {
        let d = run(&duplicating, &mut rng);
        grew += usize::from(d.len() > bytes.len());
        // Une duplication ne perd aucune séquence : l'originale subsiste dans l'ordre.
        assert!(d.len() >= bytes.len());
        let x = run(&deleting, &mut rng);
        shrank += usize::from(x.len() < bytes.len());
        assert!(x.len() >= deleting.mutation().min_length);
        let i = run(&inserting, &mut rng);
        inserted += usize::from(i.len() > bytes.len());
        assert!(i.len() <= inserting.mutation().max_length);
        let v = run(&inverting, &mut rng);
        assert_eq!(v.len(), bytes.len());
        inverted += usize::from(v.bytes() != bytes);
        let t = run(&moving, &mut rng);
        assert_eq!(t.len(), bytes.len());
        let mut moved_bytes = t.bytes();
        moved_bytes.sort_unstable();
        // Un déplacement garde exactement les mêmes séquences, dans un autre ordre.
        assert_eq!(moved_bytes, sorted);
        moved += usize::from(t.bytes() != bytes);
    }
    assert!(grew > 100 && shrank > 100 && inserted > 100 && inverted > 100 && moved > 100, "{grew} {shrank} {inserted} {inverted} {moved}");
}

#[test]
fn genetic_exchange_recombines_transfers_and_fuses_genomes_while_sharing_their_fragments() {
    let (genetics, ancestor) = catalog();
    let bytes: Vec<u8> = (0..30u8).map(|b| b.wrapping_mul(53).wrapping_add(7)).collect();
    let other = Genome::from_bytes(&genetics, &bytes);
    let mut rng = StdRng::seed_from_u64(2);
    let child = genetics.recombine(&ancestor, &other, &mut rng);
    assert!(child.len() >= 2);
    let (head, tail) = (child.bytes(), other.bytes());
    assert_eq!(&head[..1], &ancestor.bytes()[..1]);
    assert_eq!(head.last(), tail.last());
    let transferred = genetics.transfer(&ancestor, &other, &mut rng);
    assert!(transferred.len() > ancestor.len() && transferred.len() <= ancestor.len() + genetics.mutation().run_max);
    assert!(transferred.shared_chunks(&ancestor) >= ancestor.chunks().len() - 1);
    let fused = genetics.fuse(&ancestor, &other);
    assert_eq!(fused.len(), ancestor.len() + other.len());
    assert_eq!(fused.slice(0, ancestor.len()), ancestor.bytes());
    assert_eq!(fused.slice(ancestor.len(), fused.len()), other.bytes());
}

#[test]
fn replication_and_upkeep_cost_more_for_long_and_faithful_genomes() {
    let (genetics, ancestor) = catalog();
    let plain = genetics.replication_cost(&ancestor, 0.0);
    assert!(plain > 0.0);
    assert!(genetics.replication_cost(&ancestor, 1.0) > plain, "la fidélité se paie");
    let doubled = ancestor.splice(&genetics, 0, 0, &ancestor.bytes());
    assert!((genetics.replication_cost(&doubled, 0.0) - 2.0 * plain).abs() < 1e-5);
    // L'entretien se mesure à la longueur de l'ancêtre : nul pour lui, positif au-delà, négatif en deçà.
    assert_eq!(genetics.upkeep(&ancestor, ancestor.len()), 0.0);
    assert!(genetics.upkeep(&doubled, ancestor.len()) > 0.0);
    assert!(genetics.upkeep(&ancestor.splice(&genetics, 0, 4, &[]), 24) < 0.0);
    let _ = Product::Size;
}

/// Mesures du coût réel de la génétique (lancées à la main : `cargo test --release measure -- --ignored --nocapture`).
#[test]
#[ignore = "mesures de performance"]
fn measure_the_cost_of_interpreting_and_sharing_genomes() {
    use std::time::Instant;
    let (genetics, ancestor) = catalog();
    let mut rng = StdRng::seed_from_u64(42);
    // Une population réaliste : 20 000 génomes issus de lignées qui mutent à chaque génération.
    let mut population: Vec<Genome> = vec![ancestor.clone(); 64];
    while population.len() < 20_000 {
        let parent = population[rng_index(&mut rng, population.len())].clone();
        population.push(genetics.reproduce(&parent, &mut rng, 0.25, 1.0));
    }
    let distinct: std::collections::HashSet<u64> = population.iter().map(|g| g.id()).collect();
    let bytes: usize = population.iter().map(|g| g.len()).sum();
    let chunk_bytes: usize = {
        let mut seen = std::collections::HashSet::new();
        population
            .iter()
            .flat_map(|g| g.chunks().iter())
            .filter(|c| seen.insert(std::sync::Arc::as_ptr(c)))
            .map(|c| c.len())
            .sum()
    };
    let objects = {
        let mut seen = std::collections::HashSet::new();
        population.iter().filter(|g| seen.insert(g.id())).count()
    };
    eprintln!(
        "mémoire : {} cellules, {} génomes distincts ({objects} objets), {} octets de séquences si chacune avait le sien, {} octets de fragments partagés ({:.1}x moins)",
        population.len(),
        distinct.len(),
        bytes,
        chunk_bytes,
        bytes as f64 / chunk_bytes as f64
    );
    let (reused, created) = genetics.pool().statistics();
    eprintln!("registre : {reused} fragments retrouvés, {created} créés");

    // Interprétation à froid (fragments nouveaux) puis à chaud (déjà calculée).
    let fresh: Vec<Genome> = population.iter().map(|g| Genome::from_bytes(&genetics, &g.bytes())).collect();
    let start = Instant::now();
    for genome in &fresh {
        let _ = genetics.express(genome, &neutral());
    }
    let cold = start.elapsed();
    let start = Instant::now();
    for genome in &fresh {
        let _ = genetics.express(genome, &neutral());
    }
    let hot = start.elapsed();
    eprintln!(
        "interprétation : {:.2} µs par génome à froid, {:.3} µs à chaud (expression de 15 produits comprise)",
        cold.as_secs_f64() * 1e6 / fresh.len() as f64,
        hot.as_secs_f64() * 1e6 / fresh.len() as f64
    );

    // Répartition sur plusieurs fils d'exécution des génomes à interpréter.
    let isolated = |threads: usize| {
        let genetics = Genetics::compile(Definitions::builtin().species().genetics_definition()).unwrap();
        let genomes: Vec<Genome> = population.iter().map(|g| Genome::from_bytes(&genetics, &g.bytes())).collect();
        let refs: Vec<&Genome> = genomes.iter().collect();
        let start = Instant::now();
        genetics.interpret_all(&refs, threads);
        start.elapsed()
    };
    for threads in [1, 2, 4, 8] {
        eprintln!("interpret_all sur {threads} fil(s) : {:.2} ms pour {} génomes", isolated(threads).as_secs_f64() * 1e3, population.len());
    }
}

fn rng_index(rng: &mut StdRng, len: usize) -> usize {
    use rand::RngExt;
    rng.random_range(0..len)
}
