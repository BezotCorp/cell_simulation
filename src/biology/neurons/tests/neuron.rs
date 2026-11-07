use crate::biology::Neuron;
use rand::{SeedableRng, rngs::StdRng};

fn parameters(neuron: &Neuron) -> (f32, Vec<(usize, usize, f32)>) {
    (
        neuron.bias(),
        neuron
            .connections()
            .iter()
            .map(|c| (c.from_id(), c.to_id(), c.weight()))
            .collect(),
    )
}

#[test]
fn primitive_network_keeps_its_temperature_threshold_and_updates_activation() {
    let mut neuron = Neuron::new(
        1,
        &crate::definitions::Definitions::builtin()
            .species()
            .neural(),
    );
    assert!(!neuron.process(&[20.0]));
    assert!(neuron.activation() < 0.0);
    assert!(!neuron.process(&[25.0]));
    assert_eq!(neuron.activation(), 0.0);
    assert!(neuron.process(&[30.0]));
    assert!(neuron.activation() > 0.0);
}

#[test]
fn inheritance_without_mutations_preserves_parameters_but_resets_activity() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut parent = Neuron::new(
        1,
        &crate::definitions::Definitions::builtin()
            .species()
            .neural(),
    );
    parent.evolve(3, 1.0, &mut rng);
    parent.process(&[40.0]);
    let before = parameters(&parent);
    let mut child = parent.clone();
    child.evolve(3, 0.0, &mut rng);
    assert_eq!(parameters(&child), before);
    assert_eq!(child.activation(), 0.0);
    child.evolve(3, 1.0, &mut rng);
    assert_ne!(parameters(&child), before);
    assert_eq!(parameters(&parent), before);
}

#[test]
fn growth_preserves_the_signal_and_shrinkage_removes_dangling_connections() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut neuron = Neuron::new(
        1,
        &crate::definitions::Definitions::builtin()
            .species()
            .neural(),
    );
    neuron.process(&[26.0]);
    let original = neuron.activation();
    neuron.evolve(5, 0.0, &mut rng);
    neuron.process(&[26.0]);
    assert_eq!(neuron.activation(), original);
    neuron.evolve(2, 0.0, &mut rng);
    assert_eq!(neuron.complexity(), 2);
    assert!(
        neuron
            .connections()
            .iter()
            .all(|c| c.from_id() < c.to_id() && c.to_id() <= 2)
    );
    neuron.process(&[26.0]);
    assert_eq!(neuron.activation(), original);
    neuron.evolve(1, 0.0, &mut rng);
    neuron.process(&[26.0]);
    assert_eq!(neuron.activation(), original);
}

#[test]
fn empty_sensory_input_does_not_panic() {
    assert!(
        !Neuron::new(
            1,
            &crate::definitions::Definitions::builtin()
                .species()
                .neural()
        )
        .process(&[])
    );
}

/// Propagation d'origine : un tampon d'activations accumulé connexion par connexion.
fn reference_activation(neuron: &Neuron, input: f32) -> f32 {
    let mut activations = vec![0.0; neuron.complexity() as usize + 1];
    activations[0] = input;
    for connection in neuron.connections() {
        activations[connection.to_id()] += activations[connection.from_id()] * connection.weight();
    }
    (activations[neuron.complexity() as usize] + neuron.bias()).tanh()
}

#[test]
fn chained_propagation_matches_the_accumulating_reference_bit_for_bit() {
    let mut rng = StdRng::seed_from_u64(7);
    let mut neuron = Neuron::new(
        1,
        &crate::definitions::Definitions::builtin()
            .species()
            .neural(),
    );
    for generation in 0..200 {
        neuron.evolve(1 + generation % 9, 0.7, &mut rng);
        for input in [-40.0, -1.5, 0.0, 19.9, 20.0, 31.5, 90.0] {
            let expected = reference_activation(&neuron, input);
            neuron.process(&[input]);
            assert_eq!(neuron.activation().to_bits(), expected.to_bits());
        }
    }
}
