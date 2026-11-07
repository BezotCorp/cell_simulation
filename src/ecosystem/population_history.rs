use crate::ecosystem::{FIXED_STEP, LineageRecord, PopulationSample, PopulationSummary};
use serde::{Deserialize, Serialize};

/// Nombre de points conservés avant fusion : l'historique complet reste toujours affichable.
const CAPACITY: usize = 480;

/// Historique d'observation d'une expérience : effectifs, naissances, décès et énergie moyenne
/// échantillonnés au rythme du temps simulé, donc indépendamment de l'affichage et de la
/// vitesse. Il accompagne la sauvegarde sans appartenir à l'état biologique.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct PopulationHistory {
    samples: Vec<PopulationSample>,
    interval: f64,
    next: f64,
    /// Compteur de modifications, utile pour ne redessiner que ce qui a changé.
    #[serde(skip)]
    revision: u64,
}

impl PopulationHistory {
    pub(crate) fn new() -> Self {
        Self {
            samples: Vec::new(),
            interval: 1.0,
            next: 0.0,
            revision: 0,
        }
    }

    /// Historique d'une sauvegarde qui n'en contient pas : effectifs, naissances et décès
    /// sont déduits de la filiation à intervalles réguliers ; l'énergie reste inconnue.
    pub(crate) fn from_lineage(records: &[LineageRecord], ticks: u64) -> Self {
        let mut history = Self::new();
        let mut births: Vec<u64> = records
            .iter()
            .filter(|record| record.parent().is_some())
            .map(|record| record.born_at_tick())
            .collect();
        let mut deaths: Vec<u64> = records
            .iter()
            .filter_map(|record| record.died_at_tick())
            .collect();
        let founders = (records.len() - births.len()) as f32;
        births.sort_unstable();
        deaths.sort_unstable();
        let points = ((CAPACITY / 2) as u64).min(ticks.max(1));
        for index in 0..=points {
            let tick = ticks * index / points;
            let born = births.partition_point(|&t| t <= tick) as f32;
            let dead = deaths.partition_point(|&t| t <= tick) as f32;
            history.samples.push(PopulationSample::new(
                tick as f64 * FIXED_STEP.as_secs_f64(),
                founders + born - dead,
                born,
                dead,
                f32::NAN,
            ));
        }
        history.next = ticks as f64 * FIXED_STEP.as_secs_f64() + history.interval;
        history.revision = 1;
        history
    }

    pub(crate) fn samples(&self) -> &[PopulationSample] {
        &self.samples
    }

    pub(crate) fn revision(&self) -> u64 {
        self.revision
    }

    /// Vrai si un échantillon doit être pris à ce temps simulé (secondes).
    pub(crate) fn is_due(&self, elapsed_seconds: f64) -> bool {
        elapsed_seconds >= self.next
    }

    pub(crate) fn record(&mut self, summary: &PopulationSummary) {
        let time = summary.elapsed_seconds();
        self.samples.push(PopulationSample::new(
            time,
            summary.cells() as f32,
            summary.births() as f32,
            summary.deaths() as f32,
            summary.average_energy(),
        ));
        self.next = time + self.interval;
        self.revision += 1;
        if self.samples.len() >= CAPACITY {
            // Garde un point sur deux (et toujours le dernier) : la résolution diminue,
            // la durée couverte reste entière.
            let (mut index, last) = (0, self.samples.len());
            self.samples.retain(|_| {
                index += 1;
                index % 2 == 1 || index == last
            });
            self.interval *= 2.0;
        }
    }

    /// Contrôle sans rien modifier qu'un historique restauré est cohérent avec l'expérience.
    pub(crate) fn verify(&self, elapsed_seconds: f64) -> Result<(), String> {
        if !self.interval.is_finite()
            || self.interval <= 0.0
            || !self.next.is_finite()
            || self.samples.len() > 2 * CAPACITY
            || self.samples.iter().any(|sample| !sample.is_valid())
            || self
                .samples
                .windows(2)
                .any(|pair| pair[0].time() > pair[1].time())
            || self
                .samples
                .last()
                .is_some_and(|sample| sample.time() > elapsed_seconds + 1e-6)
        {
            return Err("Historique de population invalide.".into());
        }
        Ok(())
    }
}
