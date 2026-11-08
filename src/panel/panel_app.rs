use crate::admin::Order;
use crate::panel::{
    AppearanceTab, CapabilitiesTab, ConfigStore, ConfigsTab, Connection, Draft, LiveTab, StartTab, Tab,
};
use eframe::egui;

/// Fenêtre d'administration : édite un brouillon des définitions, l'applique à la simulation en
/// cours, et pilote l'expérience en direct.
pub(crate) struct PanelApp {
    connection: Connection,
    draft: Option<Draft>,
    tab: Tab,
    live: LiveTab,
    capabilities: CapabilitiesTab,
    appearance: AppearanceTab,
    start: StartTab,
    configs: ConfigsTab,
    save_path: String,
    seed_text: String,
}

impl PanelApp {
    pub(crate) fn new(connection: Connection) -> Self {
        Self {
            connection,
            draft: None,
            tab: Tab::Live,
            live: LiveTab::default(),
            capabilities: CapabilitiesTab::default(),
            appearance: AppearanceTab::default(),
            start: StartTab,
            configs: ConfigsTab::new(ConfigStore::new("data/configs")),
            save_path: String::new(),
            seed_text: String::new(),
        }
    }

    /// Intègre ce que la simulation a envoyé depuis la dernière image.
    fn absorb(&mut self) -> (bool, Option<crate::admin::Status>, Vec<(bool, String)>) {
        let Ok(mut inbox) = self.connection.inbox().lock() else {
            return (false, None, Vec::new());
        };
        if let Some(text) = inbox.definitions.take() {
            self.save_path = inbox.save_path.clone();
            match &mut self.draft {
                Some(draft) if draft.differs() => draft.synchronize(text),
                slot => match Draft::from_simulation(text) {
                    Ok(draft) => *slot = Some(draft),
                    Err(error) => inbox.log.push((false, error)),
                },
            }
        }
        (inbox.connected, inbox.status.clone(), inbox.log.iter().rev().take(6).cloned().collect())
    }
}

impl eframe::App for PanelApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let (connected, status, log) = self.absorb();
        egui::Panel::top("bar").show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.heading("Cell Lab · laboratoire");
                ui.label(if connected { "🟢 connecté à la simulation" } else { "🔴 en attente de la simulation…" });
                if let Some(draft) = &self.draft {
                    let dirty = draft.differs();
                    ui.separator();
                    ui.label(if dirty { "✏ brouillon modifié" } else { "brouillon = simulation" });
                    if ui.add_enabled(dirty, egui::Button::new("▶ Appliquer en direct")).on_hover_text("Les cellules vivantes gardent leur génome ; capacités, rendus et monde sont remplacés.").clicked() {
                        self.connection.send(Order::Redefine(draft.text()));
                        self.connection.send(Order::Refresh);
                    }
                    ui.add(egui::TextEdit::singleline(&mut self.seed_text).hint_text("graine").desired_width(70.0));
                    if ui.button("↻ Redémarrer avec le brouillon").on_hover_text("Nouvelle expérience : cellules initiales du brouillon, monde régénéré.").clicked() {
                        self.connection.send(Order::Restart { definitions: draft.text(), seed: self.seed_text.trim().parse().ok() });
                        self.connection.send(Order::Refresh);
                    }
                    if ui.add_enabled(dirty, egui::Button::new("↩ Annuler les modifications")).clicked() {
                        self.connection.send(Order::Refresh);
                        self.draft = None;
                    }
                }
            });
            ui.horizontal(|ui| {
                for tab in Tab::ALL {
                    ui.selectable_value(&mut self.tab, tab, tab.label());
                }
            });
        });
        egui::Panel::bottom("log").show(ui, |ui| {
            for (ok, message) in log {
                ui.colored_label(if ok { egui::Color32::LIGHT_GREEN } else { egui::Color32::LIGHT_RED }, message);
            }
            if !self.save_path.is_empty() {
                ui.weak(format!("Sauvegarde : {}", self.save_path));
            }
        });
        egui::CentralPanel::default().show(ui, |ui| {
            let Some(draft) = &mut self.draft else {
                ui.label("En attente des définitions de la simulation…");
                return;
            };
            egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| match self.tab {
                Tab::Live => self.live.show(ui, &self.connection, status.as_ref(), &draft.definitions),
                Tab::Capabilities => self.capabilities.show(ui, &mut draft.definitions),
                Tab::Appearance => self.appearance.show(ui, &mut draft.definitions),
                Tab::Start => self.start.show(ui, &mut draft.definitions),
                Tab::Configs => {
                    if let Some(loaded) = self.configs.show(ui, &draft.definitions) {
                        draft.replace(loaded);
                    }
                }
            });
        });
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(250));
    }
}
