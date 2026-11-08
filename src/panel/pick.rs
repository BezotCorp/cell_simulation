use eframe::egui;
use std::hash::Hash;

/// Listes déroulantes réutilisées par tous les éditeurs.
pub(crate) struct Pick;

impl Pick {
    /// Choix parmi `options` ; renvoie vrai si la valeur a changé.
    pub(crate) fn one<T: PartialEq + Clone>(
        ui: &mut egui::Ui,
        id: impl Hash + std::fmt::Debug,
        current: &mut T,
        options: &[T],
        label: impl Fn(&T) -> String,
    ) -> bool {
        let mut changed = false;
        egui::ComboBox::from_id_salt(id)
            .selected_text(label(current))
            .show_ui(ui, |ui| {
                for option in options {
                    if ui.selectable_label(option == current, label(option)).clicked() && option != current {
                        *current = option.clone();
                        changed = true;
                    }
                }
            });
        changed
    }

    /// Choix d'un nom parmi `names` ; une valeur inconnue reste affichée telle quelle.
    pub(crate) fn name(ui: &mut egui::Ui, id: impl Hash + std::fmt::Debug, current: &mut String, names: &[String]) -> bool {
        Self::one(ui, id, current, names, |name| name.clone())
    }
}
