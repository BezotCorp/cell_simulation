use serde::{Deserialize, Serialize};

/// Ressources que consomme le matériel génétique. Plus un génome est long, plus sa copie et son
/// entretien coûtent ; une copie plus fidèle (réparation) coûte davantage.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CostDefinition {
    /// Énergie dépensée par séquence copiée à la division.
    pub(crate) replication_per_sequence: f32,
    /// Majoration de la réplication par unité de réparation.
    pub(crate) fidelity_surcharge: f32,
    /// Entretien par seconde et par séquence au-delà de celles de l'ancêtre, en part du métabolisme.
    pub(crate) upkeep_per_sequence: f32,
    /// Énergie que coûte un transfert de fragment à un voisin lié.
    pub(crate) transfer: f32,
}

impl CostDefinition {
    pub(crate) fn validate(&self) -> Result<(), String> {
        let values = [
            self.replication_per_sequence,
            self.fidelity_surcharge,
            self.upkeep_per_sequence,
            self.transfer,
        ];
        if values.iter().any(|value| !value.is_finite() || *value < 0.0) {
            return Err("Coûts génétiques incohérents.".into());
        }
        Ok(())
    }
}
