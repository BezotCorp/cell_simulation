use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Liaisons durables entre organismes : une cellule fille peut rester attachée à sa mère. Les
/// liaisons sont ordonnées (rejeu exact) ; les index d'appartenance et de degré en sont dérivés.
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(from = "Vec<(Uuid, Uuid)>", into = "Vec<(Uuid, Uuid)>")]
pub(crate) struct Bonds {
    pairs: Vec<(Uuid, Uuid)>,
    linked: HashSet<(Uuid, Uuid)>,
    degree: HashMap<Uuid, u8>,
    partners: HashMap<Uuid, Vec<Uuid>>,
}

impl From<Vec<(Uuid, Uuid)>> for Bonds {
    fn from(pairs: Vec<(Uuid, Uuid)>) -> Self {
        Self::from_pairs(pairs)
    }
}

impl From<Bonds> for Vec<(Uuid, Uuid)> {
    fn from(bonds: Bonds) -> Self {
        bonds.pairs
    }
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
        self.partners.entry(a).or_default().push(b);
        self.partners.entry(b).or_default().push(a);
    }

    /// Organismes liés à `id`, dans l'ordre de création des liaisons.
    pub(crate) fn partners(&self, id: Uuid) -> &[Uuid] {
        self.partners.get(&id).map_or(&[], Vec::as_slice)
    }

    /// Ne garde que les liaisons que `keep` approuve, dans l'ordre existant.
    pub(crate) fn retain(&mut self, mut keep: impl FnMut(Uuid, Uuid) -> bool) {
        let before = self.pairs.len();
        self.pairs.retain(|(a, b)| keep(*a, *b));
        if self.pairs.len() != before {
            self.linked = self.pairs.iter().copied().collect();
            self.degree.clear();
            self.partners.clear();
            for (a, b) in &self.pairs {
                for id in [a, b] {
                    *self.degree.entry(*id).or_insert(0) += 1;
                }
                self.partners.entry(*a).or_default().push(*b);
                self.partners.entry(*b).or_default().push(*a);
            }
        }
    }
}
