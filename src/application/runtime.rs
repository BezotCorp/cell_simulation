use crate::definitions::Definitions;
use crate::ecosystem::FIXED_STEP;
use crate::world_scene::Picked;
use crate::{
    application::LaunchOptions,
    interface::{Action, Hud},
    persistence::Session,
    world_scene::WorldScene,
};
use fyrox::{
    core::algebra::Vector2,
    event::{ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent},
    gui::message::UiMessage,
    keyboard::{KeyCode, PhysicalKey},
    plugin::PluginContext,
};
use std::time::{Duration, Instant};

/// Durée réelle maximale prise en compte pour une image : une suspension de la fenêtre ou
/// un point d'arrêt ne déclenche pas un rattrapage massif à la reprise.
const MAX_FRAME: Duration = Duration::from_millis(250);
/// Part de chaque image que la simulation peut consommer ; le reste revient au rendu.
const STEP_BUDGET: Duration = Duration::from_millis(9);
/// Période de rafraîchissement des statistiques et de l'inspecteur.
const HUD_PERIOD: Duration = Duration::from_millis(125);
/// Fenêtre de mesure de la vitesse effective.
const PACE_WINDOW: Duration = Duration::from_millis(500);

/// Mesure du coût des pas biologiques, pour adapter le nombre de pas par image à la machine.
struct StepBudget {
    seconds_per_step: f64,
    window_steps: u64,
    window_time: Duration,
    effective_speed: f32,
}

impl StepBudget {
    fn new(speed: f32) -> Self {
        Self {
            seconds_per_step: 20e-6,
            window_steps: 0,
            window_time: Duration::ZERO,
            effective_speed: speed,
        }
    }

    /// Nombre de pas exécutables dans le budget (au moins un, pour toujours progresser).
    fn max_steps(&self) -> u64 {
        ((STEP_BUDGET.as_secs_f64() / self.seconds_per_step) as u64).max(1)
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

pub(crate) struct Runtime {
    session: Session,
    world: WorldScene,
    hud: Hud,
    frame: Vector2<f32>,
    pointer: Vector2<f32>,
    orbiting: bool,
    shift: bool,
    control: bool,
    last_update: Instant,
    since_hud: Duration,
    budget: StepBudget,
}
impl Runtime {
    pub(crate) fn new(session: Session, world: WorldScene, hud: Hud, frame: Vector2<f32>) -> Self {
        Self {
            budget: StepBudget::new(session.view().speed()),
            session,
            world,
            hud,
            frame,
            pointer: Vector2::zeros(),
            orbiting: false,
            shift: false,
            control: false,
            last_update: Instant::now(),
            since_hud: HUD_PERIOD,
        }
    }
    pub(crate) fn session(&self) -> &Session {
        &self.session
    }
    pub(crate) fn message(&mut self, message: &UiMessage) -> Option<Action> {
        self.hud.message(message)
    }
    pub(crate) fn update(&mut self, context: &mut PluginContext) {
        // Le temps réel écoulé pilote la simulation ; Fyrox ne doit pas, en plus, rejouer
        // les mises à jour en retard, ce qui aggraverait toute surcharge.
        *context.lag = 0.0;
        let now = Instant::now();
        let real = now.duration_since(self.last_update).min(MAX_FRAME);
        self.last_update = now;

        let started = Instant::now();
        let steps = self.session.update(real, self.budget.max_steps());
        self.budget.record(steps, started.elapsed(), real);

        let (_, area) = self.hud.viewport(context.user_interfaces.first());
        let scene = &mut context.scenes[self.world.scene()];
        if self.world.resize(scene, area) {
            self.hud
                .set_viewport(context.user_interfaces.first(), self.world.target().clone());
        }
        self.world
            .sync(self.session.simulation(), self.session.view(), scene);
        self.world.animate(real.as_secs_f32(), scene);

        self.since_hud += real;
        if self.since_hud >= HUD_PERIOD {
            self.since_hud = Duration::ZERO;
            self.hud.sync(
                context.user_interfaces.first_mut(),
                &self.session,
                self.budget.effective_speed,
            );
        }
    }

    /// Remplace l'expérience et sa scène. L'appelant a déjà construit et validé la nouvelle
    /// session : plus rien ne peut échouer ici, l'ancien état n'est jamais à moitié remplacé.
    fn install(&mut self, session: Session, context: &mut PluginContext) {
        let (_, area) = self.hud.viewport(context.user_interfaces.first());
        let world = WorldScene::build(
            session.simulation(),
            context.scenes,
            (area.x.max(1.0) as u32, area.y.max(1.0) as u32),
        );
        context.scenes.remove(self.world.scene());
        self.hud
            .set_viewport(context.user_interfaces.first(), world.target().clone());
        self.hud.reset(context.user_interfaces.first(), &session);
        self.budget = StepBudget::new(session.view().speed());
        self.last_update = Instant::now();
        self.since_hud = HUD_PERIOD;
        self.world = world;
        self.session = session;
        *context.lag = 0.0;
    }

    pub(crate) fn handle_action(
        &mut self,
        action: Action,
        options: &LaunchOptions,
        context: &mut PluginContext,
    ) {
        let result = self.perform(action, options, context);

        match result {
            Ok(Some(message)) => {
                self.hud.status(context.user_interfaces.first(), message);
            }
            Err(error) => {
                self.hud
                    .status(context.user_interfaces.first(), format!("Échec : {error}"));
                eprintln!("Cell Lab : {error}");
            }
            _ => {}
        }
    }

    fn perform(
        &mut self,
        action: Action,
        options: &LaunchOptions,
        context: &mut PluginContext,
    ) -> Result<Option<String>, String> {
        match action {
            Action::Pause => self.session.toggle_pause(),
            Action::Step => self.session.step_once(),
            Action::Slower => self.session.slower(),
            Action::Faster => self.session.faster(),
            Action::CycleView => self.session.cycle_view(),
            Action::Focus => {
                let position = self
                    .session
                    .view()
                    .selected()
                    .and_then(|id| self.world.selected_position(id));
                let definitions = self.session.simulation().definitions();
                let distance = if position.is_some() {
                    5.0
                } else {
                    definitions.world().size() as f32
                        * definitions.appearance().habitat_spacing()
                        * 1.55
                };
                self.session.focus(position.unwrap_or([0.0; 3]), distance);
            }
            Action::Save => {
                self.session.save(options.save_path())?;
                return Ok(Some(format!(
                    "Expérience sauvegardée · {}",
                    options.save_path().display()
                )));
            }
            Action::Load => {
                let loaded = Session::load(
                    options.save_path(),
                    self.session.simulation().definitions().clone(),
                )?;
                self.install(loaded, context);
                return Ok(Some(
                    "Expérience chargée · identités et état aléatoire restaurés".into(),
                ));
            }
            Action::Restart | Action::Reload => {
                // Tout ce qui peut échouer est fait avant de toucher à l'expérience courante.
                let seed = self
                    .hud
                    .seed_text()
                    .trim()
                    .parse::<u64>()
                    .map_err(|_| "La graine doit être un entier positif.".to_string())?;
                let definitions = if action == Action::Reload {
                    Definitions::load(options.definitions())?
                } else {
                    self.session.simulation().definitions().clone()
                };
                let session = Session::new(definitions, seed)?;
                // L'expérience précédente est conservée avant d'être remplacée.
                self.session
                    .save(options.save_path().with_extension("previous.bin"))?;
                self.install(session, context);
                return Ok(Some(
                    "Nouvelle expérience · la précédente est conservée dans previous.bin".into(),
                ));
            }
        }

        Ok(None)
    }

    /// Zone d'affichage du monde, en pixels de fenêtre (position, taille).
    fn world_area(&self, context: &PluginContext) -> (Vector2<f32>, Vector2<f32>) {
        self.hud.viewport(context.user_interfaces.first())
    }

    pub(crate) fn handle_os_event(
        &mut self,
        event: &Event<()>,
        context: &mut PluginContext,
    ) -> Option<Action> {
        let Event::WindowEvent { event, .. } = event else {
            return None;
        };

        let mut action = None;

        match event {
            WindowEvent::Resized(size) => {
                self.frame = Vector2::new(size.width.max(1) as f32, size.height.max(1) as f32);

                self.hud
                    .resize(context.user_interfaces.first(), self.frame.x, self.frame.y);
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.shift = modifiers.state().shift_key();
                self.control = modifiers.state().control_key();
            }
            WindowEvent::CursorMoved { position, .. } => {
                let position = Vector2::new(position.x as f32, position.y as f32);
                let delta = position - self.pointer;

                if self.orbiting {
                    if self.shift {
                        self.session.pan(delta.x, delta.y);
                    } else {
                        self.session.orbit(delta.x, delta.y);
                    }
                }

                self.pointer = position;
            }
            WindowEvent::MouseInput {
                state: button_state,
                button,
                ..
            } => {
                let (origin, size) = self.world_area(context);
                let local = self.pointer - origin;
                let over_world =
                    local.x >= 0.0 && local.y >= 0.0 && local.x < size.x && local.y < size.y;

                if *button == MouseButton::Right {
                    self.orbiting = *button_state == ElementState::Pressed && over_world;
                }

                if *button == MouseButton::Left
                    && *button_state == ElementState::Pressed
                    && over_world
                {
                    match self.world.pick(
                        &context.scenes[self.world.scene()],
                        self.session.simulation(),
                        local,
                    ) {
                        Some(Picked::Organism(id)) => self.session.select_organism(id),
                        Some(Picked::Habitat(id)) => self.session.select_habitat(id),
                        None => self.session.clear_selection(),
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let (origin, size) = self.world_area(context);
                let local = self.pointer - origin;

                if local.x >= 0.0 && local.y >= 0.0 && local.x < size.x && local.y < size.y {
                    let value = match delta {
                        MouseScrollDelta::LineDelta(_, y) => *y,
                        MouseScrollDelta::PixelDelta(position) => position.y as f32 / 30.0,
                    };

                    self.session.zoom(value);
                }
            }
            WindowEvent::KeyboardInput { event, .. }
                if event.state == ElementState::Pressed && !event.repeat =>
            {
                if let PhysicalKey::Code(key) = event.physical_key {
                    let editing_seed = self.hud.editing_seed();

                    action = match key {
                        KeyCode::Space if !editing_seed => Some(Action::Pause),
                        KeyCode::KeyN if !editing_seed => Some(Action::Step),
                        KeyCode::KeyF if !editing_seed => Some(Action::Focus),
                        KeyCode::KeyS if self.control => Some(Action::Save),
                        KeyCode::KeyO if self.control => Some(Action::Load),
                        _ => None,
                    };
                }
            }
            _ => {}
        }

        action
    }
}
