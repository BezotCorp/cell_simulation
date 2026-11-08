use crate::application::{CellLab, LaunchOptions};
use crate::definitions::Definitions;
use crate::persistence::Session;
use fyrox::{
    dpi::{LogicalSize, Size},
    engine::{GraphicsContextParams, executor::Executor},
    event_loop::EventLoop,
    window::WindowAttributes,
};

pub(crate) fn run(options: LaunchOptions) -> Result<(), String> {
    if options.help() {
        println!(
            "Cell Lab — Fyrox 3D\n\n  --definitions FICHIER.ron\n  --seed ENTIER\n  --load FICHIER.bin\n  --save FICHIER.bin\n  --headless --steps N   expérience reproductible sans fenêtre\n  --with-panel           ouvre aussi la fenêtre d'administration (laboratoire)\n  --panel                ouvre seulement la fenêtre d'administration\n  --admin-port PORT      port local du lien (47821 par défaut)\n\nDans la fenêtre : espace pause, N un pas, F centrer, Ctrl+S sauvegarder, Ctrl+O charger."
        );
        return Ok(());
    }
    if options.panel() {
        return crate::panel::run(&options);
    }
    let definitions = Definitions::load(options.definitions())?;
    let session = if let Some(path) = options.load_path() {
        Session::load(path, definitions.clone())?
    } else {
        Session::new(
            definitions.clone(),
            options.seed().unwrap_or(definitions.default_seed()),
        )?
    };
    if options.headless() {
        let mut session = session;
        session.advance_steps(options.steps());
        session.save(options.save_path())?;
        let summary = session.simulation().summary();
        println!(
            "session={} tick={} cells={} births={} deaths={} energy={:.6} save={}",
            session.simulation().id(),
            session.simulation().ticks(),
            summary.cells(),
            summary.births(),
            summary.deaths(),
            summary.average_energy(),
            options.save_path().display()
        );
        return Ok(());
    }
    let event_loop = EventLoop::new().map_err(|e| e.to_string())?;
    let attributes = WindowAttributes::default()
        .with_title(definitions.appearance().title())
        .with_inner_size(Size::Logical(LogicalSize::new(1500.0, 950.0)))
        .with_min_inner_size(Size::Logical(LogicalSize::new(1200.0, 720.0)));
    let mut executor = Executor::from_params(
        Some(event_loop),
        GraphicsContextParams {
            window_attributes: attributes,
            vsync: true,
            msaa_sample_count: Some(4),
            graphics_server_constructor: Default::default(),
            named_objects: cfg!(debug_assertions),
        },
    );
    executor.set_desired_update_rate(1.0 / crate::ecosystem::FIXED_STEP.as_secs_f32());
    // Fyrox must never enlarge the biological timestep under rendering load.
    executor.set_throttle_threshold(f32::MAX);
    let status = std::sync::Arc::new(std::sync::Mutex::new(
        crate::application::run_status::RunStatus::new(),
    ));
    executor.add_plugin(CellLab::new(options.clone(), session, status.clone()));
    executor.run();
    let status = status.lock().map_err(|error| error.to_string())?;
    if !status.initialized() {
        return Err("L’application Fyrox n’a pas pu être initialisée.".into());
    }
    if let Some(error) = status.error() {
        return Err(error.to_owned());
    }
    Ok(())
}
