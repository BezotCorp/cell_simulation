use crate::genetics::{Genetics, genetics::MAX_MOTIFS};

/// Marque « absent » dans les positions.
pub(crate) const NONE: u32 = u32::MAX;

/// Résultat de l'interprétation d'un fragment de génome : pour chaque motif, nombre d'occurrences
/// et première et dernière positions (en bits depuis le début du fragment), plus les séquences qui
/// réunissent les combinaisons de motifs voulues et la fréquence de chaque séquence complète. Il ne
/// dépend que des octets du fragment et des règles : tous les génomes qui partagent le fragment
/// partagent aussi ce résultat.
#[derive(Clone, Debug)]
pub(crate) struct ChunkAnalysis {
    rules: u64,
    counts: [u32; MAX_MOTIFS],
    first: [u32; MAX_MOTIFS],
    last: [u32; MAX_MOTIFS],
    combos: Vec<u32>,
    histogram: Option<Box<[u32; 256]>>,
}

impl ChunkAnalysis {
    pub(crate) fn of(genetics: &Genetics, bytes: &[u8]) -> Self {
        let mut analysis = Self {
            rules: genetics.id(),
            counts: [0; MAX_MOTIFS],
            first: [NONE; MAX_MOTIFS],
            last: [NONE; MAX_MOTIFS],
            combos: vec![0; genetics.combos().len()],
            histogram: genetics.needs_histogram().then(|| Box::new([0; 256])),
        };
        for (index, byte) in bytes.iter().enumerate() {
            let profile = genetics.profile(*byte);
            let mut present = profile.mask();
            while present != 0 {
                let motif = present.trailing_zeros() as usize;
                present &= present - 1;
                analysis.counts[motif] += u32::from(profile.count(motif));
                let base = index as u32 * 8;
                let first = base + u32::from(profile.first(motif).expect("motif présent"));
                let last = base + u32::from(profile.last(motif).expect("motif présent"));
                analysis.first[motif] = analysis.first[motif].min(first);
                analysis.last[motif] = if analysis.last[motif] == NONE { last } else { analysis.last[motif].max(last) };
            }
            for (combo, mask) in genetics.combos().iter().enumerate() {
                analysis.combos[combo] += u32::from(profile.mask() & mask == *mask);
            }
            if let Some(histogram) = &mut analysis.histogram {
                histogram[usize::from(*byte)] += 1;
            }
        }
        analysis
    }

    pub(crate) fn rules(&self) -> u64 {
        self.rules
    }

    pub(crate) fn count(&self, motif: usize) -> u32 {
        self.counts[motif]
    }

    pub(crate) fn first(&self, motif: usize) -> u32 {
        self.first[motif]
    }

    pub(crate) fn last(&self, motif: usize) -> u32 {
        self.last[motif]
    }

    pub(crate) fn combos(&self) -> &[u32] {
        &self.combos
    }

    pub(crate) fn histogram(&self) -> Option<&[u32; 256]> {
        self.histogram.as_deref()
    }
}
