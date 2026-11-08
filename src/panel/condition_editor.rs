use crate::definitions::ConditionDefinition;
use crate::genetics::Signal;
use crate::panel::Pick;
use eframe::egui;

/// Éditeur d'une condition d'expression : présence, nombre, ordre, combinaison, répétition,
/// position ou signal du milieu.
pub(crate) struct ConditionEditor;

impl ConditionEditor {
    const KINDS: [&'static str; 6] = ["Nombre", "Même séquence", "Ordre", "Séquences identiques", "Position", "Milieu"];

    fn kind(condition: &ConditionDefinition) -> &'static str {
        match condition {
            ConditionDefinition::Count { .. } => Self::KINDS[0],
            ConditionDefinition::SameSequence { .. } => Self::KINDS[1],
            ConditionDefinition::Ordered { .. } => Self::KINDS[2],
            ConditionDefinition::Repeated { .. } => Self::KINDS[3],
            ConditionDefinition::Within { .. } => Self::KINDS[4],
            ConditionDefinition::Signal { .. } => Self::KINDS[5],
        }
    }

    fn fresh(kind: &str, first: &str) -> ConditionDefinition {
        let motif = first.to_owned();
        match kind {
            "Nombre" => ConditionDefinition::Count { motif, at_least: 1, at_most: None },
            "Même séquence" => ConditionDefinition::SameSequence { motifs: vec![motif] },
            "Ordre" => ConditionDefinition::Ordered { first: motif.clone(), then: motif },
            "Séquences identiques" => ConditionDefinition::Repeated { motif, copies: 2 },
            "Position" => ConditionDefinition::Within { motif, sequences: 4 },
            _ => ConditionDefinition::Signal { signal: Signal::Light, at_least: 0.1, at_most: None },
        }
    }

    /// Affiche la condition ; renvoie vrai si l'utilisateur demande sa suppression.
    pub(crate) fn show(ui: &mut egui::Ui, id: u64, condition: &mut ConditionDefinition, motifs: &[String]) -> bool {
        let mut remove = false;
        ui.horizontal_wrapped(|ui| {
            let mut kind = Self::kind(condition).to_owned();
            let kinds: Vec<String> = Self::KINDS.iter().map(|k| (*k).to_owned()).collect();
            if Pick::name(ui, (id, "kind"), &mut kind, &kinds) {
                *condition = Self::fresh(&kind, motifs.first().map_or("", String::as_str));
            }
            match condition {
                ConditionDefinition::Count { motif, at_least, at_most } => {
                    Pick::name(ui, (id, "m"), motif, motifs);
                    ui.label("au moins");
                    ui.add(egui::DragValue::new(at_least).range(0..=64));
                    let mut limited = at_most.is_some();
                    if ui.checkbox(&mut limited, "au plus").changed() {
                        *at_most = limited.then_some(*at_least + 1);
                    }
                    if let Some(most) = at_most {
                        ui.add(egui::DragValue::new(most).range(0..=64));
                    }
                }
                ConditionDefinition::SameSequence { motifs: chosen } => {
                    for (i, motif) in chosen.iter_mut().enumerate() {
                        Pick::name(ui, (id, "s", i), motif, motifs);
                    }
                    if ui.small_button("+ motif").clicked() {
                        chosen.push(motifs.first().cloned().unwrap_or_default());
                    }
                    if chosen.len() > 1 && ui.small_button("− motif").clicked() {
                        chosen.pop();
                    }
                }
                ConditionDefinition::Ordered { first, then } => {
                    Pick::name(ui, (id, "a"), first, motifs);
                    ui.label("avant");
                    Pick::name(ui, (id, "b"), then, motifs);
                }
                ConditionDefinition::Repeated { motif, copies } => {
                    Pick::name(ui, (id, "m"), motif, motifs);
                    ui.label("dans");
                    ui.add(egui::DragValue::new(copies).range(2..=16));
                    ui.label("séquences identiques");
                }
                ConditionDefinition::Within { motif, sequences } => {
                    Pick::name(ui, (id, "m"), motif, motifs);
                    ui.label("parmi les");
                    ui.add(egui::DragValue::new(sequences).range(1..=96));
                    ui.label("premières séquences");
                }
                ConditionDefinition::Signal { signal, at_least, at_most } => {
                    Pick::one(ui, (id, "sig"), signal, &Signal::ALL, |s| s.label().to_owned());
                    ui.label("entre");
                    ui.add(egui::DragValue::new(at_least).range(0.0..=1.0).speed(0.01));
                    let mut limited = at_most.is_some();
                    if ui.checkbox(&mut limited, "et").changed() {
                        *at_most = limited.then_some(1.0);
                    }
                    if let Some(most) = at_most {
                        ui.add(egui::DragValue::new(most).range(0.0..=1.0).speed(0.01));
                    }
                }
            }
            remove = ui.small_button("✖").clicked();
        });
        remove
    }
}
