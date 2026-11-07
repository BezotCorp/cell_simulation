use std::time::Duration;

use crate::ecosystem::{ClockAdvance, FIXED_STEP};

/// Accumule le temps actif sans perdre la fraction restante entre deux images.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct SimulationClock {
    pending: Duration,
}

impl SimulationClock {
    pub(crate) fn new() -> Self {
        Self {
            pending: Duration::ZERO,
        }
    }

    fn span(steps: u64) -> Duration {
        Duration::from_nanos(FIXED_STEP.as_nanos() as u64 * steps)
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.pending >= FIXED_STEP {
            Err("Accumulateur temporel invalide.".into())
        } else {
            Ok(())
        }
    }

    /// Exécute au plus `max_steps` pas fixes. Le temps correspondant aux pas non exécutés
    /// est rendu à l'appelant au lieu d'être perdu ; seule la fraction inférieure à un pas
    /// reste dans l'accumulateur, qui respecte donc toujours son invariant persistant.
    pub(crate) fn advance(
        &mut self,
        elapsed: Duration,
        running: bool,
        max_steps: u64,
        mut step: impl FnMut(Duration),
    ) -> ClockAdvance {
        if !running {
            // Une pause n'accumule rien et ne rattrape rien à la reprise.
            return ClockAdvance::new(0, Duration::ZERO);
        }
        let available = self.pending + elapsed;
        let due = (available.as_nanos() / FIXED_STEP.as_nanos()) as u64;
        let steps = due.min(max_steps);
        for _ in 0..steps {
            step(FIXED_STEP);
        }
        let unused = Self::span(due - steps);
        self.pending = available - Self::span(due);
        ClockAdvance::new(steps, unused)
    }
}
