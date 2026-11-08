use crate::admin::Status;
use serde::{Deserialize, Serialize};

/// Réponse ou information de la simulation à la fenêtre d'administration.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) enum Report {
    /// Définitions courantes (texte RON) et fichier de sauvegarde.
    Definitions { ron: String, save_path: String },
    Status(Status),
    /// Résultat d'une demande.
    Done { ok: bool, message: String },
}
