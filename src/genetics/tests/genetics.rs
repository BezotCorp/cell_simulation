use crate::definitions::{Definitions, GeneticsDefinition};
use crate::genetics::{Genetics, Genome, Levels, Product, Signal, Signals};

/// Règles d'essai : trois motifs et les capacités décrites par `capabilities` (texte RON).
fn rules(capabilities: &str) -> Genetics {
    let text = format!(
        r#"(
            motifs: {{
                "a": (bits: "101"),
                "b": (bits: "1100", orientation: Reverse),
                "c": (bits: "1111", orientation: Both),
                "long": (bits: "11010011"),
            }},
            capabilities: [{capabilities}],
            neural: (gain: 0.8, bias: -20.0, sensors: [("c", Light)]),
            mutation: (bit_flip: 0.001, duplication: 0.1, deletion: 0.1, insertion: 0.1, inversion: 0.1,
                       translocation: 0.1, run_max: 3, min_length: 2, max_length: 40, repair_gain: 4.0),
            costs: (replication_per_sequence: 0.1, fidelity_surcharge: 1.0, upkeep_per_sequence: 0.01, transfer: 1.0),
        )"#
    );
    let definition: GeneticsDefinition = ron::from_str(&text).expect("règles d'essai valides");
    Genetics::compile(&definition).expect("compilation")
}

fn genome(genetics: &Genetics, sequences: &[&str]) -> Genome {
    let bytes: Vec<u8> = sequences.iter().map(|s| u8::from_str_radix(s, 2).unwrap()).collect();
    Genome::from_bytes(genetics, &bytes)
}

fn level(genetics: &Genetics, genome: &Genome, product: Product, signals: &Signals) -> f32 {
    let levels: Levels = genetics.express(genome, signals);
    levels.get(product)
}

fn neutral() -> Signals {
    Signals::new([0.0; Signal::COUNT])
}

const NEUTRAL_BYTE: &str = "00000000";

#[test]
fn motifs_are_found_in_their_orientation_and_occurrences_may_overlap() {
    let genetics = rules(
        r#"(product: Metabolism, contributions: [(when: [Count(motif: "a", at_least: 1)], amount: 0.1)]),
           (product: Size, contributions: [(when: [Count(motif: "b", at_least: 1)], amount: 0.1)]),
           (product: Repair, contributions: [(when: [Count(motif: "c", at_least: 1)], amount: 0.1)])"#,
    );
    let interpretation = |sequence: &str| genome(&genetics, &[sequence]).interpretation(&genetics).into_owned();
    // « a » = 101 (avant) : 10101010 le contient à quatre positions qui se chevauchent.
    assert_eq!(interpretation("10101010").count(0), 3);
    // « b » = 1100 lu à l'envers : c'est 0011 dans le sens d'écriture.
    assert_eq!(interpretation("00110000").count(1), 1);
    assert_eq!(interpretation("11000000").count(1), 0);
    // « c » = 1111 dans les deux sens : un seul comptage par fenêtre.
    assert_eq!(interpretation("11111111").count(2), 5);
    // Un octet peut porter plusieurs motifs à la fois.
    let both = interpretation("10100110");
    assert!(both.count(0) > 0 && both.count(1) > 0);
}

#[test]
fn a_capability_is_the_sum_of_the_contributions_whose_conditions_hold() {
    let genetics = rules(
        r#"(product: Hydration, contributions: [
              (when: [Count(motif: "a", at_least: 1)], amount: 0.25),
              (when: [Count(motif: "a", at_least: 2)], amount: 0.25),
              (when: [Count(motif: "a", at_least: 4)], amount: 0.5),
           ])"#,
    );
    let expressed = |sequences: &[&str]| level(&genetics, &genome(&genetics, sequences), Product::Hydration, &neutral());
    let a = "10100000";
    // La forme primitive est fréquente ; les formes développées demandent des copies.
    assert_eq!(expressed(&[NEUTRAL_BYTE]), 0.0);
    assert_eq!(expressed(&[a]), 0.25);
    assert_eq!(expressed(&[a, a]), 0.5);
    assert_eq!(expressed(&[a, NEUTRAL_BYTE, a]), 0.5);
    assert_eq!(expressed(&[a, a, a, a]), 1.0);
    // Les quantités restent dans [0, 1].
    assert_eq!(expressed(&[a, a, a, a, a, a]), 1.0);
}

#[test]
fn conditions_can_require_order_position_combination_repetition_and_absence() {
    let genetics = rules(
        r#"(product: Pigment, contributions: [(when: [Ordered(first: "a", then: "long")], amount: 0.5)]),
           (product: Thermal, contributions: [(when: [SameSequence(motifs: ["a", "c"])], amount: 0.5)]),
           (product: Raptorial, contributions: [(when: [Repeated(motif: "long", copies: 2)], amount: 0.5)]),
           (product: Adhesin, contributions: [(when: [Within(motif: "a", sequences: 2)], amount: 0.5)]),
           (product: Motor, contributions: [(when: [Count(motif: "a", at_least: 1, at_most: Some(1))], amount: 0.5)])"#,
    );
    let (a, long, both) = ("10100000", "11010011", "10111110");
    let at = |sequences: &[&str], product| level(&genetics, &genome(&genetics, sequences), product, &neutral());
    // L'ordre : « a » avant « long » compte, l'inverse non.
    assert_eq!(at(&[a, NEUTRAL_BYTE, long], Product::Pigment), 0.5);
    assert_eq!(at(&[long, NEUTRAL_BYTE, a], Product::Pigment), 0.0);
    // La combinaison : dans une même séquence seulement.
    assert_eq!(at(&[both], Product::Thermal), 0.5);
    assert_eq!(at(&[a, "11110000"], Product::Thermal), 0.0);
    // La répétition : deux séquences complètes identiques portant le motif.
    assert_eq!(at(&[long, NEUTRAL_BYTE, long], Product::Raptorial), 0.5);
    assert_eq!(at(&[long, NEUTRAL_BYTE, "11010011"], Product::Raptorial), 0.5);
    assert_eq!(at(&[long, NEUTRAL_BYTE], Product::Raptorial), 0.0);
    // La position : le motif doit apparaître dans les deux premières séquences.
    assert_eq!(at(&[NEUTRAL_BYTE, a, NEUTRAL_BYTE], Product::Adhesin), 0.5);
    assert_eq!(at(&[NEUTRAL_BYTE, NEUTRAL_BYTE, a], Product::Adhesin), 0.0);
    // L'absence : un motif présent au plus une fois (un répresseur le ferait taire au-delà).
    assert_eq!(at(&[a], Product::Motor), 0.5);
    assert_eq!(at(&[a, a], Product::Motor), 0.0);
}

#[test]
fn some_contributions_depend_on_the_environment_and_others_only_on_the_genome() {
    let genetics = rules(
        r#"(product: Pigment, contributions: [
              (when: [Count(motif: "a", at_least: 1)], amount: 0.25),
              (when: [Count(motif: "a", at_least: 1), Signal(signal: Light, at_least: 0.5)], amount: 0.5),
           ])"#,
    );
    let genome = genome(&genetics, &["10100000"]);
    let signals = |light: f32| {
        let mut values = [0.0; Signal::COUNT];
        values[Signal::Light.index()] = light;
        Signals::new(values)
    };
    // La part du génome est constante ; celle du milieu s'ajoute seulement en pleine lumière.
    assert_eq!(level(&genetics, &genome, Product::Pigment, &signals(0.0)), 0.25);
    assert_eq!(level(&genetics, &genome, Product::Pigment, &signals(0.4)), 0.25);
    assert_eq!(level(&genetics, &genome, Product::Pigment, &signals(0.9)), 0.75);
}

#[test]
fn a_dose_makes_the_effect_proportional_to_the_number_of_copies_up_to_a_limit() {
    let genetics = rules(
        r#"(product: Neural, contributions: [(amount: 0.125, per_copy: Some((motif: "a", up_to: 3)))])"#,
    );
    let at = |copies: usize| {
        let sequences: Vec<&str> = std::iter::repeat_n("10100000", copies).collect();
        level(&genetics, &genome(&genetics, &sequences), Product::Neural, &neutral())
    };
    assert_eq!([at(0), at(1), at(2), at(3), at(6)], [0.0, 0.125, 0.25, 0.375, 0.375]);
}

#[test]
fn the_neural_sensor_follows_the_first_sensor_motif_along_the_genome() {
    let genetics = rules("");
    let plain = genome(&genetics, &["00000000"]);
    assert_eq!(plain.interpretation(&genetics).sensor(), crate::genetics::Sensor::Temperature);
    let lit = genome(&genetics, &["00000000", "11110000"]);
    assert_eq!(lit.interpretation(&genetics).sensor(), crate::genetics::Sensor::Light);
}

#[test]
fn the_catalog_ancestor_has_primitive_functions_and_nothing_more() {
    let species = Definitions::builtin().species().clone();
    let genetics = species.genetics().clone();
    let ancestor = species.ancestor_genome();
    assert_eq!(ancestor.len(), 24);
    let levels = genetics.express(&ancestor, &neutral());
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
        (Product::Thermal, 0.0),
        (Product::Raptorial, 0.0),
        (Product::Adhesin, 0.0),
        (Product::Conjugation, 0.0),
        (Product::Recombination, 0.0),
    ] {
        assert!((levels.get(product) - expected).abs() < 1e-6, "{product:?} {}", levels.get(product));
    }
}

#[test]
fn a_bad_catalog_is_refused_with_a_clear_reason() {
    let definition = |text: &str| -> Result<Genetics, String> {
        let text = text.replace("MOTIFS", r#""a": (bits: "101")"#);
        let definition: GeneticsDefinition = ron::from_str(&format!(
            r#"(motifs: {{ {text} }}, capabilities: [], neural: (gain: 1.0, bias: 0.0),
                mutation: (bit_flip: 0.001, duplication: 0.1, deletion: 0.1, insertion: 0.1, inversion: 0.1,
                           translocation: 0.1, run_max: 3, min_length: 2, max_length: 40, repair_gain: 4.0),
                costs: (replication_per_sequence: 0.1, fidelity_surcharge: 1.0, upkeep_per_sequence: 0.01, transfer: 1.0))"#
        ))
        .unwrap();
        Genetics::compile(&definition)
    };
    assert!(definition("MOTIFS").is_ok());
    assert!(definition(r#""x": (bits: "10201")"#).is_err());
    assert!(definition(r#""x": (bits: "101010101")"#).is_err());
    assert!(definition(r#""x": (bits: "")"#).is_err());
    // Un motif cité par une règle doit exister.
    let unknown: GeneticsDefinition = ron::from_str(
        r#"(motifs: {"a": (bits: "101")}, neural: (gain: 1.0, bias: 0.0),
            capabilities: [(product: Size, contributions: [(when: [Count(motif: "absent", at_least: 1)], amount: 1.0)])],
            mutation: (bit_flip: 0.001, duplication: 0.1, deletion: 0.1, insertion: 0.1, inversion: 0.1,
                       translocation: 0.1, run_max: 3, min_length: 2, max_length: 40, repair_gain: 4.0),
            costs: (replication_per_sequence: 0.1, fidelity_surcharge: 1.0, upkeep_per_sequence: 0.01, transfer: 1.0))"#,
    )
    .unwrap();
    assert!(Genetics::compile(&unknown).unwrap_err().contains("absent"));
}
