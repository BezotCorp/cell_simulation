use crate::genetics::{Interpretation, Signal, Signals};

/// Condition compilée : les noms de motifs du catalogue sont devenus des indices.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Condition {
    Count {
        motif: usize,
        at_least: u32,
        at_most: Option<u32>,
    },
    /// `combo` indexe la liste des combinaisons de motifs qui doivent partager une séquence.
    SameSequence { combo: usize },
    Ordered { first: usize, then: usize },
    Repeated { motif: usize, copies: u32 },
    /// Le motif commence avant le bit `bits` du génome.
    Within { motif: usize, bits: u32 },
    Signal {
        signal: Signal,
        at_least: f32,
        at_most: Option<f32>,
    },
}

impl Condition {
    /// Vrai pour les conditions qui dépendent du milieu et non du génome seul.
    pub(crate) fn is_dynamic(&self) -> bool {
        matches!(self, Self::Signal { .. })
    }

    /// Évalue une condition qui ne dépend que du génome ; les conditions de milieu valent vrai ici.
    pub(crate) fn holds_in_genome(&self, genome: &Interpretation) -> bool {
        match *self {
            Self::Count {
                motif,
                at_least,
                at_most,
            } => {
                let count = genome.count(motif);
                count >= at_least && at_most.is_none_or(|most| count <= most)
            }
            Self::SameSequence { combo } => genome.combo(combo) > 0,
            Self::Ordered { first, then } => matches!(
                (genome.first(first), genome.last(then)),
                (Some(first), Some(then)) if first < then
            ),
            Self::Repeated { motif, copies } => genome.repeated(motif) >= copies,
            Self::Within { motif, bits } => genome.first(motif).is_some_and(|start| start < bits),
            Self::Signal { .. } => true,
        }
    }

    /// Évalue une condition de milieu ; les conditions du génome valent vrai ici.
    pub(crate) fn holds_in_environment(&self, signals: &Signals) -> bool {
        match *self {
            Self::Signal {
                signal,
                at_least,
                at_most,
            } => {
                let value = signals.get(signal);
                value >= at_least && at_most.is_none_or(|most| value <= most)
            }
            _ => true,
        }
    }
}
