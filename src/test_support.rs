//! Aides de test : fabriquer des catalogues dont l'ancêtre diffère de celui du jeu en ajoutant ou
//! retirant des séquences qui portent un motif donné.

use crate::definitions::Definitions;
use std::sync::Arc;

const CATALOG: &str = include_str!("../data/definitions.ron");

pub(crate) fn motif_byte(definitions: &Definitions, motif: &str) -> u8 {
    definitions
        .species()
        .genetics()
        .carrier(motif)
        .unwrap_or_else(|| panic!("motif inconnu : {motif}"))
}

/// Catalogue du jeu dont l'ancêtre reçoit une séquence par motif de `add` et perd, pour chaque motif
/// de `remove`, une séquence qui le porte.
pub(crate) fn catalog(add: &[&str], remove: &[&str]) -> Arc<Definitions> {
    let base: Definitions = ron::from_str(CATALOG).expect("catalogue du jeu");
    let genetics = base.species().genetics().clone();
    let mut sequences = base.species().ancestor_genome().bytes();
    for motif in remove {
        let index = (0..genetics.motif_count())
            .find(|index| genetics.motif_name(*index) == *motif)
            .unwrap_or_else(|| panic!("motif inconnu : {motif}"));
        let position = sequences
            .iter()
            .position(|byte| genetics.profile(*byte).mask() & (1 << index) != 0)
            .unwrap_or_else(|| panic!("l'ancêtre ne porte pas {motif}"));
        sequences.remove(position);
    }
    for motif in add {
        sequences.push(motif_byte(&base, motif));
    }
    let list: Vec<String> = sequences.iter().map(|byte| format!("\"{byte:08b}\"")).collect();
    let start = CATALOG.find("sequences: [").expect("ancêtre") + "sequences: [".len();
    let end = start + CATALOG[start..].find(']').expect("fin de l'ancêtre");
    let text = format!("{}{}{}", &CATALOG[..start], list.join(", "), &CATALOG[end..]);
    let definitions: Definitions = ron::from_str(&text).expect("catalogue modifié");
    definitions.validate().expect("catalogue modifié valide");
    Arc::new(definitions)
}

/// Catalogue du jeu modifié par `edit` (cellules initiales, capacités, composants, monde…).
pub(crate) fn edited(edit: impl FnOnce(&mut Definitions)) -> Arc<Definitions> {
    let mut definitions: Definitions = ron::from_str(CATALOG).expect("catalogue du jeu");
    edit(&mut definitions);
    definitions.validate().expect("catalogue modifié valide");
    Arc::new(definitions)
}
