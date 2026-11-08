use crate::definitions::MotifDefinition;
use crate::genetics::{Motif, Orientation};

fn motif(bits: &str, orientation: Orientation) -> Motif {
    let definition: MotifDefinition =
        ron::from_str(&format!(r#"(bits: "{bits}", orientation: {orientation:?})"#)).unwrap();
    let (pattern, length) = definition.pattern().unwrap();
    Motif::new(pattern, length, orientation)
}

fn positions(motif: &Motif, byte: u8) -> Vec<u8> {
    (0..=(8 - motif.length())).filter(|start| motif.matches_at(byte, *start)).collect()
}

#[test]
fn a_window_is_read_from_the_high_bit_and_the_reverse_orientation_reads_it_backwards() {
    let forward = motif("1101", Orientation::Forward);
    // 1101 0000 : le motif commence au premier bit.
    assert_eq!(positions(&forward, 0b1101_0000), vec![0]);
    assert_eq!(positions(&forward, 0b0000_1101), vec![4]);
    let reverse = motif("1101", Orientation::Reverse);
    // 1011 dans le sens d'écriture est 1101 lu à l'envers.
    assert_eq!(positions(&reverse, 0b1011_0000), vec![0]);
    assert!(positions(&reverse, 0b1101_0000).is_empty());
    let both = motif("1101", Orientation::Both);
    assert_eq!(positions(&both, 0b1011_1101), vec![0, 4]);
}

#[test]
fn overlapping_occurrences_are_all_reported_and_motifs_up_to_eight_bits_work() {
    let run = motif("11", Orientation::Forward);
    assert_eq!(positions(&run, 0b0111_1000), vec![1, 2, 3]);
    let whole = motif("10110010", Orientation::Forward);
    assert_eq!(positions(&whole, 0b1011_0010), vec![0]);
    assert!(positions(&whole, 0b1011_0011).is_empty());
    let single = motif("1", Orientation::Forward);
    assert_eq!(positions(&single, 0b1000_0001), vec![0, 7]);
}

#[test]
fn invalid_motif_patterns_are_refused() {
    for bits in ["", "2", "102", "123456789", "1 0"] {
        let definition: MotifDefinition = ron::from_str(&format!(r#"(bits: "{bits}")"#)).unwrap();
        assert!(definition.pattern().is_err(), "{bits}");
    }
}
