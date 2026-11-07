use crate::ecosystem::{FIXED_STEP, LineageRecord, PopulationSummary};

/// Nombre de points conservés avant fusion : l'historique complet reste affichable.
const CAPACITY: usize = 480;

/// Instantané des grandeurs suivies. `energy` vaut NaN quand elle n'est pas connue.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Sample {
    time: f32,
    cells: f32,
    births: f32,
    deaths: f32,
    energy: f32,
}

impl Sample {
    pub(crate) fn time(&self) -> f32 {
        self.time
    }
    pub(crate) fn cells(&self) -> f32 {
        self.cells
    }
    pub(crate) fn births(&self) -> f32 {
        self.births
    }
    pub(crate) fn deaths(&self) -> f32 {
        self.deaths
    }
    pub(crate) fn energy(&self) -> f32 {
        self.energy
    }
}

/// Historique d'observation des populations. C'est une vue de l'expérience : il n'est pas
/// sauvegardé et se reconstruit, pour les effectifs, à partir de la filiation.
pub(crate) struct Telemetry {
    samples: Vec<Sample>,
    interval: f64,
    next: f64,
    revision: u64,
}

impl Telemetry {
    pub(crate) fn new() -> Self {
        Self {
            samples: Vec::new(),
            interval: 1.0,
            next: 0.0,
            revision: 0,
        }
    }

    /// Historique d'une expérience restaurée : effectifs, naissances et décès sont déduits de
    /// la filiation à intervalles réguliers ; l'énergie, non conservée, reste inconnue.
    pub(crate) fn from_lineage(records: &[LineageRecord], ticks: u64) -> Self {
        let mut telemetry = Self::new();
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
            telemetry.samples.push(Sample {
                time: (tick as f64 * FIXED_STEP.as_secs_f64()) as f32,
                cells: founders + born - dead,
                births: born,
                deaths: dead,
                energy: f32::NAN,
            });
        }
        telemetry.next = ticks as f64 * FIXED_STEP.as_secs_f64() + telemetry.interval;
        telemetry.revision = 1;
        telemetry
    }

    pub(crate) fn samples(&self) -> &[Sample] {
        &self.samples
    }

    /// Change à chaque point ajouté ou fusionné ; permet de ne redessiner que si nécessaire.
    pub(crate) fn revision(&self) -> u64 {
        self.revision
    }

    pub(crate) fn record(&mut self, summary: &PopulationSummary) {
        let time = summary.elapsed_seconds();
        if time < self.next {
            return;
        }
        self.samples.push(Sample {
            time: time as f32,
            cells: summary.cells() as f32,
            births: summary.births() as f32,
            deaths: summary.deaths() as f32,
            energy: summary.average_energy(),
        });
        self.next = time + self.interval;
        self.revision += 1;
        if self.samples.len() >= CAPACITY {
            // Garde un point sur deux : la résolution diminue, la durée couverte reste entière.
            let (mut index, last) = (0, self.samples.len());
            self.samples.retain(|_| {
                index += 1;
                index % 2 == 1 || index == last
            });
            self.interval *= 2.0;
        }
    }
}
