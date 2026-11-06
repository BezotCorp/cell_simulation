use crate::cells::Cell;
use crate::environment::{Entity, Grid, Terrain};
use crate::simulation_engine::SimulationStats;
use eframe::egui::{self, Align2, Color32, Pos2, Rect, Stroke, Vec2};

pub struct SimulationRenderer {
    cell_size: f32,
}

impl SimulationRenderer {
    pub(crate) fn new(cell_size: f32) -> Self {
        Self { cell_size }
    }

    pub(crate) fn cell_size(&self) -> f32 {
        self.cell_size
    }

    pub fn render(&self, ui: &mut egui::Ui, grid: &Grid, cells: &[Cell], stats: &SimulationStats) {
        // Panel des statistiques générales
        ui.vertical(|ui| {
            ui.heading("Statistiques de la simulation");
            ui.label(format!("Nombre de cellules : {}", stats.cell_count()));
            ui.label(format!("Génération : {}", stats.generation()));
            ui.label(format!(
                "Niveau d'eau : {:.1}%",
                stats.water_level() * 100.0
            ));
            ui.label(format!("Énergie moyenne : {:.1}", stats.average_energy()));

            // Barre de niveau d'eau
            let water_rect = ui.allocate_space(Vec2::new(200.0, 20.0)).1;
            ui.painter().rect_filled(
                Rect::from_min_size(
                    water_rect.min,
                    Vec2::new(
                        water_rect.width() * stats.water_level(),
                        water_rect.height(),
                    ),
                ),
                0.0,
                Color32::from_rgb(0, 0, 255),
            );
        });

        ui.add_space(10.0);

        // Affichage de la grille
        let (response, painter) = ui.allocate_painter(
            Vec2::new(
                grid.size() as f32 * self.cell_size,
                grid.size() as f32 * self.cell_size,
            ),
            egui::Sense::hover(),
        );

        let rect = response.rect;

        // Dessine chaque case
        for y in 0..grid.size() {
            for x in 0..grid.size() {
                let cell_rect = Rect::from_min_size(
                    Pos2::new(
                        rect.min.x + x as f32 * self.cell_size,
                        rect.min.y + y as f32 * self.cell_size,
                    ),
                    Vec2::new(self.cell_size, self.cell_size),
                );

                // Couleur de fond basée sur la température et le niveau d'eau
                let temp = grid.get_temperature(x, y);
                let t = (temp - 15.0) / 20.0;
                let mut bg_color =
                    Color32::from_rgb((255.0 * t) as u8, 0, (255.0 * (1.0 - t)) as u8);

                // Ajout d'une teinte bleue pour l'eau
                if grid.get_environment_type(x, y) == "water" {
                    bg_color = Color32::from_rgb(
                        (bg_color.r() as f32 * 0.7) as u8,
                        (bg_color.g() as f32 * 0.7) as u8,
                        255,
                    );
                }

                painter.rect_filled(cell_rect, 0.0, bg_color);

                // Dessine l'entité dans la case
                match grid.get_entity(x, y) {
                    Entity::Food(energy) => {
                        painter.circle_filled(
                            cell_rect.center(),
                            self.cell_size / 4.0,
                            Color32::GREEN,
                        );
                        painter.text(
                            cell_rect.center(),
                            Align2::CENTER_CENTER,
                            format!("{:.0}", energy),
                            egui::FontId::default(),
                            Color32::WHITE,
                        );
                    }
                    Entity::HeatSource(_) => {
                        painter.circle_filled(
                            cell_rect.center(),
                            self.cell_size / 3.0,
                            Color32::RED,
                        );
                    }
                    Entity::Obstacle(terrain_type) => {
                        let color = match terrain_type {
                            Terrain::Rock => Color32::GRAY,
                            Terrain::Sand => Color32::from_rgb(194, 178, 128),
                            Terrain::Soil => Color32::from_rgb(139, 69, 19),
                            Terrain::Wall => Color32::BLACK,
                        };
                        painter.rect_filled(cell_rect.shrink(4.0), 2.0, color);
                    }
                    Entity::Oxygen(level) => {
                        painter.circle_stroke(
                            cell_rect.center(),
                            self.cell_size / 4.0,
                            Stroke::new(
                                2.0,
                                Color32::from_rgba_premultiplied(
                                    255,
                                    255,
                                    255,
                                    (*level * 255.0) as u8,
                                ),
                            ),
                        );
                    }
                    Entity::Water(depth) => {
                        painter.rect_filled(
                            cell_rect,
                            0.0,
                            Color32::from_rgba_premultiplied(0, 0, 255, (*depth * 128.0) as u8),
                        );
                    }
                    Entity::Empty => {}
                    _ => {}
                }

                painter.rect_stroke(
                    cell_rect,
                    0.0,
                    Stroke::new(1.0, Color32::GRAY),
                    egui::StrokeKind::Inside,
                );
            }
        }

        // Dessine toutes les cellules
        for cell in cells {
            let cell_pos = Pos2::new(
                rect.min.x + cell.x() as f32 * self.cell_size + self.cell_size / 2.0,
                rect.min.y + cell.y() as f32 * self.cell_size + self.cell_size / 2.0,
            );

            let energy_ratio = cell.energy() / 100.0;
            let cell_color = Color32::from_rgb(
                (255.0 * (1.0 - energy_ratio)) as u8,
                (255.0 * energy_ratio) as u8,
                0,
            );

            painter.circle_filled(cell_pos, self.cell_size / 3.0, cell_color);
        }
    }
}
