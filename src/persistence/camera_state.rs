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
    /// Déplace la cible comme si l'on saisissait le sol : glisser vers la droite fait défiler le
    /// monde vers la droite, glisser vers le bas le rapproche du bas de l'écran. Le déplacement
    /// reste dans le plan du sol ; la hauteur de la cible ne change pas.
    pub(crate) fn pan(&mut self, dx: f32, dy: f32) {
        let scale = self.distance * 0.0018;
        // Une unité de glissement vertical couvre plus de sol quand la vue est rasante.
        let depth_scale = scale / self.pitch.sin().max(0.3);
        let (sin, cos) = self.yaw.sin_cos();
        // Droite de la caméra : (cos, 0, -sin) ; avant, projeté au sol : (-sin, 0, -cos).
        self.target[0] += -dx * scale * cos - dy * depth_scale * sin;
        self.target[2] += dx * scale * sin - dy * depth_scale * cos;
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
