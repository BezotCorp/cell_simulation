use crate::definitions::Definitions;
use crate::panel::ConfigStore;
use eframe::egui;

/// Onglet des configurations : enregistrer le brouillon sous un nom, le retrouver plus tard.
pub(crate) struct ConfigsTab {
    store: ConfigStore,
    name: String,
    names: Vec<String>,
    message: String,
}

impl ConfigsTab {
    pub(crate) fn new(store: ConfigStore) -> Self {
        let names = store.names();
        Self { store, name: String::new(), names, message: String::new() }
    }

    /// Renvoie la configuration à charger dans le brouillon, si l'utilisateur en a choisi une.
    pub(crate) fn show(&mut self, ui: &mut egui::Ui, draft: &Definitions) -> Option<Definitions> {
        let mut loaded = None;
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.name).hint_text("nom de la configuration").desired_width(240.0));
            if ui.button("💾 Enregistrer le brouillon").clicked() {
                let mut copy = draft.clone();
                copy.bump_revision();
                self.message = match self.store.save(&self.name, &copy) {
                    Ok(()) => format!("« {} » enregistrée", self.name.trim()),
                    Err(error) => error,
                };
                self.names = self.store.names();
            }
        });
        ui.weak(format!("Dossier : {}", self.store.directory().display()));
        ui.separator();
        let mut remove = None;
        for name in &self.names {
            ui.horizontal(|ui| {
                ui.label(name);
                if ui.button("Charger dans le brouillon").clicked() {
                    match self.store.load(name) {
                        Ok(definitions) => {
                            loaded = Some(definitions);
                            self.message = format!("« {name} » chargée dans le brouillon (pas encore appliquée)");
                        }
                        Err(error) => self.message = error,
                    }
                }
                if ui.button("Supprimer").clicked() {
                    remove = Some(name.clone());
                }
            });
        }
        if let Some(name) = remove {
            self.message = match self.store.delete(&name) {
                Ok(()) => format!("« {name} » supprimée"),
                Err(error) => error,
            };
            self.names = self.store.names();
        }
        if self.names.is_empty() {
            ui.label("Aucune configuration enregistrée.");
        }
        if !self.message.is_empty() {
            ui.separator();
            ui.label(&self.message);
        }
        loaded
    }
}
