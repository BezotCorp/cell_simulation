use crate::admin::Status;

/// Ce que la simulation a envoyé à la fenêtre d'administration, déposé par le fil de lecture.
#[derive(Default)]
pub(crate) struct Inbox {
    pub(crate) connected: bool,
    /// Définitions courantes de la simulation (texte RON), à lire une fois.
    pub(crate) definitions: Option<String>,
    pub(crate) save_path: String,
    pub(crate) status: Option<Status>,
    /// Journal des demandes : (réussie, message), les plus récentes en dernier.
    pub(crate) log: Vec<(bool, String)>,
}
