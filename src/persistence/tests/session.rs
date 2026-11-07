use crate::definitions::Definitions;
use crate::persistence::Session;
use std::time::Duration;

fn run(session: &mut Session, real: Duration) {
    session.advance(session.simulated_time(real), u64::MAX);
}

/// État observable sans les identités : les nouvelles filles reçoivent des UUID v7 horodatés, qui
/// diffèrent d'une exécution à l'autre sans rien changer à la biologie.
fn observable(session: &Session) -> (Vec<u8>, Vec<String>) {
    let configuration = bincode_next::config::standard();
    let simulation = session.simulation();
    let cells = simulation
        .cells()
        .iter()
        .map(|cell| {
            format!(
                "{:?} {:?} {} {:?} {:?} {:?} {}",
                cell.position(),
                cell.age(),
                cell.energy(),
                cell.genome(),
                cell.levels(),
                cell.last_division(),
                cell.activation(),
            )
        })
        .collect();
    let mut water =
        bincode_next::serde::encode_to_vec(simulation.grid().hydrology(), configuration).unwrap();
    water.extend(
        bincode_next::serde::encode_to_vec(simulation.grid().storm(), configuration).unwrap(),
    );
    water.extend(format!("{:?}{}", simulation.summary(), simulation.ticks()).bytes());
    (water, cells)
}

fn session_bytes(session: &Session) -> Vec<u8> {
    bincode_next::serde::encode_to_vec(session, bincode_next::config::standard()).unwrap()
}

#[test]
fn serialized_session_restores_random_state_and_fractional_clock() {
    let definitions = Definitions::builtin();
    let mut original = Session::new(definitions.clone(), 42).unwrap();
    original.advance_steps(2500);
    run(&mut original, Duration::from_millis(7));
    let (mut restored, _): (Session, usize) =
        bincode_next::serde::decode_from_slice(&session_bytes(&original), bincode_next::config::standard())
            .unwrap();
    restored.validate(definitions).unwrap();
    restored.restore_positions(&original.positions()).unwrap();
    restored.restore_hydrology(original.simulation().grid().hydrology().clone()).unwrap();
    restored
        .restore_life(original.life(), original.simulation().grid().storm().clone())
        .unwrap();
    // Includes food spreading, stochastic movement and inherited neural mutations.
    for _ in 0..1000 {
        run(&mut original, Duration::from_millis(29));
        run(&mut restored, Duration::from_millis(29));
        assert!(observable(&original) == observable(&restored), "divergence");
    }
}

#[test]
fn pause_preserves_pending_fraction_without_catching_up() {
    let mut session = Session::new(Definitions::builtin(), 0).unwrap();
    run(&mut session, Duration::from_millis(7));
    session.toggle_pause();
    run(&mut session, Duration::from_secs(60));
    assert_eq!(session.simulation().ticks(), 0);
    session.toggle_pause();
    run(&mut session, Duration::from_millis(9));
    assert_eq!(session.simulation().ticks(), 1);
}

#[test]
fn explicit_step_pauses_and_advances_exactly_once() {
    let mut session = Session::new(Definitions::builtin(), 0).unwrap();
    session.step_once();
    assert!(session.view().paused());
    assert_eq!(session.simulation().ticks(), 1);
    run(&mut session, Duration::from_secs(1));
    assert_eq!(session.simulation().ticks(), 1);
}

#[test]
fn check_rejects_an_inconsistent_session_without_modifying_it() {
    let session = Session::new(Definitions::builtin(), 5).unwrap();
    let text = include_str!("../../../data/definitions.ron").replace("size: 10", "size: 11");
    let other: Definitions = ron::from_str(&text).unwrap();
    assert!(session.check(&other).is_err());
    assert!(session.check(&Definitions::builtin()).is_ok());
}

#[test]
fn saved_history_survives_a_restore_and_old_saves_rebuild_theirs_from_the_lineage() {
    let definitions = Definitions::builtin();
    let mut session = Session::new(definitions.clone(), 42).unwrap();
    session.advance_steps(90_000);
    let directory = std::env::temp_dir().join(format!("cell-lab-history-{}", uuid::Uuid::now_v7()));
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("session.bin");
    session.save(&path).unwrap();
    let restored = Session::load(&path, definitions.clone()).unwrap();
    assert_eq!(restored.history().samples(), session.history().samples());
    // L'énergie moyenne fait partie de l'historique sauvegardé.
    assert!(
        restored
            .history()
            .samples()
            .iter()
            .all(|sample| sample.energy().is_finite())
    );
    // Sans section d'historique (versions 1 et 2), les effectifs sont déduits de la filiation.
    session.rebuild_history();
    let last = session.history().samples().last().unwrap();
    assert_eq!(last.cells() as usize, session.simulation().cells().len());
    assert!(last.energy().is_nan());
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn history_is_sampled_in_simulated_time_whatever_the_frame_pattern() {
    let definitions = Definitions::builtin();
    let mut smooth = Session::new(definitions.clone(), 9).unwrap();
    let mut jerky = Session::new(definitions, 9).unwrap();
    for _ in 0..600 {
        run(&mut smooth, Duration::from_millis(16));
    }
    for _ in 0..6 {
        run(&mut jerky, Duration::from_millis(1600));
    }
    assert_eq!(smooth.history().samples(), jerky.history().samples());
    assert!(smooth.history().samples().len() > 5);
}

#[test]
fn selections_survive_a_save_and_restore() {
    let definitions = Definitions::builtin();
    let directory = std::env::temp_dir().join(format!("cell-lab-select-{}", uuid::Uuid::now_v7()));
    std::fs::create_dir(&directory).unwrap();
    let mut session = Session::new(definitions.clone(), 5).unwrap();
    session.advance_steps(8_000);
    let habitat = session.simulation().grid().habitat_id(3, 4);
    session.select_habitat(habitat);
    session.save(directory.join("habitat.bin")).unwrap();
    let restored = Session::load(directory.join("habitat.bin"), definitions.clone()).unwrap();
    assert_eq!(restored.view().habitat(), Some(habitat));
    let organism = session.simulation().cells()[0].id();
    session.select_organism(organism);
    session.save(directory.join("organism.bin")).unwrap();
    let restored = Session::load(directory.join("organism.bin"), definitions).unwrap();
    assert_eq!(restored.view().selected(), Some(organism));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn saved_fine_positions_survive_and_a_save_without_them_is_still_loadable() {
    let definitions = Definitions::builtin();
    let mut session = Session::new(definitions.clone(), 42).unwrap();
    session.advance_steps(60_000);
    assert!(session.simulation().cells().len() > 5);
    let directory = std::env::temp_dir().join(format!("cell-lab-positions-{}", uuid::Uuid::now_v7()));
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("session.bin");
    session.save(&path).unwrap();
    let mut restored = Session::load(&path, definitions.clone()).unwrap();
    assert_eq!(restored.positions(), session.positions());
    // La reprise poursuit exactement comme l'original, déplacements libres compris.
    session.advance_steps(5_000);
    restored.advance_steps(5_000);
    // Les identités des nouvelles filles viennent de l'horloge : seules les positions se comparent.
    let offsets = |session: &Session| {
        session
            .positions()
            .into_iter()
            .map(|(_, east, south)| (east, south))
            .collect::<Vec<_>>()
    };
    assert_eq!(offsets(&restored), offsets(&session));
    assert_eq!(restored.simulation().summary(), session.simulation().summary());

    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn saved_water_state_survives_and_old_saves_rebuild_it_from_their_water_level() {
    let definitions = Definitions::builtin();
    let mut session = Session::new(definitions.clone(), 42).unwrap();
    session.advance_steps(90_000);
    let directory = std::env::temp_dir().join(format!("cell-lab-water-{}", uuid::Uuid::now_v7()));
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("session.bin");
    session.save(&path).unwrap();
    let mut restored = Session::load(&path, definitions.clone()).unwrap();
    let water = |session: &Session| session.simulation().grid().hydrology().summary();
    // L'intensité de pluie est dérivée du pas précédent : elle n'est pas sauvegardée.
    assert_eq!(water(&restored).water(), water(&session).water());
    assert_eq!(water(&restored).solids(), water(&session).solids());
    // La reprise poursuit exactement : météo, relief et courants compris.
    session.advance_steps(20_000);
    restored.advance_steps(20_000);
    assert_eq!(water(&restored), water(&session));

    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn genomes_bonds_and_storms_survive_a_save_and_the_catalog_ancestor_does_not_matter_to_a_save() {
    let text = include_str!("../../../data/definitions.ron")
        .replace("{Digestive: 1.0}", "{Digestive: 1.0, Adhesin: 1.0}");
    let definitions: std::sync::Arc<Definitions> = std::sync::Arc::new(ron::from_str(&text).unwrap());
    let mut session = Session::new(definitions.clone(), 1).unwrap();
    session.advance_steps(60_000);
    assert!(!session.simulation().bonds().is_empty(), "il faut des liaisons à sauvegarder");
    let directory = std::env::temp_dir().join(format!("cell-lab-life-{}", uuid::Uuid::now_v7()));
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("session.bin");
    session.save(&path).unwrap();
    // Le génome de l'ancêtre ne sert qu'aux nouvelles expériences : un catalogue dont l'ancêtre
    // diffère reste compatible, seuls l'identité, les règles de l'espèce et le monde comptent.
    assert!(Session::load(&path, Definitions::builtin()).is_ok());
    let restored = Session::load(&path, definitions).unwrap();
    assert_eq!(restored.life(), session.life());
    assert_eq!(restored.simulation().grid().storm().total(), session.simulation().grid().storm().total());
    for (a, b) in session.simulation().cells().iter().zip(restored.simulation().cells()) {
        assert_eq!(a.genome(), b.genome());
        assert_eq!(a.levels(), b.levels());
        assert_eq!(a.morphology(), b.morphology());
    }
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn organisms_of_an_old_file_keep_their_phenotype_when_translated_into_genomes() {
    let fixture = |name: &str| std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name);
    // Version 3 : aucun mécanisme n'existait, les organismes ingéraient et nageaient comme l'ancêtre.
    let old = Session::load(fixture("legacy-v3.bin"), Definitions::builtin()).unwrap();
    assert!(!old.simulation().cells().is_empty());
    for cell in old.simulation().cells() {
        let physiology = cell.physiology();
        assert!((physiology.ingestion() - 1.0).abs() < 0.01);
        assert!((physiology.motility() - 1.0).abs() < 0.01);
        assert_eq!(physiology.adhesion(), 0.0);
        assert_eq!(physiology.photosynthesis(), 0.0);
        assert!((cell.capacity() - 100.0).abs() < 0.01);
        assert!(cell.energy() > 0.0 && cell.energy() <= cell.capacity());
        assert!((0.0..=1.0).contains(&physiology.water_adaptation()));
        assert!((0.5..=2.0).contains(&physiology.metabolism_rate()));
        assert!((30.0..=100.0).contains(&cell.division_threshold()), "{}", cell.division_threshold());
    }
    // Version 6 : mécanismes et liaisons du fichier sont traduits sans perte de structure.
    let recent = Session::load(fixture("legacy-v6.bin"), Definitions::builtin()).unwrap();
    assert!(!recent.simulation().bonds().is_empty(), "les liaisons du fichier sont conservées");
    assert!(recent.simulation().cells().iter().any(|cell| cell.adhesion() > 0.9));
    assert!(recent.simulation().cells().iter().all(|cell| cell.genome().validate().is_ok()));
}
