use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Liaisons durables entre organismes : une cellule fille peut rester attachée à sa mère. Les
/// liaisons sont ordonnées (rejeu exact) ; les index d'appartenance et de degré en sont dérivés.
#[derive(Clone, Default)]
pub(crate) struct Bonds {
    pairs: Vec<(Uuid, Uuid)>,
    linked: HashSet<(Uuid, Uuid)>,
    degree: HashMap<Uuid, u8>,
}

fn ordered(a: Uuid, b: Uuid) -> (Uuid, Uuid) {
    if a <= b { (a, b) } else { (b, a) }
}

impl Bonds {
    pub(crate) fn from_pairs(pairs: Vec<(Uuid, Uuid)>) -> Self {
        let mut bonds = Self::default();
        for (a, b) in pairs {
            bonds.link(a, b);
        }
        bonds
    }

    pub(crate) fn pairs(&self) -> &[(Uuid, Uuid)] {
        &self.pairs
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }

    pub(crate) fn len(&self) -> usize {
        self.pairs.len()
    }

    pub(crate) fn degree(&self, id: Uuid) -> u8 {
        self.degree.get(&id).copied().unwrap_or(0)
    }

    pub(crate) fn are_linked(&self, a: Uuid, b: Uuid) -> bool {
        !self.linked.is_empty() && self.linked.contains(&ordered(a, b))
    }

    /// Relie deux organismes distincts ; une liaison déjà présente n'est pas dupliquée.
    pub(crate) fn link(&mut self, a: Uuid, b: Uuid) {
        if a == b || !self.linked.insert(ordered(a, b)) {
            return;
        }
        self.pairs.push(ordered(a, b));
        for id in [a, b] {
            *self.degree.entry(id).or_insert(0) += 1;
        }
    }

    /// Ne garde que les liaisons que `keep` approuve, dans l'ordre existant.
    pub(crate) fn retain(&mut self, mut keep: impl FnMut(Uuid, Uuid) -> bool) {
        let before = self.pairs.len();
        self.pairs.retain(|(a, b)| keep(*a, *b));
        if self.pairs.len() != before {
            self.linked = self.pairs.iter().copied().collect();
            self.degree.clear();
            for (a, b) in &self.pairs {
                for id in [a, b] {
                    *self.degree.entry(*id).or_insert(0) += 1;
                }
            }
        }
    }
}
