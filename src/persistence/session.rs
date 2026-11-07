use crate::definitions::Definitions;
use crate::ecosystem::Simulation;
use crate::ecosystem::SimulationClock;
use crate::persistence::SessionView;
use serde::{Deserialize, Serialize};
use std::{path::Path, sync::Arc, time::Duration};

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Session {
    simulation: Simulation,
    view: SessionView,
    clock: SimulationClock,
    /// Temps simulé réclamé mais non exécuté faute de budget de calcul. Dérivé du matériel,
    /// il n'appartient pas à l'expérience et n'est pas sauvegardé.
    #[serde(skip)]
    backlog: Duration,
}
impl Session {
    pub(crate) fn new(definitions: Arc<Definitions>, seed: u64) -> Result<Self, String> {
        let mut view = SessionView::default();
        view.focus(
            [0.0; 3],
            definitions.world().size() as f32 * definitions.appearance().habitat_spacing() * 1.55,
        );
        Ok(Self {
            simulation: Simulation::from_definitions(definitions, seed)?,
            view,
            clock: SimulationClock::new(),
            backlog: Duration::ZERO,
        })
    }
    pub(crate) fn simulation(&self) -> &Simulation {
        &self.simulation
    }
    pub(crate) fn advance_steps(&mut self, steps: u64) {
        self.simulation.advance_steps(steps);
        self.clear_stale_selection();
    }
    /// Avance l'expérience de `elapsed` (temps réel) à la vitesse choisie, sans dépasser
    /// `max_steps` pas fixes. Le temps simulé non exécuté est reporté dans le retard, borné
    /// à quatre fois la demande courante pour qu'une machine lente ralentisse l'expérience
    /// au lieu de s'endetter indéfiniment. Renvoie le nombre de pas exécutés.
    pub(crate) fn update(&mut self, elapsed: Duration, max_steps: u64) -> u64 {
        if self.view.paused() {
            self.backlog = Duration::ZERO;
            return 0;
        }
        let requested = elapsed.mul_f64(self.view.speed() as f64);
        let advance = self
            .clock
            .advance(requested + self.backlog, true, max_steps, |dt| {
                self.simulation.step(dt)
            });
        self.backlog = advance.unused.min(requested * 4);
        self.clear_stale_selection();
        advance.steps
    }

    pub(crate) fn backlog(&self) -> Duration {
        self.backlog
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
    fn verify_view(&self) -> Result<(), String> {
        self.view.validate()?;
        self.clock.validate()?;
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
        self.verify_view()?;
        self.simulation.verify(definitions).map(drop)
    }

    /// Valide une session restaurée et lui associe `definitions` ; en cas d'erreur
    /// elle reste inchangée et doit être abandonnée par l'appelant.
    pub(crate) fn validate(&mut self, definitions: Arc<Definitions>) -> Result<(), String> {
        self.verify_view()?;
        self.simulation.bind_definitions(definitions)?;
        self.clear_stale_selection();
        Ok(())
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
