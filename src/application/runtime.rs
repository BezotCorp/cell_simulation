use crate::world_scene::ComponentLibrary;
use crate::definitions::Definitions;
use crate::world_scene::{OrbitCamera, Picked};
use crate::{
    application::{LaunchOptions, Pacing},
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
use std::time::Duration;

/// Période de rafraîchissement des statistiques et de l'inspecteur.
const HUD_PERIOD: Duration = Duration::from_millis(125);

pub(crate) struct Runtime {
    session: Session,
    world: WorldScene,
    hud: Hud,
    frame: Vector2<f32>,
    pointer: Vector2<f32>,
    orbiting: bool,
    shift: bool,
    control: bool,
    since_hud: Duration,
    pacing: Pacing,
    /// Vrai tant que la caméra d'une expérience neuve n'a pas été cadrée sur le biotope : la
    /// zone d'affichage n'est connue qu'après la première mise en page.
    pending_framing: bool,
    /// Bibliothèque de composants anatomiques, chargée une fois et partagée par toutes les scènes.
    library: ComponentLibrary,
}
impl Runtime {
    pub(crate) fn new(
        session: Session,
        world: WorldScene,
        hud: Hud,
        frame: Vector2<f32>,
        fresh: bool,
        library: ComponentLibrary,
    ) -> Self {
        Self {
            library,
            pacing: Pacing::new(session.view().speed()),
            pending_framing: fresh,
            session,
            world,
            hud,
            frame,
            pointer: Vector2::zeros(),
            orbiting: false,
            shift: false,
            control: false,
            since_hud: HUD_PERIOD,
        }
    }
    pub(crate) fn session(&self) -> &Session {
        &self.session
    }
    pub(crate) fn message(&mut self, message: &UiMessage) -> Option<Action> {
        self.hud.message(message)
    }
    /// Cible et distance de caméra qui cadrent tout le biotope dans la zone d'affichage courante.
    fn framing(&self, area: Vector2<f32>) -> ([f32; 3], f32) {
        let definitions = self.session.simulation().definitions();
        let extent = definitions.world().size() as f32 * definitions.appearance().habitat_spacing();
        let camera = self.session.view().camera();
        OrbitCamera::framing(extent, area.x / area.y.max(1.0), camera.yaw(), camera.pitch())
    }

    pub(crate) fn update(&mut self, context: &mut PluginContext) {
        // Le temps réel écoulé pilote la simulation ; Fyrox ne doit pas, en plus, rejouer
        // les mises à jour en retard, ce qui aggraverait toute surcharge.
        *context.lag = 0.0;
        let real = self.pacing.advance(&mut self.session);

        let (_, area) = self.hud.viewport(context.user_interfaces.first());
        if self.pending_framing && area.x >= 16.0 && area.y >= 16.0 {
            self.pending_framing = false;
            let (target, distance) = self.framing(area);
            self.session.focus(target, distance);
        }
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
            self.hud
                .sync(context.user_interfaces.first_mut(), &self.session, &self.pacing);
        }
    }

    /// Remplace l'expérience et sa scène. L'appelant a déjà construit et validé la nouvelle
    /// session : plus rien ne peut échouer ici, l'ancien état n'est jamais à moitié remplacé.
    fn install(&mut self, session: Session, fresh: bool, context: &mut PluginContext) {
        let (_, area) = self.hud.viewport(context.user_interfaces.first());
        let world = WorldScene::build(
            session.simulation(),
            context.scenes,
            (area.x.max(1.0) as u32, area.y.max(1.0) as u32),
            self.library.clone(),
        );
        context.scenes.remove(self.world.scene());
        self.hud
            .set_viewport(context.user_interfaces.first(), world.target().clone());
        self.hud.reset(context.user_interfaces.first(), &session);
        self.pacing = Pacing::new(session.view().speed());
        self.pending_framing = fresh;
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
                let (_, area) = self.hud.viewport(context.user_interfaces.first());
                let (target, distance) = match position {
                    Some(position) => (position, 5.0),
                    None => self.framing(area),
                };
                self.session.focus(target, distance);
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
                self.install(loaded, false, context);
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
                self.install(session, true, context);
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
