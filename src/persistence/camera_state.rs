use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct CameraState {
    yaw: f32,
    pitch: f32,
    distance: f32,
    target: [f32; 3],
}
impl Default for CameraState {
    fn default() -> Self {
        Self {
            yaw: -0.65,
            pitch: 0.82,
            distance: 15.5,
            target: [0.0; 3],
        }
    }
}
impl CameraState {
    pub(crate) fn yaw(&self) -> f32 {
        self.yaw
    }
    pub(crate) fn pitch(&self) -> f32 {
        self.pitch
    }
    pub(crate) fn distance(&self) -> f32 {
        self.distance
    }
    pub(crate) fn target(&self) -> [f32; 3] {
        self.target
    }
    pub(crate) fn focus(&mut self, target: [f32; 3], distance: f32) {
        self.target = target;
        self.distance = distance.clamp(2.0, 300.0);
    }
    pub(crate) fn orbit(&mut self, dx: f32, dy: f32) {
        self.yaw -= dx * 0.006;
        self.pitch = (self.pitch + dy * 0.006).clamp(0.1, 1.5);
    }
    pub(crate) fn pan(&mut self, dx: f32, dy: f32) {
        let scale = self.distance * 0.0018;
        self.target[0] -= dx * scale * self.yaw.cos();
        self.target[2] += dx * scale * self.yaw.sin();
        self.target[1] += dy * scale;
    }
    pub(crate) fn zoom(&mut self, amount: f32) {
        self.distance = (self.distance * (1.0 - amount * 0.09)).clamp(2.0, 300.0);
    }
    pub(crate) fn validate(&self) -> Result<(), String> {
        if !self.yaw.is_finite()
            || !self.pitch.is_finite()
            || !(0.1..=1.5).contains(&self.pitch)
            || !self.distance.is_finite()
            || !(2.0..=300.0).contains(&self.distance)
            || self
                .target
                .iter()
                .any(|v| !v.is_finite() || v.abs() > 10000.0)
        {
            return Err("Caméra invalide.".into());
        }
        Ok(())
    }
}
