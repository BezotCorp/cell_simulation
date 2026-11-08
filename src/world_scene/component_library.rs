use crate::world_scene::library_definition::LibraryDefinition;
use fyrox::{
    asset::manager::ResourceManager,
    resource::model::{Model, ModelResource},
};
use std::{collections::HashMap, path::Path};

/// Bibliothèque de composants modulaires (modèles GLB exportés de Blender). Un emplacement sans
/// modèle disponible est dessiné par une forme procédurale : la scène ne dépend jamais de la
/// présence des assets, et la biologie ne les connaît pas. `Default` donne la bibliothèque vide.
#[derive(Clone, Default)]
pub(crate) struct ComponentLibrary {
    slots: HashMap<String, Vec<ModelResource>>,
    manager: Option<ResourceManager>,
    directory: std::path::PathBuf,
    files: std::sync::Arc<std::sync::Mutex<HashMap<String, ModelResource>>>,
}

impl ComponentLibrary {
    /// Lit `library.ron` dans `directory` et demande au gestionnaire de ressources les modèles qu'il
    /// cite. Une bibliothèque absente ou invalide laisse simplement les formes procédurales ; le
    /// chargement des modèles se poursuit en tâche de fond.
    pub(crate) fn load(manager: &ResourceManager, directory: &Path) -> Self {
        let Ok(text) = std::fs::read_to_string(directory.join("library.ron")) else {
            return Self::with_directory(manager, directory);
        };
        let definition: LibraryDefinition = match ron::from_str(&text) {
            Ok(definition) => definition,
            Err(error) => {
                eprintln!("Cell Lab : bibliothèque d'assets ignorée ({error}).");
                return Self::default();
            }
        };
        let slots = definition
            .slots
            .into_iter()
            .map(|(slot, files)| {
                let models = files
                    .iter()
                    .map(|file| manager.request::<Model>(directory.join(file)))
                    .collect();
                (slot, models)
            })
            .collect();
        Self { slots, manager: Some(manager.clone()), directory: directory.to_path_buf(), files: Default::default() }
    }

    /// Bibliothèque sans emplacements, mais qui sait charger un fichier GLB de `directory` à la demande.
    fn with_directory(manager: &ResourceManager, directory: &Path) -> Self {
        Self {
            manager: Some(manager.clone()),
            directory: directory.to_path_buf(),
            ..Self::default()
        }
    }

    /// Modèle du fichier `file` (relatif au dossier de la bibliothèque), demandé au premier usage ;
    /// `None` tant qu'il n'est pas chargé ou s'il est invalide.
    pub(crate) fn file(&self, file: &str) -> Option<ModelResource> {
        let manager = self.manager.as_ref()?;
        let mut files = self.files.lock().ok()?;
        let model = files
            .entry(file.to_owned())
            .or_insert_with(|| manager.request::<Model>(self.directory.join(file)));
        model.is_ok().then(|| model.clone())
    }

    /// Modèle de l'emplacement `slot` pour la variante `variant`, s'il est chargé.
    pub(crate) fn model(&self, slot: &str, variant: u8) -> Option<&ModelResource> {
        let models = self.slots.get(slot)?;
        let model = models.get(usize::from(variant) % models.len().max(1))?;
        model.is_ok().then_some(model)
    }
}
