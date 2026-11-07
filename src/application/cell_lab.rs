use crate::persistence::Session;
use crate::{
    application::{LaunchOptions, Runtime, run_status::RunStatus},
    interface::Hud,
    world_scene::{OrbitCamera, WorldScene},
};

use fyrox::{
    core::{algebra::Vector2, pool::Handle, reflect::prelude::*, visitor::prelude::*},
    event::Event,
    gui::{UserInterface, message::UiMessage},
    plugin::{
        Plugin, PluginContext, PluginRegistrationContext,
        error::{GameError, GameResult},
    },
};

#[derive(Visit, Reflect, Default)]
#[reflect(non_cloneable)]
pub(crate) struct CellLab {
    #[visit(skip)]
    #[reflect(hidden)]
    options: LaunchOptions,
    #[visit(skip)]
    #[reflect(hidden)]
    pending: Option<Session>,
    #[visit(skip)]
    #[reflect(hidden)]
    state: Option<Runtime>,
    #[visit(skip)]
    #[reflect(hidden)]
    status: std::sync::Arc<std::sync::Mutex<RunStatus>>,
}

impl std::fmt::Debug for CellLab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CellLab")
            .field("running", &self.state.is_some())
            .finish()
    }
}

impl CellLab {
    pub(crate) fn new(
        options: LaunchOptions,
        session: Session,
        status: std::sync::Arc<std::sync::Mutex<RunStatus>>,
    ) -> Self {
        Self {
            options,
            pending: Some(session),
            state: None,
            status,
        }
    }

    pub(crate) fn session(&self) -> Option<&Session> {
        self.state.as_ref().map(|s| s.session())
    }
}

impl Plugin for CellLab {
    fn register(&self, context: PluginRegistrationContext) -> GameResult {
        context
            .serialization_context
            .script_constructors
            .add::<OrbitCamera>("Cell Lab / caméra orbitale");
        Ok(())
    }

    fn init(&mut self, _scene_path: Option<&str>, context: PluginContext) -> GameResult {
        let session = self
            .pending
            .take()
            .ok_or_else(|| GameError::str("Session initiale absente"))?;

        let world = WorldScene::build(session.simulation(), context.scenes, (1148, 784));

        context
            .user_interfaces
            .add(UserInterface::new(Vector2::new(1500.0, 950.0)));

        let (frame, scale) = match context.graphics_context {
            fyrox::engine::GraphicsContext::Initialized(graphics) => {
                // Les points lumineux thermiques n'ont pas besoin de cartes d'ombre cubiques,
                // coûteuses ; les ombres de la lumière principale sont conservées.
                let mut quality = graphics.renderer.get_quality_settings();
                quality.point_shadows_enabled = false;
                if let Err(error) = graphics.renderer.set_quality_settings(&quality) {
                    eprintln!("Cell Lab : réglages de rendu refusés : {error}");
                }
                let size = graphics.window.inner_size();
                (
                    Vector2::new(size.width as f32, size.height as f32),
                    (graphics.window.scale_factor() as f32).clamp(1.0, 2.5),
                )
            }
            _ => (Vector2::new(1500.0, 950.0), 1.0),
        };

        let mut hud = Hud::new(
            context.user_interfaces.first_mut(),
            &session,
            world.target().clone(),
            scale,
        );

        hud.resize(context.user_interfaces.first(), frame.x, frame.y);
        self.state = Some(Runtime::new(
            session,
            world,
            hud,
            frame,
            self.options.load_path().is_none(),
        ));

        *context.lag = 0.0;

        self.status
            .lock()
            .map_err(|e| GameError::str(e.to_string()))?
            .mark_initialized();

        Ok(())
    }

    fn on_deinit(&mut self, _context: PluginContext) -> GameResult {
        if let Some(state) = &self.state {
            if let Err(error) = state
                .session()
                .save(self.options.save_path().with_extension("autosave.bin"))
            {
                if let Ok(mut status) = self.status.lock() {
                    status.record_error(error.clone());
                }
                return Err(GameError::str(error));
            }
        }

        Ok(())
    }

    fn update(&mut self, context: &mut PluginContext) -> GameResult {
        self.state
            .as_mut()
            .ok_or_else(|| GameError::str("Application non initialisée"))?
            .update(context);
        Ok(())
    }
    fn on_ui_message(
        &mut self,
        context: &mut PluginContext,
        message: &UiMessage,
        _ui: Handle<UserInterface>,
    ) -> GameResult {
        if let Some(state) = &mut self.state {
            if let Some(action) = state.message(message) {
                state.handle_action(action, &self.options, context);
            }
        }
        Ok(())
    }
    fn on_os_event(&mut self, event: &Event<()>, mut context: PluginContext) -> GameResult {
        if let Some(state) = &mut self.state {
            if let Some(action) = state.handle_os_event(event, &mut context) {
                state.handle_action(action, &self.options, &mut context);
            }
        }
        Ok(())
    }
}
