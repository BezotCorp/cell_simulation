use eframe::egui;

use crate::simulation_engine::Simulation;
use crate::ui::SimulationRenderer;

const GRID_SIZE: usize = 10;
const CELL_SIZE: f32 = 40.0;

pub(crate) struct App {
    simulation: Simulation,
    renderer: SimulationRenderer,
    paused: bool,
}

impl App {
    pub(crate) fn new() -> Self {
        Self {
            simulation: Simulation::new(GRID_SIZE),
            renderer: SimulationRenderer::new(CELL_SIZE),
            paused: false,
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            // Contrôles de la simulation
            ui.horizontal(|ui| {
                if ui
                    .button(if self.paused { "▶ Play" } else { "⏸ Pause" })
                    .clicked()
                {
                    self.paused = !self.paused;
                }
            });

            // Affiche la simulation
            let stats = self.simulation.get_stats();
            self.renderer.render(
                ui,
                &self.simulation.grid(),
                &self.simulation.cells(),
                &stats,
            );

            // Met à jour la simulation si non pausée
            if !self.paused {
                self.simulation.update();
            }
        });

        // Force le rafraîchissement continu
        ui.ctx().request_repaint();
    }
}
