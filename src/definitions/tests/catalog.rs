use crate::definitions::Definitions;
const CATALOG: &str = include_str!("../../../data/definitions.ron");

#[test]
fn catalog_rejects_inconsistent_energy_budgets() {
    let catalog: Definitions =
        ron::from_str(&CATALOG.replace("energy_capacity: 100.0", "energy_capacity: 10.0")).unwrap();
    assert!(catalog.validate().is_err());
}
#[test]
fn catalog_requires_unique_version_seven_identities() {
    let duplicate: Definitions = ron::from_str(&CATALOG.replace(
        "019a59d8-0000-7000-8000-000000000002",
        "019a59d8-0000-7000-8000-000000000001",
    ))
    .unwrap();
    assert!(duplicate.validate().is_err());
    let nil: Definitions = ron::from_str(&CATALOG.replace(
        "019a59d8-0000-7000-8000-000000000001",
        "00000000-0000-0000-0000-000000000000",
    ))
    .unwrap();
    assert!(nil.validate().is_err());
}
#[test]
fn fingerprint_tracks_human_edits_and_is_stable_across_ron_roundtrip() {
    let catalog = Definitions::builtin();
    let text = ron::ser::to_string(&*catalog).unwrap();
    let decoded: Definitions = ron::from_str(&text).unwrap();
    assert_eq!(
        catalog.fingerprint().unwrap(),
        decoded.fingerprint().unwrap()
    );
    let changed: Definitions =
        ron::from_str(&CATALOG.replace("ambient_temperature: 20.0", "ambient_temperature: 21.0"))
            .unwrap();
    assert_ne!(
        catalog.fingerprint().unwrap(),
        changed.fingerprint().unwrap()
    );
}
#[test]
fn graphic_edits_keep_the_biological_fingerprint_but_not_the_full_one() {
    let catalog = Definitions::builtin();
    for (from, to) in [
        ("CELL LAB / Living systems", "Un autre titre"),
        ("cell_radius: 0.19", "cell_radius: 0.25"),
        ("color: (76, 226, 182, 255)", "color: (10, 20, 30, 255)"),
    ] {
        let styled: Definitions = ron::from_str(&CATALOG.replace(from, to)).unwrap();
        styled.validate().unwrap();
        assert_eq!(
            catalog.fingerprint().unwrap(),
            styled.fingerprint().unwrap(),
            "{from}"
        );
        assert_ne!(
            catalog.legacy_fingerprint().unwrap(),
            styled.legacy_fingerprint().unwrap()
        );
    }
    let species: Definitions =
        ron::from_str(&CATALOG.replace("basal_metabolism: 0.5", "basal_metabolism: 0.6")).unwrap();
    assert_ne!(
        catalog.fingerprint().unwrap(),
        species.fingerprint().unwrap()
    );
}

#[test]
fn incomplete_material_catalog_is_rejected() {
    let text = CATALOG
        .lines()
        .filter(|line| !line.trim_start().starts_with("Selection:"))
        .collect::<Vec<_>>()
        .join("\n");
    let catalog: Definitions = ron::from_str(&text).unwrap();
    assert!(catalog.validate().is_err());
}

#[test]
fn initial_genes_and_traits_do_not_alter_the_fingerprint_that_guards_restored_organisms() {
    let catalog = Definitions::builtin();
    for (from, to) in [
        ("neuron_complexity: 0", "neuron_complexity: 2"),
        ("mutation_rate: 0.1", "mutation_rate: 0.2"),
    ] {
        let edited: Definitions = ron::from_str(&CATALOG.replace(from, to)).unwrap();
        assert_eq!(catalog.fingerprint().unwrap(), edited.fingerprint().unwrap());
    }
    // Les sauvegardes antérieures aux mécanismes se reconnaissent à l'ancêtre qui avait un neurone.
    let neural: Definitions =
        ron::from_str(&CATALOG.replace("neuron_complexity: 0", "neuron_complexity: 1")).unwrap();
    assert_eq!(
        catalog.pre_traits_fingerprint().unwrap(),
        neural.pre_traits_fingerprint().unwrap()
    );
}
