use crate::genetics::{
    Chunk, Genetics, Levels, Product, Sensor, Signals,
    chunk_analysis::NONE,
    genetics::MAX_MOTIFS,
};
use std::sync::Arc;

/// Ce que dit un génome des règles du jeu, indépendamment du milieu : occurrences et positions des
/// motifs, combinaisons, séquences répétées, puis la part des contributions toujours vraie (`fixed`)
/// et la liste de celles qui attendent un signal du milieu. Calculée une fois par génome distinct.
#[derive(Clone, Debug)]
pub(crate) struct Interpretation {
    rules: u64,
    counts: [u32; MAX_MOTIFS],
    first: [u32; MAX_MOTIFS],
    last: [u32; MAX_MOTIFS],
    combos: Vec<u32>,
    repeated: [u32; MAX_MOTIFS],
    fixed: [f32; Product::COUNT],
    dynamic: Vec<usize>,
    sensor: Sensor,
}

impl Interpretation {
    pub(crate) fn of(genetics: &Genetics, chunks: &[Arc<Chunk>]) -> Self {
        let mut interpretation = Self {
            rules: genetics.id(),
            counts: [0; MAX_MOTIFS],
            first: [NONE; MAX_MOTIFS],
            last: [NONE; MAX_MOTIFS],
            combos: vec![0; genetics.combos().len()],
            repeated: [0; MAX_MOTIFS],
            fixed: [0.0; Product::COUNT],
            dynamic: Vec::new(),
            sensor: Sensor::default(),
        };
        let mut histogram = genetics.needs_histogram().then(|| [0u32; 256]);
        let mut offset = 0u32;
        for chunk in chunks {
            let analysis = chunk.analysis(genetics);
            for motif in 0..genetics.motif_count() {
                interpretation.counts[motif] += analysis.count(motif);
                if analysis.first(motif) != NONE {
                    let (first, last) = (offset + analysis.first(motif), offset + analysis.last(motif));
                    interpretation.first[motif] = interpretation.first[motif].min(first);
                    if interpretation.last[motif] == NONE || last > interpretation.last[motif] {
                        interpretation.last[motif] = last;
                    }
                }
            }
            for (total, part) in interpretation.combos.iter_mut().zip(analysis.combos()) {
                *total += part;
            }
            if let (Some(total), Some(part)) = (histogram.as_mut(), analysis.histogram()) {
                for (sum, value) in total.iter_mut().zip(part) {
                    *sum += value;
                }
            }
            offset += chunk.len() as u32 * 8;
        }
        if let Some(histogram) = histogram {
            for byte in 0..256 {
                let mask = genetics.profile(byte as u8).mask();
                for motif in 0..genetics.motif_count() {
                    if mask & (1 << motif) != 0 {
                        interpretation.repeated[motif] = interpretation.repeated[motif].max(histogram[byte]);
                    }
                }
            }
        }
        interpretation.sensor = genetics
            .sensors()
            .iter()
            .filter(|(motif, _)| interpretation.counts[*motif] > 0)
            .min_by_key(|(motif, _)| interpretation.first[*motif])
            .map_or(Sensor::default(), |(_, sensor)| *sensor);
        for (index, contribution) in genetics.contributions().iter().enumerate() {
            let in_genome = contribution
                .conditions()
                .iter()
                .all(|condition| condition.holds_in_genome(&interpretation));
            if !in_genome {
                continue;
            }
            if contribution.is_dynamic() {
                interpretation.dynamic.push(index);
            } else {
                interpretation.fixed[contribution.product().index()] +=
                    contribution.amount() * interpretation.dose(contribution.per_copy());
            }
        }
        interpretation
    }

    fn dose(&self, per_copy: Option<(usize, u32)>) -> f32 {
        per_copy.map_or(1.0, |(motif, up_to)| self.counts[motif].min(up_to) as f32)
    }

    pub(crate) fn rules(&self) -> u64 {
        self.rules
    }

    pub(crate) fn count(&self, motif: usize) -> u32 {
        self.counts[motif]
    }

    pub(crate) fn first(&self, motif: usize) -> Option<u32> {
        Some(self.first[motif]).filter(|position| *position != NONE)
    }

    pub(crate) fn last(&self, motif: usize) -> Option<u32> {
        Some(self.last[motif]).filter(|position| *position != NONE)
    }

    pub(crate) fn combo(&self, combo: usize) -> u32 {
        self.combos[combo]
    }

    pub(crate) fn repeated(&self, motif: usize) -> u32 {
        self.repeated[motif]
    }

    pub(crate) fn sensor(&self) -> Sensor {
        self.sensor
    }

    /// Quantités de produits dans le milieu décrit par `signals` : la part fixe, plus les
    /// contributions dont les conditions de milieu sont réunies.
    pub(crate) fn express(&self, genetics: &Genetics, signals: &Signals) -> Levels {
        let mut values = self.fixed;
        for index in &self.dynamic {
            let contribution = &genetics.contributions()[*index];
            if contribution
                .conditions()
                .iter()
                .all(|condition| condition.holds_in_environment(signals))
            {
                values[contribution.product().index()] +=
                    contribution.amount() * self.dose(contribution.per_copy());
            }
        }
        Levels::new(values.map(|value| value.clamp(0.0, 1.0)))
    }
}
