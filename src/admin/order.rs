use crate::ecosystem::Intervention;
use serde::{Deserialize, Serialize};

/// Demande de la fenêtre d'administration à la simulation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) enum Order {
    /// Agit sur l'expérience en cours.
    Intervene(Intervention),
    /// Adopte ces définitions (texte RON) sans perdre les cellules vivantes.
    Redefine(String),
    /// Démarre une nouvelle expérience avec ces définitions et cette graine.
    Restart { definitions: String, seed: Option<u64> },
    SetPaused(bool),
    /// Avance de quelques pas pendant la pause.
    Step,
    /// Vitesse de simulation, en multiple du temps réel.
    SetSpeed(f32),
    /// Sauvegarde l'expérience dans le fichier de sauvegarde courant.
    Save,
    /// Renvoie les définitions courantes.
    Refresh,
}
