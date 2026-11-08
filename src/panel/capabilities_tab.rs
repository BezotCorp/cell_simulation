use crate::definitions::{
    CapabilityDefinition, ConditionDefinition, ContributionDefinition, Definitions, MotifDefinition, PerCopyDefinition,
};
use crate::genetics::{Orientation, Product, Sensor};
use crate::panel::{ConditionEditor, Pick};
use eframe::egui;

/// Onglet des capacités biologiques : motifs génétiques, contributions et conditions.
#[derive(Default)]
pub(crate) struct CapabilitiesTab {
    selected: usize,
    new_motif: String,
}

impl CapabilitiesTab {
    pub(crate) fn show(&mut self, ui: &mut egui::Ui, definitions: &mut Definitions) {
        let genetics = definitions.species_mut().genetics_mut();
        let motifs: Vec<String> = genetics.motifs.keys().cloned().collect();
        ui.columns(2, |columns| {
            self.list(&mut columns[0], &mut genetics.capabilities, &motifs);
            egui::ScrollArea::vertical().id_salt("cap_edit").show(&mut columns[1], |ui| {
                if let Some(capability) = genetics.capabilities.get_mut(self.selected) {
                    Self::edit(ui, self.selected, capability, &motifs);
                } else {
                    ui.label("Aucune capacité sélectionnée.");
                }
            });
        });
        ui.separator();
        ui.collapsing("Motifs génétiques", |ui| self.motifs(ui, &mut genetics.motifs));
        ui.collapsing("Mutations, coûts et réseau neuronal", |ui| {
            let m = &mut genetics.mutation;
            ui.label("Taux par division et par bit/séquence (avant exposition au milieu) :");
            for (label, value) in [
                ("erreur de copie (par bit)", &mut m.bit_flip),
                ("duplication", &mut m.duplication),
                ("suppression", &mut m.deletion),
                ("insertion", &mut m.insertion),
                ("inversion", &mut m.inversion),
                ("déplacement", &mut m.translocation),
            ] {
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(value).range(0.0..=1.0).speed(0.0005).max_decimals(5));
                    ui.label(label);
                });
            }
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut m.min_length).range(1..=m.max_length));
                ui.label("longueur minimale du génome");
                ui.add(egui::DragValue::new(&mut m.max_length).range(m.min_length..=512));
                ui.label("maximale");
            });
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut m.repair_gain).range(0.0..=50.0).speed(0.05));
                ui.label("efficacité de la réparation");
            });
            let c = &mut genetics.costs;
            for (label, value) in [
                ("coût de réplication par séquence", &mut c.replication_per_sequence),
                ("surcoût de fidélité (par unité de réparation)", &mut c.fidelity_surcharge),
                ("entretien par séquence en plus", &mut c.upkeep_per_sequence),
                ("coût d'un transfert de fragment", &mut c.transfer),
            ] {
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(value).range(0.0..=10.0).speed(0.001));
                    ui.label(label);
                });
            }
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut genetics.neural.gain).speed(0.01));
                ui.label("gain neuronal");
                ui.add(egui::DragValue::new(&mut genetics.neural.bias).speed(0.1));
                ui.label("biais");
            });
            ui.label("Capteurs du réseau neuronal : le premier motif présent dans le génome choisit ce que le réseau lit.");
            let mut gone = None;
            for (i, (motif, sensor)) in genetics.neural.sensors.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    Pick::name(ui, ("sensor_motif", i), motif, &motifs);
                    Pick::one(ui, ("sensor", i), sensor, &Sensor::ALL, |s| s.label().to_owned());
                    if ui.small_button("✖").clicked() {
                        gone = Some(i);
                    }
                });
            }
            if let Some(i) = gone {
                genetics.neural.sensors.remove(i);
            }
            if ui.small_button("+ capteur").clicked() {
                genetics.neural.sensors.push((motifs.first().cloned().unwrap_or_default(), Sensor::Temperature));
            }
        });
    }

    fn list(&mut self, ui: &mut egui::Ui, capabilities: &mut Vec<CapabilityDefinition>, motifs: &[String]) {
        ui.heading("Capacités");
        egui::ScrollArea::vertical().id_salt("cap_list").max_height(360.0).show(ui, |ui| {
            for (i, capability) in capabilities.iter().enumerate() {
                let total: f32 = capability.contributions.iter().map(|c| c.amount).sum();
                let text = format!("{} · {} contribution(s) · Σ{:.2}", capability.product.label(), capability.contributions.len(), total);
                if ui.selectable_label(self.selected == i, text).clicked() {
                    self.selected = i;
                }
            }
        });
        ui.horizontal(|ui| {
            if ui.button("+ Nouvelle").clicked() {
                capabilities.push(CapabilityDefinition {
                    product: Product::Metabolism,
                    contributions: vec![ContributionDefinition {
                        when: vec![ConditionDefinition::Count { motif: motifs.first().cloned().unwrap_or_default(), at_least: 1, at_most: None }],
                        amount: 0.25,
                        per_copy: None,
                    }],
                });
                self.selected = capabilities.len() - 1;
            }
            if ui.button("Dupliquer").clicked() && self.selected < capabilities.len() {
                capabilities.push(capabilities[self.selected].clone());
                self.selected = capabilities.len() - 1;
            }
            if ui.button("Supprimer").clicked() && self.selected < capabilities.len() {
                capabilities.remove(self.selected);
                self.selected = self.selected.saturating_sub(1);
            }
        });
    }

    fn edit(ui: &mut egui::Ui, index: usize, capability: &mut CapabilityDefinition, motifs: &[String]) {
        ui.heading("Mécanisme");
        Pick::one(ui, (index, "product"), &mut capability.product, &Product::ALL, |p| p.label().to_owned());
        ui.weak(capability.product.description());
        ui.separator();
        ui.label("Le niveau du produit est la somme (bornée à 1) des contributions dont toutes les conditions sont vraies.");
        let mut removed = None;
        for (i, contribution) in capability.contributions.iter_mut().enumerate() {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("Contribution {}", i + 1));
                    ui.add(egui::DragValue::new(&mut contribution.amount).range(0.0..=1.0).speed(0.01));
                    if ui.small_button("Supprimer").clicked() {
                        removed = Some(i);
                    }
                });
                let mut gone = None;
                for (j, condition) in contribution.when.iter_mut().enumerate() {
                    let id = (index as u64) << 24 | (i as u64) << 12 | j as u64;
                    if ConditionEditor::show(ui, id, condition, motifs) {
                        gone = Some(j);
                    }
                }
                if let Some(j) = gone {
                    contribution.when.remove(j);
                }
                ui.horizontal(|ui| {
                    if ui.small_button("+ condition").clicked() {
                        contribution.when.push(ConditionDefinition::Count {
                            motif: motifs.first().cloned().unwrap_or_default(),
                            at_least: 1,
                            at_most: None,
                        });
                    }
                    let mut dosed = contribution.per_copy.is_some();
                    if ui.checkbox(&mut dosed, "dose par copie").changed() {
                        contribution.per_copy = dosed
                            .then(|| PerCopyDefinition { motif: motifs.first().cloned().unwrap_or_default(), up_to: 4 });
                    }
                    if let Some(dose) = &mut contribution.per_copy {
                        Pick::name(ui, (index, i, "dose"), &mut dose.motif, motifs);
                        ui.label("jusqu'à");
                        ui.add(egui::DragValue::new(&mut dose.up_to).range(1..=32));
                    }
                });
            });
        }
        if let Some(i) = removed {
            capability.contributions.remove(i);
        }
        if ui.button("+ contribution").clicked() {
            capability.contributions.push(ContributionDefinition {
                when: vec![ConditionDefinition::Count { motif: motifs.first().cloned().unwrap_or_default(), at_least: 1, at_most: None }],
                amount: 0.25,
                per_copy: None,
            });
        }
    }

    fn motifs(&mut self, ui: &mut egui::Ui, motifs: &mut std::collections::BTreeMap<String, MotifDefinition>) {
        ui.label("Un motif est une courte suite de bits (1 à 8) cherchée dans chaque séquence de 8 bits.");
        let mut removed = None;
        for (name, motif) in motifs.iter_mut() {
            ui.horizontal(|ui| {
                ui.monospace(format!("{name:<14}"));
                ui.add(egui::TextEdit::singleline(&mut motif.bits).desired_width(80.0).font(egui::TextStyle::Monospace));
                let valid = motif.pattern().is_ok();
                ui.label(if valid { "✔" } else { "✖ 1 à 8 bits (0/1)" });
                Pick::one(ui, ("orient", name.clone()), &mut motif.orientation, &Orientation::ALL, |o| o.label().to_owned());
                if ui.small_button("Supprimer").clicked() {
                    removed = Some(name.clone());
                }
            });
        }
        if let Some(name) = removed {
            motifs.remove(&name);
        }
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.new_motif).hint_text("nom du nouveau motif").desired_width(160.0));
            if ui.button("+ Motif").clicked() && !self.new_motif.trim().is_empty() {
                motifs.insert(
                    self.new_motif.trim().to_owned(),
                    MotifDefinition { bits: "1011".into(), orientation: Orientation::Forward },
                );
                self.new_motif.clear();
            }
        });
    }
}
