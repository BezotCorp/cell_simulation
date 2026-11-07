use rand::{SeedableRng, rngs::StdRng};

#[test]
fn repeated_mutations_preserve_viable_parameter_ranges() {
    let mut genes = mutation_prone_genes();
    let mut rng = StdRng::seed_from_u64(42);
    for _ in 0..100_000 {
        genes.mutate(
            &mut rng,
            &crate::definitions::Definitions::builtin().species().rules(),
        );
        assert!(
            genes.metabolism_rate()
                >= crate::definitions::Definitions::builtin()
                    .species()
                    .rules()
                    .basal_metabolism()
        );
        assert!(
            genes.division_threshold()
                <= crate::definitions::Definitions::builtin()
                    .species()
                    .rules()
                    .energy_capacity()
        );
        assert!(
            genes.division_threshold()
                >= 2.0
                    * crate::definitions::Definitions::builtin()
                        .species()
                        .rules()
                        .basal_metabolism()
                    * crate::definitions::Definitions::builtin()
                        .species()
                        .rules()
                        .maturity_seconds() as f32
        );
        assert!((0.0..=1.0).contains(&genes.mutation_rate()));
        assert!((0.0..=1.0).contains(&genes.water_adaptation()));
        assert!((0.0..=1.0).contains(&genes.land_adaptation()));
        assert!(genes.neuron_complexity() >= 1);
    }
}

#[test]
fn neural_complexity_can_grow_from_the_primitive_network() {
    let mut genes = mutation_prone_genes();
    let mut rng = StdRng::seed_from_u64(42);
    for _ in 0..100 {
        genes.mutate(
            &mut rng,
            &crate::definitions::Definitions::builtin().species().rules(),
        );
        if genes.neuron_complexity() > 1 {
            return;
        }
    }
    panic!("la complexité primitive ne doit pas rester bloquée à un");
}

fn mutation_prone_genes() -> crate::biology::CellGenes {
    let text = include_str!("../../../data/definitions.ron")
        .replace("mutation_rate: 0.1", "mutation_rate: 1.0");
    let definitions: crate::definitions::Definitions = ron::from_str(&text).unwrap();
    definitions.species().initial_genes().clone()
}
