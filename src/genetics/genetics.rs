use crate::definitions::{
    ConditionDefinition, CostDefinition, GeneticsDefinition, MutationDefinition, NeuralDefinition,
};
use crate::genetics::{
    ByteProfile, ChunkPool, Condition, Contribution, Genome, Levels, Motif, Sensor,
    Signals,
};
use std::collections::BTreeMap;

/// Nombre maximal de motifs distincts d'un catalogue (un masque de 32 bits les désigne).
pub(crate) const MAX_MOTIFS: usize = 32;

/// Règles génétiques compilées depuis le catalogue : motifs, table de profils des 256 séquences
/// possibles, contributions, réseau neuronal, taux de mutation et coûts. Elle est partagée par
/// toutes les cellules d'une simulation et porte le registre des fragments de génome.
#[derive(Debug)]
pub(crate) struct Genetics {
    id: u64,
    names: Vec<String>,
    motifs: Vec<Motif>,
    profiles: Vec<ByteProfile>,
    combos: Vec<u32>,
    needs_histogram: bool,
    contributions: Vec<Contribution>,
    sensors: Vec<(usize, Sensor)>,
    neural: NeuralDefinition,
    mutation: MutationDefinition,
    costs: CostDefinition,
    pool: ChunkPool,
}

impl Genetics {
    /// Compile la définition : résout les noms de motifs, précalcule la table des séquences et
    /// refuse toute incohérence (motif inconnu, quantité non finie, trop de motifs…).
    pub(crate) fn compile(definition: &GeneticsDefinition) -> Result<Self, String> {
        if definition.motifs.is_empty() || definition.motifs.len() > MAX_MOTIFS {
            return Err(format!("Un catalogue compte de 1 à {MAX_MOTIFS} motifs."));
        }
        definition.mutation.validate()?;
        definition.costs.validate()?;
        let names: Vec<String> = definition.motifs.keys().cloned().collect();
        let index: BTreeMap<&str, usize> = names.iter().enumerate().map(|(i, n)| (n.as_str(), i)).collect();
        let lookup = |name: &str| {
            index
                .get(name)
                .copied()
                .ok_or_else(|| format!("Motif inconnu dans une règle : « {name} »."))
        };
        let mut motifs = Vec::new();
        for definition in definition.motifs.values() {
            let (pattern, length) = definition.pattern()?;
            motifs.push(Motif::new(pattern, length, definition.orientation()));
        }

        let mut combos: Vec<u32> = Vec::new();
        let mut contributions = Vec::new();
        let mut needs_histogram = false;
        for capability in &definition.capabilities {
            for contribution in &capability.contributions {
                if !contribution.amount.is_finite() || contribution.amount.abs() > 4.0 {
                    return Err("Contribution biologique hors des bornes.".into());
                }
                let mut conditions = Vec::new();
                for condition in &contribution.when {
                    conditions.push(match condition {
                        ConditionDefinition::Count { motif, at_least, at_most } => Condition::Count {
                            motif: lookup(motif)?,
                            at_least: *at_least,
                            at_most: *at_most,
                        },
                        ConditionDefinition::SameSequence { motifs } => {
                            let mut mask = 0u32;
                            for motif in motifs {
                                mask |= 1 << lookup(motif)?;
                            }
                            let combo = combos.iter().position(|known| *known == mask).unwrap_or_else(|| {
                                combos.push(mask);
                                combos.len() - 1
                            });
                            Condition::SameSequence { combo }
                        }
                        ConditionDefinition::Ordered { first, then } => Condition::Ordered {
                            first: lookup(first)?,
                            then: lookup(then)?,
                        },
                        ConditionDefinition::Repeated { motif, copies } => {
                            needs_histogram = true;
                            Condition::Repeated {
                                motif: lookup(motif)?,
                                copies: *copies,
                            }
                        }
                        ConditionDefinition::Within { motif, sequences } => Condition::Within {
                            motif: lookup(motif)?,
                            bits: sequences.saturating_mul(8),
                        },
                        ConditionDefinition::Signal { signal, at_least, at_most } => {
                            if !at_least.is_finite() || at_most.is_some_and(|most| !most.is_finite()) {
                                return Err("Seuil de signal non fini.".into());
                            }
                            Condition::Signal {
                                signal: *signal,
                                at_least: *at_least,
                                at_most: *at_most,
                            }
                        }
                    });
                }
                let per_copy = match &contribution.per_copy {
                    Some(dose) => Some((lookup(&dose.motif)?, dose.up_to)),
                    None => None,
                };
                contributions.push(Contribution::new(capability.product, contribution.amount, conditions, per_copy));
            }
        }
        let mut sensors = Vec::new();
        for (motif, sensor) in &definition.neural.sensors {
            sensors.push((lookup(motif)?, *sensor));
        }
        if !definition.neural.gain.is_finite() || !definition.neural.bias.is_finite() {
            return Err("Réseau neuronal du catalogue invalide.".into());
        }
        let profiles = (0..=255u8).map(|byte| ByteProfile::of(byte, &motifs)).collect();
        // L'identité des règles : un fragment interprété sous d'autres règles est réinterprété.
        let id = bincode_next::serde::encode_to_vec(definition, bincode_next::config::standard())
            .map_err(|e| e.to_string())?
            .iter()
            .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
                (hash ^ u64::from(*byte)).wrapping_mul(0x0100_0000_01b3)
            });
        Ok(Self {
            id,
            names,
            motifs,
            profiles,
            combos,
            needs_histogram,
            contributions,
            sensors,
            neural: definition.neural.clone(),
            mutation: definition.mutation.clone(),
            costs: definition.costs.clone(),
            pool: ChunkPool::default(),
        })
    }

    pub(crate) fn id(&self) -> u64 {
        self.id
    }

    pub(crate) fn motif_count(&self) -> usize {
        self.motifs.len()
    }

    /// Séquence de 8 bits qui porte le motif `name` avec le moins d'autres motifs possible.
    pub(crate) fn carrier(&self, name: &str) -> Option<u8> {
        let index = (0..self.motif_count()).find(|index| self.motif_name(*index) == name)?;
        (0..=255u8)
            .filter(|byte| self.profile(*byte).mask() & (1 << index) != 0)
            .min_by_key(|byte| self.profile(*byte).mask().count_ones())
    }

    pub(crate) fn motif_name(&self, motif: usize) -> &str {
        &self.names[motif]
    }

    pub(crate) fn profile(&self, byte: u8) -> &ByteProfile {
        &self.profiles[usize::from(byte)]
    }

    pub(crate) fn combos(&self) -> &[u32] {
        &self.combos
    }

    pub(crate) fn needs_histogram(&self) -> bool {
        self.needs_histogram
    }

    pub(crate) fn contributions(&self) -> &[Contribution] {
        &self.contributions
    }

    pub(crate) fn sensors(&self) -> &[(usize, Sensor)] {
        &self.sensors
    }

    pub(crate) fn neural(&self) -> &NeuralDefinition {
        &self.neural
    }

    pub(crate) fn mutation(&self) -> &MutationDefinition {
        &self.mutation
    }

    pub(crate) fn costs(&self) -> &CostDefinition {
        &self.costs
    }

    pub(crate) fn pool(&self) -> &ChunkPool {
        &self.pool
    }

    /// Produits exprimés par `genome` dans le milieu `signals`.
    pub(crate) fn express(&self, genome: &Genome, signals: &Signals) -> Levels {
        genome.interpretation(self).express(self, signals)
    }

    /// Énergie dépensée pour copier `genome` à la division.
    pub(crate) fn replication_cost(&self, genome: &Genome, repair: f32) -> f32 {
        self.costs.replication_per_sequence
            * genome.len() as f32
            * (1.0 + self.costs.fidelity_surcharge * repair)
    }

    /// Entretien par seconde et par unité de métabolisme au-delà du génome de référence.
    pub(crate) fn upkeep(&self, genome: &Genome, reference_length: usize) -> f32 {
        self.costs.upkeep_per_sequence * (genome.len() as f32 - reference_length as f32)
    }

    /// Calcule l'interprétation de plusieurs génomes à la fois, en répartissant les génomes encore
    /// non interprétés sur `threads` fils d'exécution. Chaque interprétation est pure : le
    /// résultat ne dépend ni de l'ordre ni du nombre de fils. Renvoie le nombre de génomes calculés.
    pub(crate) fn interpret_all(&self, genomes: &[&Genome], threads: usize) -> usize {
        let pending: Vec<&&Genome> = genomes.iter().filter(|genome| !genome.is_interpreted()).collect();
        let threads = threads.clamp(1, pending.len().max(1));
        if threads == 1 {
            pending.iter().for_each(|genome| drop(genome.interpretation(self)));
        } else {
            std::thread::scope(|scope| {
                for part in pending.chunks(pending.len().div_ceil(threads)) {
                    scope.spawn(move || part.iter().for_each(|genome| drop(genome.interpretation(self))));
                }
            });
        }
        pending.len()
    }
}
