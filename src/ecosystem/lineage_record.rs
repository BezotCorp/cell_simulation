use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct LineageRecord {
    id: Uuid,
    parent: Option<Uuid>,
    born_at_tick: u64,
    died_at_tick: Option<u64>,
}

impl LineageRecord {
    pub(crate) fn new(id: Uuid, parent: Option<Uuid>, born_at_tick: u64) -> Self {
        Self {
            id,
            parent,
            born_at_tick,
            died_at_tick: None,
        }
    }

    pub(crate) fn id(&self) -> Uuid {
        self.id
    }

    pub(crate) fn parent(&self) -> Option<Uuid> {
        self.parent
    }

    pub(crate) fn born_at_tick(&self) -> u64 {
        self.born_at_tick
    }

    pub(crate) fn died_at_tick(&self) -> Option<u64> {
        self.died_at_tick
    }

    pub(crate) fn record_death(&mut self, tick: u64) {
        self.died_at_tick = Some(tick);
    }
}
