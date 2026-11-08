use crate::genetics::{Motif, genetics::MAX_MOTIFS};

/// Motifs que contient une séquence de 8 bits, calculés une fois pour les 256 valeurs possibles :
/// interpréter un génome revient alors à consulter une table, sans chercher de motifs.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ByteProfile {
    /// Bit `m` à 1 si le motif `m` apparaît au moins une fois.
    mask: u32,
    counts: [u8; MAX_MOTIFS],
    /// Premier et dernier départ (en bits depuis le poids fort) de chaque motif, 255 si absent.
    first: [u8; MAX_MOTIFS],
    last: [u8; MAX_MOTIFS],
}

impl ByteProfile {
    pub(crate) fn of(byte: u8, motifs: &[Motif]) -> Self {
        let mut profile = Self {
            mask: 0,
            counts: [0; MAX_MOTIFS],
            first: [255; MAX_MOTIFS],
            last: [255; MAX_MOTIFS],
        };
        for (index, motif) in motifs.iter().enumerate() {
            for start in 0..=(8 - motif.length()) {
                if motif.matches_at(byte, start) {
                    profile.mask |= 1 << index;
                    profile.counts[index] += 1;
                    if profile.first[index] == 255 {
                        profile.first[index] = start;
                    }
                    profile.last[index] = start;
                }
            }
        }
        profile
    }

    pub(crate) fn mask(&self) -> u32 {
        self.mask
    }

    pub(crate) fn count(&self, motif: usize) -> u8 {
        self.counts[motif]
    }

    pub(crate) fn first(&self, motif: usize) -> Option<u8> {
        Some(self.first[motif]).filter(|start| *start != 255)
    }

    pub(crate) fn last(&self, motif: usize) -> Option<u8> {
        Some(self.last[motif]).filter(|start| *start != 255)
    }
}
