use crate::definitions::Definitions;

/// Définitions en cours d'édition dans la fenêtre d'administration, comparées à celles que
/// la simulation utilise réellement.
pub(crate) struct Draft {
    pub(crate) definitions: Definitions,
    /// Texte RON des définitions de la simulation à la dernière synchronisation.
    simulation: String,
}

impl Draft {
    /// Lit les définitions envoyées par la simulation.
    pub(crate) fn from_simulation(text: String) -> Result<Self, String> {
        let definitions: Definitions = ron::from_str(&text).map_err(|e| format!("Définitions illisibles : {e}"))?;
        Ok(Self { definitions, simulation: text })
    }

    pub(crate) fn text(&self) -> String {
        ron::ser::to_string(&self.definitions).unwrap_or_default()
    }

    /// Vrai si le brouillon diffère de ce que la simulation utilise.
    pub(crate) fn differs(&self) -> bool {
        self.text() != self.simulation
    }

    /// La simulation vient de confirmer ses définitions.
    pub(crate) fn synchronize(&mut self, text: String) {
        self.simulation = text;
    }

    /// Remplace tout le brouillon (par exemple une configuration enregistrée).
    pub(crate) fn replace(&mut self, definitions: Definitions) {
        self.definitions = definitions;
    }
}
