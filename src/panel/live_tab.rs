use crate::admin::{Order, Status};
use crate::definitions::{Definitions, FounderDefinition};
use crate::ecosystem::{Intervention, KillTarget, ResourceKind};
use crate::panel::{Connection, Pick};
use eframe::egui;

/// Onglet de pilotage en direct : cadence, création et mort de cellules, orages, ressources.
pub(crate) struct LiveTab {
    group: FounderDefinition,
    kill_fraction: f32,
    kill_count: u32,
    storm_sites: u32,
    storm_intensity: f32,
    rain_sites: u32,
    rain_amount: f32,
    resource: ResourceKind,
    resource_count: u32,
    resource_amount: f32,
    speed: f32,
}

impl Default for LiveTab {
    fn default() -> Self {
        Self {
            group: FounderDefinition { count: 10, ..FounderDefinition::default() },
            kill_fraction: 0.5,
            kill_count: 10,
            storm_sites: 4,
            storm_intensity: 2.5,
            rain_sites: 12,
            rain_amount: 0.6,
            resource: ResourceKind::Food,
            resource_count: 20,
            resource_amount: 20.0,
            speed: 1.0,
        }
    }
}

impl LiveTab {
    pub(crate) fn show(&mut self, ui: &mut egui::Ui, connection: &Connection, status: Option<&Status>, definitions: &Definitions) {
        let act = |intervention: Intervention| connection.send(Order::Intervene(intervention));
        if let Some(status) = status {
            ui.horizontal_wrapped(|ui| {
                ui.strong(format!("{} cellules", status.cells));
                ui.label(format!("· {:.0} s simulées · {} naissances · {} morts", status.seconds, status.births, status.deaths));
                ui.label(format!("· énergie moyenne {:.1} · {} génomes distincts · eau {:.0} %", status.average_energy, status.genomes, status.flooded * 100.0));
            });
            ui.horizontal_wrapped(|ui| {
                for (label, count) in &status.census {
                    ui.label(format!("{label} {count}"));
                }
            });
            self.speed = status.speed;
        }
        ui.horizontal(|ui| {
            let paused = status.is_some_and(|s| s.paused);
            if ui.button(if paused { "▶ Reprendre" } else { "⏸ Pause" }).clicked() {
                connection.send(Order::SetPaused(!paused));
            }
            if ui.button("⏭ Un pas").clicked() {
                connection.send(Order::SetPaused(true));
                connection.send(Order::Step);
            }
            for speed in [0.25, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0] {
                if ui.selectable_label((self.speed - speed).abs() < 1e-3, format!("× {speed}")).clicked() {
                    connection.send(Order::SetSpeed(speed));
                }
            }
            if ui.button("💾 Sauvegarder").clicked() {
                connection.send(Order::Save);
            }
        });
        ui.separator();

        ui.heading("Créer des cellules");
        let motifs: Vec<String> = definitions.species().genetics_definition().motifs.keys().cloned().collect();
        ui.horizontal_wrapped(|ui| {
            ui.add(egui::DragValue::new(&mut self.group.count).range(1..=5000));
            ui.label("cellules, énergie");
            ui.add(egui::DragValue::new(&mut self.group.energy).range(0.05..=1.0).speed(0.01));
            ui.checkbox(&mut self.group.scatter, "réparties partout");
        });
        ui.horizontal_wrapped(|ui| {
            ui.label("avec les motifs :");
            let mut remove = None;
            for (i, motif) in self.group.add_motifs.iter_mut().enumerate() {
                Pick::name(ui, ("live_motif", i), motif, &motifs);
                if ui.small_button("✖").clicked() {
                    remove = Some(i);
                }
            }
            if let Some(i) = remove {
                self.group.add_motifs.remove(i);
            }
            if ui.small_button("+ motif").clicked() {
                self.group.add_motifs.push(motifs.first().cloned().unwrap_or_default());
            }
        });
        if ui.button("➕ Créer").clicked() {
            act(Intervention::Spawn { group: self.group.clone(), at: None });
        }

        ui.separator();
        ui.heading("Supprimer des cellules");
        ui.horizontal_wrapped(|ui| {
            ui.add(egui::Slider::new(&mut self.kill_fraction, 0.0..=1.0).text("part"));
            if ui.button("☠ Tuer cette part").clicked() {
                act(Intervention::Kill(KillTarget::Fraction(self.kill_fraction)));
            }
            ui.add(egui::DragValue::new(&mut self.kill_count).range(1..=100_000));
            if ui.button("Tuer au hasard").clicked() {
                act(Intervention::Kill(KillTarget::Count(self.kill_count)));
            }
            if ui.button("Tuer les plus faibles").clicked() {
                act(Intervention::Kill(KillTarget::Weakest(self.kill_count)));
            }
            if ui.button("Tout tuer").clicked() {
                act(Intervention::Kill(KillTarget::Everyone));
            }
        });

        ui.separator();
        ui.heading("Météo");
        ui.horizontal_wrapped(|ui| {
            ui.add(egui::DragValue::new(&mut self.storm_sites).range(1..=500));
            ui.label("sites, intensité ×");
            ui.add(egui::DragValue::new(&mut self.storm_intensity).range(1.0..=10.0).speed(0.1));
            if ui.button("⚡ Déclencher un orage").clicked() {
                act(Intervention::Storm { sites: self.storm_sites, intensity: self.storm_intensity });
            }
        });
        ui.horizontal_wrapped(|ui| {
            ui.add(egui::DragValue::new(&mut self.rain_sites).range(1..=2000));
            ui.label("sites, quantité");
            ui.add(egui::DragValue::new(&mut self.rain_amount).range(0.05..=10.0).speed(0.05));
            if ui.button("☁ Former des nuages").clicked() {
                act(Intervention::Rain { sites: self.rain_sites, amount: self.rain_amount });
            }
        });

        ui.separator();
        ui.heading("Ressources");
        ui.horizontal_wrapped(|ui| {
            Pick::one(ui, "resource", &mut self.resource, &ResourceKind::ALL, |r| r.label().to_owned());
            ui.add(egui::DragValue::new(&mut self.resource_count).range(1..=5000));
            ui.label("éléments, valeur");
            ui.add(egui::DragValue::new(&mut self.resource_amount).range(0.0..=500.0).speed(0.5));
            if ui.button("Ajouter").clicked() {
                act(Intervention::Scatter { resource: self.resource, count: self.resource_count, amount: self.resource_amount });
            }
            if ui.button("🍃 Remplir toute la nourriture").clicked() {
                act(Intervention::FillFood);
            }
        });
        ui.weak("L'environnement permanent (nourriture, température ambiante…) se règle dans l'onglet Départ, puis « Appliquer en direct ».");
    }
}
