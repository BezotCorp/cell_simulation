use serde::{Deserialize, Serialize};

/// Instantané de l'expérience, envoyé régulièrement à la fenêtre d'administration.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct Status {
    pub(crate) seconds: f64,
    pub(crate) cells: usize,
    pub(crate) births: u64,
    pub(crate) deaths: u64,
    pub(crate) average_energy: f32,
    pub(crate) flooded: f32,
    pub(crate) paused: bool,
    pub(crate) speed: f32,
    pub(crate) world_size: usize,
    /// Génomes distincts vivants.
    pub(crate) genomes: usize,
    /// Cellules qui expriment chaque produit (libellé, nombre).
    pub(crate) census: Vec<(String, usize)>,
}
