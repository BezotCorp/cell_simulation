use serde::Deserialize;
use std::collections::BTreeMap;

/// Contenu de `data/assets/library.ron` : pour chaque emplacement anatomique (« flagellum »,
/// « spine »…), les modèles GLB, relatifs au dossier de la bibliothèque, qui le représentent. Le
/// moteur choisit la variante d'après le développement du composant ; le génome n'en sait rien.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LibraryDefinition {
    #[serde(default)]
    pub(super) slots: BTreeMap<String, Vec<String>>,
}
