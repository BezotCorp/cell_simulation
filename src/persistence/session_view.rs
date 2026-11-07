use crate::persistence::{CameraState, ViewMode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct SessionView {
    paused: bool,
    speed: f32,
    mode: ViewMode,
    camera: CameraState,
    selected: Option<Uuid>,
    habitat: Option<Uuid>,
}
impl Default for SessionView {
    fn default() -> Self {
        Self {
            paused: false,
            speed: 1.0,
            mode: ViewMode::Ecosystem,
            camera: CameraState::default(),
            selected: None,
            habitat: None,
        }
    }
}
impl SessionView {
    pub(crate) fn paused(&self) -> bool {
        self.paused
    }
    pub(crate) fn speed(&self) -> f32 {
        self.speed
    }
    pub(crate) fn mode(&self) -> ViewMode {
        self.mode
    }
    pub(crate) fn camera(&self) -> &CameraState {
        &self.camera
    }
    pub(crate) fn selected(&self) -> Option<Uuid> {
        self.selected
    }
    pub(crate) fn habitat(&self) -> Option<Uuid> {
        self.habitat
    }
    pub(crate) fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }
    pub(crate) fn pause(&mut self) {
        self.paused = true;
    }
    pub(crate) fn slower(&mut self) {
        self.speed = (self.speed / 2.0).max(0.25);
    }
    pub(crate) fn faster(&mut self) {
        self.speed = (self.speed * 2.0).min(64.0);
    }
    pub(crate) fn cycle_mode(&mut self) {
        self.mode = match self.mode {
            ViewMode::Ecosystem => ViewMode::Temperature,
            ViewMode::Temperature => ViewMode::Adaptation,
            ViewMode::Adaptation => ViewMode::Ecosystem,
        };
    }
    pub(crate) fn select_organism(&mut self, id: Uuid) {
        self.selected = Some(id);
        self.habitat = None;
    }
    pub(crate) fn select_habitat(&mut self, id: Uuid) {
        self.habitat = Some(id);
        self.selected = None;
    }
    pub(crate) fn clear_selection(&mut self) {
        self.selected = None;
        self.habitat = None;
    }
    pub(crate) fn focus(&mut self, target: [f32; 3], distance: f32) {
        self.camera.focus(target, distance);
    }
    pub(crate) fn orbit(&mut self, dx: f32, dy: f32) {
        self.camera.orbit(dx, dy);
    }
    pub(crate) fn pan(&mut self, dx: f32, dy: f32) {
        self.camera.pan(dx, dy);
    }
    pub(crate) fn zoom(&mut self, amount: f32) {
        self.camera.zoom(amount);
    }
    pub(crate) fn validate(&self) -> Result<(), String> {
        self.camera.validate()?;
        if !self.speed.is_finite() || !(0.25..=64.0).contains(&self.speed) {
            return Err("Vitesse de simulation invalide.".into());
        }
        if self.selected.is_some() && self.habitat.is_some() {
            return Err("Sélection ambiguë.".into());
        }
        Ok(())
    }
}
