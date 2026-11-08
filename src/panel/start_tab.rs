use crate::definitions::{Definitions, FounderDefinition};
use crate::panel::Pick;
use eframe::egui;

/// Onglet du départ de l'expérience : cellules initiales, règles de la biologie, monde.
#[derive(Default)]
pub(crate) struct StartTab;

impl StartTab {
    pub(crate) fn show(&mut self, ui: &mut egui::Ui, definitions: &mut Definitions) {
        ui.horizontal(|ui| {
            ui.label("Nom de l'espèce");
            ui.text_edit_singleline(definitions.species_mut().name_mut());
        });
        ui.heading("Cellules initiales");
        ui.weak("Chaque groupe démarre avec son génome. Sans groupe, une seule cellule descendante de l'ancêtre apparaît.");
        let motifs: Vec<String> = definitions.species().genetics_definition().motifs.keys().cloned().collect();
        let founders = definitions.species_mut().founders_mut();
        let mut gone = None;
        for (i, group) in founders.iter_mut().enumerate() {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("Groupe {}", i + 1));
                    ui.add(egui::DragValue::new(&mut group.count).range(1..=5000));
                    ui.label("cellules, énergie initiale");
                    ui.add(egui::DragValue::new(&mut group.energy).range(0.05..=1.0).speed(0.01));
                    ui.checkbox(&mut group.scatter, "réparties partout");
                    if ui.small_button("Supprimer").clicked() {
                        gone = Some(i);
                    }
                });
                ui.horizontal_wrapped(|ui| {
                    ui.label("Motifs ajoutés au génome :");
                    let mut remove = None;
                    for (j, motif) in group.add_motifs.iter_mut().enumerate() {
                        Pick::name(ui, ("founder", i, j), motif, &motifs);
                        if ui.small_button("✖").clicked() {
                            remove = Some(j);
                        }
                    }
                    if let Some(j) = remove {
                        group.add_motifs.remove(j);
                    }
                    if ui.small_button("+ motif").clicked() {
                        group.add_motifs.push(motifs.first().cloned().unwrap_or_default());
                    }
                });
                let mut custom = group.sequences.is_some();
                if ui.checkbox(&mut custom, "génome entièrement personnalisé (séquences de 8 bits)").changed() {
                    group.sequences = custom.then(Vec::new);
                }
                if let Some(sequences) = &mut group.sequences {
                    let mut text = sequences.join(" ");
                    if ui.add(egui::TextEdit::multiline(&mut text).desired_width(f32::INFINITY).font(egui::TextStyle::Monospace)).changed() {
                        *sequences = text.split_whitespace().map(str::to_owned).collect();
                    }
                }
            });
        }
        if let Some(i) = gone {
            founders.remove(i);
        }
        if ui.button("+ Groupe de cellules").clicked() {
            founders.push(FounderDefinition::default());
        }

        ui.separator();
        ui.collapsing("Génome de l'ancêtre", |ui| {
            let sequences = &mut definitions.species_mut().ancestor_mut().sequences;
            let mut text = sequences.join(" ");
            ui.label("Séquences de 8 bits, séparées par des espaces :");
            if ui.add(egui::TextEdit::multiline(&mut text).desired_width(f32::INFINITY).font(egui::TextStyle::Monospace)).changed() {
                *sequences = text.split_whitespace().map(str::to_owned).collect();
            }
        });
        ui.collapsing("Biologie : énergie et cadence", |ui| {
            let rules = definitions.species_mut().rules_mut();
            for (label, value, max) in [
                ("capacité d'énergie du corps de référence", &mut rules.energy_capacity, 1000.0),
                ("métabolisme de base (par seconde)", &mut rules.basal_metabolism, 10.0),
                ("secondes entre deux déplacements", &mut rules.movement_seconds, 30.0),
                ("secondes de métabolisme par déplacement", &mut rules.movement_energy_seconds, 30.0),
                ("réserve minimale pour se déplacer", &mut rules.movement_reserve, 500.0),
            ] {
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(value).range(0.01..=max).speed(0.05));
                    ui.label(label);
                });
            }
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut rules.maturity_seconds).range(1..=3600));
                ui.label("secondes avant maturité");
            });
        });
        ui.collapsing("Monde", |ui| {
            let world = definitions.world_mut();
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut world.size).range(1..=128));
                ui.label("habitats par côté (redémarrage requis)");
                ui.add(egui::DragValue::new(&mut world.seed));
                ui.label("graine");
            });
            for (label, value, max) in [
                ("réserve alimentaire maximale d'un site", &mut world.food_capacity, 500.0),
                ("régénération de nourriture par seconde", &mut world.food_regeneration_per_second, 50.0),
                ("température ambiante (°C)", &mut world.ambient_temperature, 60.0),
            ] {
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(value).range(-20.0..=max).speed(0.1));
                    ui.label(label);
                });
            }
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut world.food_attempts).range(0..=100_000));
                ui.label("tentatives de placement de nourriture");
                ui.add(egui::DragValue::new(&mut world.heat_sources).range(0..=1000));
                ui.label("sources de chaleur");
            });
            for (label, value) in [
                ("probabilité de diffusion des ressources", &mut world.resource_spread_probability),
                ("probabilité d'obstacle", &mut world.terrain_probability),
                ("probabilité d'oxygène", &mut world.oxygen_probability),
                ("probabilité de flaque", &mut world.water_probability),
            ] {
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(value).range(0.0..=1.0).speed(0.005));
                    ui.label(label);
                });
            }
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut world.water_decay_per_second).range(0.9..=1.0).speed(0.00001).max_decimals(6));
                ui.label("conservation de l'eau par seconde");
            });
        });
    }
}
