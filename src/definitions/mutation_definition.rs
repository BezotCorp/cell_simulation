use serde::{Deserialize, Serialize};

/// Taux de mutation à la division. Ils s'appliquent à une copie fidèle du génome ; ils sont
/// multipliés par l'exposition du milieu et divisés par `1 + repair_gain × réparation`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MutationDefinition {
    /// Probabilité d'erreur de copie de chaque bit.
    pub(crate) bit_flip: f64,
    /// Événements structurels attendus par division (poisson).
    pub(crate) duplication: f64,
    pub(crate) deletion: f64,
    pub(crate) insertion: f64,
    pub(crate) inversion: f64,
    pub(crate) translocation: f64,
    /// Longueur maximale, en séquences, d'un fragment dupliqué, supprimé, inséré ou déplacé.
    pub(crate) run_max: usize,
    /// Bornes de la longueur du génome, en séquences.
    pub(crate) min_length: usize,
    pub(crate) max_length: usize,
    /// Efficacité de la réparation sur tous les taux.
    pub(crate) repair_gain: f64,
}

impl MutationDefinition {
    pub(crate) fn validate(&self) -> Result<(), String> {
        let rates = [
            self.bit_flip,
            self.duplication,
            self.deletion,
            self.insertion,
            self.inversion,
            self.translocation,
            self.repair_gain,
        ];
        if rates.iter().any(|rate| !rate.is_finite() || *rate < 0.0)
            || self.bit_flip > 0.5
            || self.run_max == 0
            || self.min_length == 0
            || self.min_length > self.max_length
            || self.max_length > 4096
        {
            return Err("Taux de mutation incohérents.".into());
        }
        Ok(())
    }
}
