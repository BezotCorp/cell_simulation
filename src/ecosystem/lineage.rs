use crate::ecosystem::LineageRecord;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::{Uuid, Version};

/// Historique complet des filiations. Aucun enregistrement n'est jamais retiré : les index
/// et compteurs ci-dessous sont dérivés et reconstruits à la restauration.
#[derive(Clone, Default, Serialize, Deserialize)]
pub(crate) struct Lineage {
    records: Vec<LineageRecord>,
    #[serde(skip)]
    index: HashMap<Uuid, usize>,
    #[serde(skip)]
    deaths: u64,
}

impl Lineage {
    pub(crate) fn records(&self) -> &[LineageRecord] {
        &self.records
    }

    pub(crate) fn get(&self, id: Uuid) -> Option<&LineageRecord> {
        self.index.get(&id).map(|&index| &self.records[index])
    }

    pub(crate) fn is_alive(&self, id: Uuid) -> bool {
        self.get(id)
            .is_some_and(|record| record.died_at_tick().is_none())
    }

    pub(crate) fn deaths(&self) -> u64 {
        self.deaths
    }

    pub(crate) fn record_birth(&mut self, id: Uuid, parent: Option<Uuid>, tick: u64) {
        self.index.insert(id, self.records.len());
        self.records.push(LineageRecord::new(id, parent, tick));
    }

    pub(crate) fn record_death(&mut self, id: Uuid, tick: u64) {
        if let Some(&index) = self.index.get(&id) {
            let record = &mut self.records[index];
            if record.died_at_tick().is_none() {
                record.record_death(tick);
                self.deaths += 1;
            }
        }
    }

    /// Contrôle l'historique sans le modifier et renvoie l'index identité → position.
    /// `reserved` contient les identités déjà utilisées par le monde.
    pub(crate) fn verify(
        &self,
        ticks: u64,
        births: u64,
        reserved: &HashSet<Uuid>,
    ) -> Result<HashMap<Uuid, usize>, String> {
        let mut index = HashMap::with_capacity(self.records.len());

        for (position, record) in self.records.iter().enumerate() {
            if record.id().get_version() != Some(Version::SortRand)
                || reserved.contains(&record.id())
                || index.insert(record.id(), position).is_some()
                || record.born_at_tick() > ticks
                || record
                    .died_at_tick()
                    .is_some_and(|tick| tick < record.born_at_tick() || tick > ticks)
            {
                return Err("Filiation invalide.".into());
            }
        }

        let founders = self
            .records
            .iter()
            .filter(|record| record.parent().is_none())
            .count() as u64;
        if self.records.len() as u64 - founders != births || founders > 1 {
            return Err("Compteurs de filiation incohérents.".into());
        }

        for record in &self.records {
            if let Some(parent_id) = record.parent() {
                let parent = index
                    .get(&parent_id)
                    .map(|&position| &self.records[position])
                    .ok_or("Parent absent de la filiation")?;

                if parent.born_at_tick() >= record.born_at_tick()
                    || parent
                        .died_at_tick()
                        .is_some_and(|tick| tick < record.born_at_tick())
                {
                    return Err("Chronologie de filiation invalide.".into());
                }
            } else if record.born_at_tick() != 0 {
                return Err("Fondatrice née après le début de l’expérience.".into());
            }
        }

        Ok(index)
    }

    /// Installe l'index produit par [`Lineage::verify`] et recompte les décès.
    pub(crate) fn adopt_index(&mut self, index: HashMap<Uuid, usize>) {
        self.deaths = self
            .records
            .iter()
            .filter(|record| record.died_at_tick().is_some())
            .count() as u64;
        self.index = index;
    }
}
