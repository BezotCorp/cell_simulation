use crate::definitions::{
    Definitions, MaterialKind, PartAxis, PartCount, PartDefinition, PartDriver, PartPlacement, PartShape, PartVariant,
    SignalRange,
};
use crate::genetics::{Product, Signal};
use crate::panel::Pick;
use eframe::egui;

/// Onglet des rendus 3D : composants visibles, leurs conditions d'apparition, leurs variations
/// selon le développement, et les matériaux.
#[derive(Default)]
pub(crate) struct AppearanceTab {
    selected: usize,
}

impl AppearanceTab {
    pub(crate) fn show(&mut self, ui: &mut egui::Ui, definitions: &mut Definitions) {
        ui.columns(2, |columns| {
            let parts = definitions.species_mut().parts_mut();
            self.list(&mut columns[0], parts);
            egui::ScrollArea::vertical().id_salt("part_edit").show(&mut columns[1], |ui| {
                if let Some(part) = parts.get_mut(self.selected) {
                    Self::edit(ui, part);
                }
            });
        });
        ui.separator();
        ui.collapsing("Matériaux (couleurs et brillance)", |ui| {
            for (kind, material) in definitions.appearance_mut().materials.iter_mut() {
                ui.horizontal(|ui| {
                    ui.monospace(format!("{:<11}", kind.label()));
                    ui.color_edit_button_srgba_unmultiplied(&mut material.color);
                    ui.add(egui::Slider::new(&mut material.roughness, 0.0..=1.0).text("rugosité"));
                    ui.add(egui::Slider::new(&mut material.metallic, 0.0..=1.0).text("métal"));
                    ui.add(egui::Slider::new(&mut material.emission, 0.0..=3.0).text("émission"));
                });
            }
        });
    }

    fn list(&mut self, ui: &mut egui::Ui, parts: &mut Vec<PartDefinition>) {
        ui.heading("Composants 3D");
        egui::ScrollArea::vertical().id_salt("part_list").max_height(380.0).show(ui, |ui| {
            for (i, part) in parts.iter().enumerate() {
                if ui.selectable_label(self.selected == i, format!("{} · {}", part.name, Self::driver_label(part.driver))).clicked() {
                    self.selected = i;
                }
            }
        });
        ui.horizontal(|ui| {
            if ui.button("+ Nouveau").clicked() {
                parts.push(PartDefinition {
                    name: format!("composant {}", parts.len() + 1),
                    driver: PartDriver::Product(Product::Metabolism),
                    appears_at: 0.25,
                    when: Vec::new(),
                    count: PartCount { per_level: 4.0, max: 12 },
                    placement: PartPlacement::Spread { low: -1.0, high: 1.0 },
                    shape: PartShape::Sphere,
                    material: MaterialKind::Healthy,
                    size: [[0.15; 3], [0.3; 3]],
                    axis: PartAxis::Y,
                    reach: 1.0,
                    variants: Vec::new(),
                    models: Vec::new(),
                });
                self.selected = parts.len() - 1;
            }
            if ui.button("Dupliquer").clicked() && self.selected < parts.len() {
                let mut copy = parts[self.selected].clone();
                copy.name.push_str(" (copie)");
                parts.push(copy);
                self.selected = parts.len() - 1;
            }
            if ui.button("Supprimer").clicked() && self.selected < parts.len() {
                parts.remove(self.selected);
                self.selected = self.selected.saturating_sub(1);
            }
        });
    }

    fn driver_label(driver: PartDriver) -> String {
        match driver {
            PartDriver::Product(product) => product.label().to_owned(),
            PartDriver::NeuralUnits => "Unités nerveuses".into(),
            PartDriver::Always => "Toujours".into(),
        }
    }

    fn edit(ui: &mut egui::Ui, part: &mut PartDefinition) {
        ui.heading("Apparition");
        ui.horizontal(|ui| {
            ui.label("Nom");
            ui.text_edit_singleline(&mut part.name);
        });
        ui.horizontal(|ui| {
            ui.label("Développé par");
            let mut options: Vec<PartDriver> = Product::ALL.iter().map(|p| PartDriver::Product(*p)).collect();
            options.push(PartDriver::NeuralUnits);
            options.push(PartDriver::Always);
            Pick::one(ui, "driver", &mut part.driver, &options, |d| Self::driver_label(*d));
        });
        ui.horizontal(|ui| {
            ui.label("Apparaît à partir du niveau");
            ui.add(egui::DragValue::new(&mut part.appears_at).range(0.0..=1.0).speed(0.01));
        });
        ui.label("Conditions sur le milieu d'acclimatation :");
        let mut gone = None;
        for (i, range) in part.when.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                Pick::one(ui, ("sig", i), &mut range.signal, &Signal::ALL, |s| s.label().to_owned());
                ui.label("entre");
                ui.add(egui::DragValue::new(&mut range.at_least).range(0.0..=1.0).speed(0.01));
                let mut limited = range.at_most.is_some();
                if ui.checkbox(&mut limited, "et").changed() {
                    range.at_most = limited.then_some(1.0);
                }
                if let Some(most) = &mut range.at_most {
                    ui.add(egui::DragValue::new(most).range(0.0..=1.0).speed(0.01));
                }
                if ui.small_button("✖").clicked() {
                    gone = Some(i);
                }
            });
        }
        if let Some(i) = gone {
            part.when.remove(i);
        }
        if ui.small_button("+ condition de milieu").clicked() {
            part.when.push(SignalRange { signal: Signal::Water, at_least: 0.5, at_most: None });
        }

        ui.separator();
        ui.heading("Forme");
        ui.horizontal(|ui| {
            Pick::one(ui, "shape", &mut part.shape, &[PartShape::Sphere, PartShape::Crystal, PartShape::Cube], |s| format!("{s:?}"));
            Pick::one(ui, "material", &mut part.material, &MaterialKind::ALL, |m| m.label());
            ui.label("axe");
            Pick::one(ui, "axis", &mut part.axis, &[PartAxis::Y, PartAxis::Z], |a| format!("{a:?}"));
        });
        ui.horizontal(|ui| {
            ui.label("exemplaires = niveau ×");
            ui.add(egui::DragValue::new(&mut part.count.per_level).range(0.1..=64.0).speed(0.1));
            ui.label("× taille, au plus");
            ui.add(egui::DragValue::new(&mut part.count.max).range(1..=40));
        });
        ui.horizontal(|ui| {
            ui.label("distance au centre");
            ui.add(egui::DragValue::new(&mut part.reach).range(0.1..=3.0).speed(0.01));
        });
        for (label, row) in [("taille à développement nul", 0), ("taille à développement complet", 1)] {
            ui.horizontal(|ui| {
                ui.label(label);
                for axis in &mut part.size[row] {
                    ui.add(egui::DragValue::new(axis).range(0.01..=3.0).speed(0.005));
                }
            });
        }
        Self::placement(ui, &mut part.placement);

        ui.separator();
        ui.heading("Variations selon le développement");
        let mut gone = None;
        for (i, variant) in part.variants.iter_mut().enumerate() {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label("à partir du niveau");
                    ui.add(egui::DragValue::new(&mut variant.from).range(0.0..=1.0).speed(0.01));
                    ui.label("échelle");
                    ui.add(egui::DragValue::new(&mut variant.scale).range(0.2..=4.0).speed(0.01));
                    if ui.small_button("✖").clicked() {
                        gone = Some(i);
                    }
                });
                ui.horizontal(|ui| {
                    let mut recolored = variant.material.is_some();
                    if ui.checkbox(&mut recolored, "autre matériau").changed() {
                        variant.material = recolored.then_some(MaterialKind::Healthy);
                    }
                    if let Some(material) = &mut variant.material {
                        Pick::one(ui, ("vm", i), material, &MaterialKind::ALL, |m| m.label());
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("modèle GLB");
                    let mut text = variant.model.clone().unwrap_or_default();
                    if ui.add(egui::TextEdit::singleline(&mut text).hint_text("components/….glb").desired_width(220.0)).changed() {
                        variant.model = (!text.trim().is_empty()).then_some(text);
                    }
                });
            });
        }
        if let Some(i) = gone {
            part.variants.remove(i);
        }
        if ui.button("+ variation").clicked() {
            let from = part.variants.last().map_or(0.5, |v| (v.from + 0.25).min(1.0));
            part.variants.push(PartVariant { from, material: None, scale: 1.2, model: None });
        }
        ui.horizontal(|ui| {
            ui.label("Modèles GLB tour à tour (data/assets/)");
            let mut text = part.models.join(", ");
            if ui.add(egui::TextEdit::singleline(&mut text).desired_width(260.0)).changed() {
                part.models = text.split(',').map(str::trim).filter(|s| !s.is_empty()).map(str::to_owned).collect();
            }
        });
    }

    fn placement(ui: &mut egui::Ui, placement: &mut PartPlacement) {
        let kinds = ["Répartie", "Couronne", "Faisceau", "Unique"];
        let mut current = match placement {
            PartPlacement::Spread { .. } => kinds[0],
            PartPlacement::Ring => kinds[1],
            PartPlacement::Bundle { .. } => kinds[2],
            PartPlacement::Fixed { .. } => kinds[3],
        }
        .to_owned();
        ui.horizontal(|ui| {
            ui.label("Placement");
            let names: Vec<String> = kinds.iter().map(|k| (*k).to_owned()).collect();
            if Pick::name(ui, "placement", &mut current, &names) {
                *placement = match current.as_str() {
                    "Répartie" => PartPlacement::Spread { low: -1.0, high: 1.0 },
                    "Couronne" => PartPlacement::Ring,
                    "Faisceau" => PartPlacement::Bundle { front: true, lift: 0.2, around: 0.2 },
                    _ => PartPlacement::Fixed { x: 0.0, y: 1.0, z: 0.0 },
                };
            }
            match placement {
                PartPlacement::Spread { low, high } => {
                    ui.label("du bas");
                    ui.add(egui::DragValue::new(low).range(-1.0..=1.0).speed(0.01));
                    ui.label("au haut");
                    ui.add(egui::DragValue::new(high).range(-1.0..=1.0).speed(0.01));
                }
                PartPlacement::Ring => {}
                PartPlacement::Bundle { front, lift, around } => {
                    ui.checkbox(front, "à l'avant");
                    ui.label("soulevé");
                    ui.add(egui::DragValue::new(lift).range(-1.0..=1.0).speed(0.01));
                    ui.label("écarté");
                    ui.add(egui::DragValue::new(around).range(0.0..=1.0).speed(0.01));
                }
                PartPlacement::Fixed { x, y, z } => {
                    for v in [x, y, z] {
                        ui.add(egui::DragValue::new(v).range(-1.0..=1.0).speed(0.01));
                    }
                }
            }
        });
    }
}
