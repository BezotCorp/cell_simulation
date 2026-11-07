use std::time::Duration;

/// Résultat d'un avancement de l'horloge : pas exécutés et temps réclamé mais non consommé.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ClockAdvance {
    steps: u64,
    unused: Duration,
}

impl ClockAdvance {
    pub(crate) fn new(steps: u64, unused: Duration) -> Self {
        Self { steps, unused }
    }

    pub(crate) fn steps(&self) -> u64 {
        self.steps
    }

    /// Multiple entier de `FIXED_STEP` que le plafond de pas a empêché d'exécuter.
    pub(crate) fn unused(&self) -> Duration {
        self.unused
    }
}
