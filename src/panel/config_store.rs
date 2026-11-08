use crate::definitions::Definitions;
use std::path::{Path, PathBuf};

/// Configurations nommées enregistrées par l'utilisateur dans un dossier : chacune est un
/// catalogue complet (capacités, rendus, cellules initiales, monde) au format RON.
pub(crate) struct ConfigStore {
    directory: PathBuf,
}

impl ConfigStore {
    pub(crate) fn new(directory: impl Into<PathBuf>) -> Self {
        Self { directory: directory.into() }
    }

    pub(crate) fn directory(&self) -> &Path {
        &self.directory
    }

    fn path(&self, name: &str) -> Result<PathBuf, String> {
        let name = name.trim();
        if name.is_empty() || name.chars().any(|c| !(c.is_alphanumeric() || " -_".contains(c))) {
            return Err("Nom de configuration invalide (lettres, chiffres, espaces, - et _).".into());
        }
        Ok(self.directory.join(format!("{name}.ron")))
    }

    /// Noms des configurations enregistrées, par ordre alphabétique.
    pub(crate) fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(&self.directory)
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|entry| {
                let path = entry.path();
                (path.extension()? == "ron").then(|| path.file_stem()?.to_str().map(str::to_owned))?
            })
            .collect();
        names.sort();
        names
    }

    pub(crate) fn save(&self, name: &str, definitions: &Definitions) -> Result<(), String> {
        let path = self.path(name)?;
        std::fs::create_dir_all(&self.directory).map_err(|e| e.to_string())?;
        let text = ron::ser::to_string_pretty(definitions, ron::ser::PrettyConfig::default().struct_names(false))
            .map_err(|e| e.to_string())?;
        std::fs::write(&path, text).map_err(|e| format!("{} : {e}", path.display()))
    }

    pub(crate) fn load(&self, name: &str) -> Result<Definitions, String> {
        let path = self.path(name)?;
        let text = std::fs::read_to_string(&path).map_err(|e| format!("{} : {e}", path.display()))?;
        let definitions: Definitions = ron::from_str(&text).map_err(|e| format!("{} : {e}", path.display()))?;
        definitions.validate()?;
        Ok(definitions)
    }

    pub(crate) fn delete(&self, name: &str) -> Result<(), String> {
        std::fs::remove_file(self.path(name)?).map_err(|e| e.to_string())
    }
}
