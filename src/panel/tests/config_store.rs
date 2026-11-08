use crate::definitions::Definitions;
use crate::panel::ConfigStore;

fn directory(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("cell_lab_configs_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    path
}

#[test]
fn a_configuration_is_saved_listed_loaded_and_deleted_without_hand_editing() {
    let store = ConfigStore::new(directory("cycle"));
    assert!(store.names().is_empty());
    let mut definitions = (*Definitions::builtin()).clone();
    definitions.species_mut().parts_mut().truncate(3);
    store.save("Ma première expérience", &definitions).unwrap();
    store.save("autre", &Definitions::builtin()).unwrap();
    assert_eq!(store.names(), vec!["Ma première expérience".to_owned(), "autre".to_owned()]);
    let loaded = store.load("Ma première expérience").unwrap();
    assert_eq!(loaded.species().parts().len(), 3);
    assert_eq!(ron::ser::to_string(&loaded).unwrap(), ron::ser::to_string(&definitions).unwrap());
    store.delete("autre").unwrap();
    assert_eq!(store.names().len(), 1);
    let _ = std::fs::remove_dir_all(store.directory());
}

#[test]
fn names_cannot_escape_the_directory_and_corrupt_files_are_refused() {
    let store = ConfigStore::new(directory("refus"));
    for name in ["", "  ", "../dehors", "a/b", "x.ron"] {
        assert!(store.save(name, &Definitions::builtin()).is_err(), "{name}");
    }
    std::fs::create_dir_all(store.directory()).unwrap();
    std::fs::write(store.directory().join("cassée.ron"), "(pas du ron").unwrap();
    assert!(store.load("cassée").is_err());
    let _ = std::fs::remove_dir_all(store.directory());
}
