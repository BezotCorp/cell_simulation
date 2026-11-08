use crate::definitions::Definitions;
use crate::ecosystem::{ClockAdvance, FIXED_STEP, PopulationHistory, Simulation, SimulationClock};
use crate::persistence::SessionView;
use serde::{Deserialize, Serialize};
use std::{path::Path, sync::Arc, time::Duration};

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Session {
    simulation: Simulation,
    view: SessionView,
    clock: SimulationClock,
    /// Historique d'observation, sauvegardé avec l'expérience.
    history: PopulationHistory,
}
impl Session {
    pub(crate) fn new(definitions: Arc<Definitions>, seed: u64) -> Result<Self, String> {
        let mut view = SessionView::default();
        view.focus(
            [0.0; 3],
            definitions.world().size() as f32 * definitions.appearance().habitat_spacing() * 1.55,
        );
        let simulation = Simulation::from_definitions(definitions, seed)?;
        let mut history = PopulationHistory::new();
        history.record(&simulation.summary());
        Ok(Self {
            simulation,
            view,
            clock: SimulationClock::new(),
            history,
        })
    }

    /// Fait avancer d'un pas et prend un échantillon d'historique lorsqu'il est dû.
    fn step_recorded(simulation: &mut Simulation, history: &mut PopulationHistory) {
        simulation.step(FIXED_STEP);
        if history.is_due(simulation.ticks() as f64 * FIXED_STEP.as_secs_f64()) {
            history.record(&simulation.summary());
        }
    }
    pub(crate) fn simulation(&self) -> &Simulation {
        &self.simulation
    }
    pub(crate) fn history(&self) -> &PopulationHistory {
        &self.history
    }

    fn elapsed_seconds(&self) -> f64 {
        self.simulation.ticks() as f64 * FIXED_STEP.as_secs_f64()
    }

    pub(crate) fn advance_steps(&mut self, steps: u64) {
        for _ in 0..steps {
            Self::step_recorded(&mut self.simulation, &mut self.history);
        }
        self.clear_stale_selection();
    }

    /// Temps simulé correspondant à `real` secondes d'horloge murale à la vitesse choisie ;
    /// nul en pause, car une pause n'accumule rien.
    pub(crate) fn simulated_time(&self, real: Duration) -> Duration {
        if self.view.paused() {
            Duration::ZERO
        } else {
            real.mul_f64(self.view.speed() as f64)
        }
    }

    /// Exécute au plus `max_steps` pas fixes pour couvrir `owed` de temps simulé et rend
    /// ce qui n'a pas pu être exécuté ; voir [`SimulationClock::advance`].
    pub(crate) fn advance(&mut self, owed: Duration, max_steps: u64) -> ClockAdvance {
        let advance = self.clock.advance(
            owed,
            !self.view.paused(),
            max_steps,
            |_| Self::step_recorded(&mut self.simulation, &mut self.history),
        );
        self.clear_stale_selection();
        advance
    }
    pub(crate) fn save(&self, path: impl AsRef<Path>) -> Result<(), String> {
        crate::persistence::binary::save(self, path.as_ref())
    }
    pub(crate) fn load(
        path: impl AsRef<Path>,
        definitions: Arc<Definitions>,
    ) -> Result<Self, String> {
        crate::persistence::binary::load(path.as_ref(), definitions)
    }
    /// Cohérence de la vue et de l'horloge, indépendante de l'index des habitats.
    fn verify_state(&self) -> Result<(), String> {
        self.view.validate()?;
        self.clock.validate()
    }

    /// L'habitat sélectionné doit exister ; l'index d'identités doit être à jour.
    fn verify_selection(&self) -> Result<(), String> {
        if self
            .view
            .habitat()
            .is_some_and(|id| self.simulation.grid().habitat_position(id).is_none())
        {
            return Err("Habitat sélectionné absent du monde.".into());
        }
        Ok(())
    }

    /// Contrôle l'ensemble de la session sans la modifier (utilisé avant toute écriture).
    pub(crate) fn check(&self, definitions: &Definitions) -> Result<(), String> {
        self.verify_state()?;
        self.verify_selection()?;
        self.history.verify(self.elapsed_seconds())?;
        self.simulation.verify(definitions).map(drop)
    }

    /// Valide une session restaurée et lui associe `definitions` ; en cas d'erreur
    /// elle doit être abandonnée par l'appelant. Les index dérivés (identités d'habitats,
    /// filiation) n'existent qu'après la liaison : la sélection est donc contrôlée ensuite.
    pub(crate) fn validate(&mut self, definitions: Arc<Definitions>) -> Result<(), String> {
        self.verify_state()?;
        self.history.verify(self.elapsed_seconds())?;
        self.simulation.bind_definitions(definitions)?;
        self.verify_selection()?;
        self.clear_stale_selection();
        Ok(())
    }

    /// Applique une intervention de l'expérimentateur à l'expérience.
    pub(crate) fn intervene(&mut self, intervention: &crate::ecosystem::Intervention) -> Result<String, String> {
        let message = self.simulation.intervene(intervention)?;
        self.clear_stale_selection();
        Ok(message)
    }

    /// Adopte de nouvelles définitions sans perdre les cellules vivantes ; en cas d'erreur
    /// l'expérience reste intacte.
    pub(crate) fn redefine(&mut self, definitions: Arc<Definitions>) -> Result<(), String> {
        definitions.validate()?;
        self.simulation.redefine(definitions)
    }

    pub(crate) fn set_paused(&mut self, paused: bool) {
        self.view.set_paused(paused);
    }

    pub(crate) fn set_speed(&mut self, speed: f32) {
        self.view.set_speed(speed);
    }

    pub(crate) fn view(&self) -> &SessionView {
        &self.view
    }
    pub(crate) fn toggle_pause(&mut self) {
        self.view.toggle_pause();
    }
    pub(crate) fn step_once(&mut self) {
        self.view.pause();
        self.advance_steps(1);
    }
    pub(crate) fn slower(&mut self) {
        self.view.slower();
    }
    pub(crate) fn faster(&mut self) {
        self.view.faster();
    }
    pub(crate) fn cycle_view(&mut self) {
        self.view.cycle_mode();
    }
    pub(crate) fn focus(&mut self, target: [f32; 3], distance: f32) {
        self.view.focus(target, distance);
    }
    pub(crate) fn orbit(&mut self, dx: f32, dy: f32) {
        self.view.orbit(dx, dy);
    }
    pub(crate) fn pan(&mut self, dx: f32, dy: f32) {
        self.view.pan(dx, dy);
    }
    pub(crate) fn zoom(&mut self, amount: f32) {
        self.view.zoom(amount);
    }
    pub(crate) fn select_organism(&mut self, id: uuid::Uuid) {
        if self.simulation.is_alive(id) {
            self.view.select_organism(id);
        } else {
            self.view.clear_selection();
        }
    }
    pub(crate) fn select_habitat(&mut self, id: uuid::Uuid) {
        if self.simulation.grid().habitat_position(id).is_some() {
            self.view.select_habitat(id);
        } else {
            self.view.clear_selection();
        }
    }
    pub(crate) fn clear_selection(&mut self) {
        self.view.clear_selection();
    }
    fn clear_stale_selection(&mut self) {
        if self
            .view
            .selected()
            .is_some_and(|id| !self.simulation.is_alive(id))
        {
            self.view.clear_selection();
        }
    }
}
