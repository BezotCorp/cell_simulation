mod appearance_tab;
mod capabilities_tab;
mod condition_editor;
mod config_store;
mod configs_tab;
mod connection;
mod draft;
mod inbox;
mod live_tab;
mod panel_app;
mod pick;
mod start_tab;
mod tab;

#[cfg(test)]
mod tests;

use crate::application::LaunchOptions;

pub(crate) use appearance_tab::AppearanceTab;
pub(crate) use capabilities_tab::CapabilitiesTab;
pub(crate) use condition_editor::ConditionEditor;
pub(crate) use config_store::ConfigStore;
pub(crate) use configs_tab::ConfigsTab;
pub(crate) use connection::Connection;
pub(crate) use draft::Draft;
pub(crate) use inbox::Inbox;
pub(crate) use live_tab::LiveTab;
pub(crate) use panel_app::PanelApp;
pub(crate) use pick::Pick;
pub(crate) use start_tab::StartTab;
pub(crate) use tab::Tab;

/// Ouvre la fenêtre d'administration, qui se connecte à la simulation en cours.
pub(crate) fn run(options: &LaunchOptions) -> Result<(), String> {
    let port = options.admin_port();
    let native = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Cell Lab · laboratoire")
            .with_inner_size([1100.0, 760.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Cell Lab · laboratoire",
        native,
        Box::new(move |context| {
            let ctx = context.egui_ctx.clone();
            let connection = Connection::start(port, move || ctx.request_repaint());
            Ok(Box::new(PanelApp::new(connection)))
        }),
    )
    .map_err(|e| e.to_string())
}
