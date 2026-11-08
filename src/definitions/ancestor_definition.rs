use serde::{Deserialize, Serialize};

/// Génome de l'ancêtre : une suite ordonnée de séquences de 8 bits écrites en binaire.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AncestorDefinition {
    pub(crate) sequences: Vec<String>,
}

impl AncestorDefinition {
    pub(crate) fn bytes(&self) -> Result<Vec<u8>, String> {
        Self::parse(&self.sequences)
    }

    /// Décode des séquences écrites en texte binaire de huit chiffres.
    pub(crate) fn parse(sequences: &[String]) -> Result<Vec<u8>, String> {
        sequences
            .iter()
            .map(|text| {
                if text.len() == 8 && text.chars().all(|c| c == '0' || c == '1') {
                    Ok(u8::from_str_radix(text, 2).expect("huit bits binaires"))
                } else {
                    Err(format!("Séquence invalide : « {text} » (8 bits attendus)."))
                }
            })
            .collect()
    }
}
