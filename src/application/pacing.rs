use crate::ecosystem::{ClockAdvance, FIXED_STEP};
use crate::persistence::Session;
use std::time::{Duration, Instant};

/// Durée réelle maximale prise en compte pour une image : une suspension de la fenêtre ou
/// un point d'arrêt ne déclenche pas un rattrapage massif à la reprise.
const MAX_FRAME: Duration = Duration::from_millis(250);
/// Part de chaque image que la simulation peut consommer ; le reste revient au rendu.
const STEP_BUDGET: Duration = Duration::from_millis(9);
/// Fenêtre de mesure de la vitesse effective.
const PACE_WINDOW: Duration = Duration::from_millis(500);
/// Le retard conservé ne dépasse jamais ce multiple de la demande d'une image.
const DEBT_FRAMES: u32 = 4;

/// Cadence d'exécution de l'expérience face à la machine.
///
/// Trois grandeurs, à ne pas confondre :
/// - la **vitesse demandée** (× 0,25 à × 64) vient de l'utilisateur ;
/// - le **retard** est le temps simulé demandé mais pas encore exécuté parce que le budget
///   de calcul d'une image était épuisé. Il est rendu à l'image suivante, borné à quatre
///   fois la demande d'une image : au-delà, la machine ralentit l'expérience au lieu de
///   s'endetter indéfiniment (ce qui ne perd aucun pas biologique : le temps simulé est
///   compté en pas, il n'est jamais comprimé ni sauté) ;
/// - la **vitesse effective** est ce qui a réellement été exécuté sur la dernière fenêtre
///   de mesure ; l'interface l'affiche dès qu'elle diffère de la vitesse demandée.
///
/// Rien de tout cela n'est sauvegardé : c'est un état d'exécution propre à la machine.
pub(crate) struct Pacing {
    last_update: Instant,
    seconds_per_step: f64,
    debt: Duration,
    window_steps: u64,
    window_time: Duration,
    effective_speed: f32,
}

impl Pacing {
    pub(crate) fn new(speed: f32) -> Self {
        Self {
            last_update: Instant::now(),
            seconds_per_step: 20e-6,
            debt: Duration::ZERO,
            window_steps: 0,
            window_time: Duration::ZERO,
            effective_speed: speed,
        }
    }

    pub(crate) fn effective_speed(&self) -> f32 {
        self.effective_speed
    }

    pub(crate) fn debt(&self) -> Duration {
        self.debt
    }

    /// Nombre de pas exécutables dans le budget (au moins un, pour toujours progresser).
    fn max_steps(&self) -> u64 {
        ((STEP_BUDGET.as_secs_f64() / self.seconds_per_step) as u64).max(1)
    }

    /// Avance la session du temps réel écoulé depuis l'appel précédent. Renvoie ce temps réel
    /// (plafonné), pour animer l'affichage avec la même mesure.
    pub(crate) fn advance(&mut self, session: &mut Session) -> Duration {
        let now = Instant::now();
        let real = now.duration_since(self.last_update).min(MAX_FRAME);
        self.last_update = now;
        let started = Instant::now();
        let advance = self.advance_capped(session, real, self.max_steps());
        self.record(advance.steps(), started.elapsed(), real);
        real
    }

    /// Cœur déterministe de [`Pacing::advance`] : `real` secondes de temps réel, au plus
    /// `max_steps` pas.
    pub(crate) fn advance_capped(
        &mut self,
        session: &mut Session,
        real: Duration,
        max_steps: u64,
    ) -> ClockAdvance {
        let requested = session.simulated_time(real);
        let advance = session.advance(self.debt + requested, max_steps);
        self.debt = advance.unused().min(requested * DEBT_FRAMES);
        advance
    }

    fn record(&mut self, steps: u64, compute: Duration, real: Duration) {
        if steps > 0 {
            let measured = compute.as_secs_f64() / steps as f64;
            self.seconds_per_step += (measured - self.seconds_per_step) * 0.2;
        }
        self.window_steps += steps;
        self.window_time += real;
        if self.window_time >= PACE_WINDOW {
            self.effective_speed = (self.window_steps as f64 * FIXED_STEP.as_secs_f64()
                / self.window_time.as_secs_f64()) as f32;
            self.window_steps = 0;
            self.window_time = Duration::ZERO;
        }
    }
}
