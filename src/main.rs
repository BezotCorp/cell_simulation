mod app;
mod cells;
mod environment;
mod simulation_engine;
mod ui;

use crate::app::App;
use crate::simulation_engine::Simulation;
use crate::ui::SimulationRenderer;
use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([600.0, 700.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Simulation de cellules évolutives",
        options,
        Box::new(|_cc| Ok(Box::new(App::new()))),
    )
}
