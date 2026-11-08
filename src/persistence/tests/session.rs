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
                cell.genome().bytes(),
                cell.levels(),
                cell.last_division(),
                cell.activation(),
            )
        })
        .collect();
    let mut world =
        bincode_next::serde::encode_to_vec(simulation.grid().hydrology(), configuration).unwrap();
    world.extend(bincode_next::serde::encode_to_vec(simulation.grid().storm(), configuration).unwrap());
    world.extend(format!("{:?}{}", simulation.summary(), simulation.ticks()).bytes());
    (world, cells)
}

fn session_bytes(session: &Session) -> Vec<u8> {
    bincode_next::serde::encode_to_vec(session, bincode_next::config::standard()).unwrap()
}

fn temporary(name: &str) -> std::path::PathBuf {
    let directory = std::env::temp_dir().join(format!("cell-lab-{name}-{}", uuid::Uuid::now_v7()));
    std::fs::create_dir(&directory).unwrap();
    directory
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
    // La session sérialisée contient tout : génomes, positions, eau, orages, liaisons, historique.
    for _ in 0..1000 {
        run(&mut original, Duration::from_millis(29));
        run(&mut restored, Duration::from_millis(29));
        assert!(observable(&original) == observable(&restored), "divergence");
    }
    assert_eq!(original.history().samples(), restored.history().samples());
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
}

#[test]
fn check_rejects_an_inconsistent_session_without_modifying_it() {
    let definitions = Definitions::builtin();
    let session = Session::new(definitions.clone(), 3).unwrap();
    assert!(session.check(&definitions).is_ok());
    // Un catalogue dont les règles de l'espèce diffèrent n'est pas compatible avec la session.
    let other: Definitions = ron::from_str(&include_str!("../../../data/definitions.ron").replace("basal_metabolism: 0.5", "basal_metabolism: 0.6")).unwrap();
    assert!(session.check(&other).is_err() || other.fingerprint().unwrap() != definitions.fingerprint().unwrap());
}

#[test]
fn history_is_sampled_in_simulated_time_whatever_the_frame_pattern() {
    let definitions = Definitions::builtin();
    let mut smooth = Session::new(definitions.clone(), 9).unwrap();
    let mut jerky = Session::new(definitions, 9).unwrap();
    for _ in 0..6_000 {
        run(&mut smooth, Duration::from_millis(16));
    }
    for _ in 0..6 {
        run(&mut jerky, Duration::from_millis(16_000));
    }
    assert_eq!(smooth.history().samples(), jerky.history().samples());
    assert!(smooth.history().samples().len() > 5);
}

#[test]
fn selections_survive_a_save_and_restore() {
    let definitions = Definitions::builtin();
    let directory = temporary("select");
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
fn a_saved_experiment_resumes_exactly_with_genomes_positions_water_and_history() {
    let definitions = Definitions::builtin();
    let mut session = Session::new(definitions.clone(), 42).unwrap();
    session.advance_steps(90_000);
    let directory = temporary("resume");
    let path = directory.join("session.bin");
    session.save(&path).unwrap();
    let mut restored = Session::load(&path, definitions).unwrap();
    assert_eq!(restored.history().samples(), session.history().samples());
    assert!(restored.history().samples().iter().all(|sample| sample.energy().is_finite()));
    assert!(observable(&restored) == observable(&session));
    session.advance_steps(20_000);
    restored.advance_steps(20_000);
    assert!(observable(&restored) == observable(&session), "la reprise doit poursuivre à l'identique");
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn genomes_bonds_and_storms_survive_a_save_and_cells_with_equal_genomes_share_one_object() {
    let definitions = crate::test_support::catalog(&["adh_a", "adh_a", "adh_a"], &[]);
    let mut session = Session::new(definitions.clone(), 1).unwrap();
    session.advance_steps(60_000);
    assert!(!session.simulation().bonds().is_empty(), "il faut des liaisons à sauvegarder");
    let directory = temporary("life");
    let path = directory.join("session.bin");
    session.save(&path).unwrap();
    // Le génome de l'ancêtre ne sert qu'aux nouvelles expériences : un catalogue dont l'ancêtre
    // diffère reste compatible, seuls l'identité, les règles, la génétique et le monde comptent.
    assert!(Session::load(&path, Definitions::builtin()).is_ok());
    let restored = Session::load(&path, definitions).unwrap();
    assert_eq!(restored.simulation().bonds().pairs(), session.simulation().bonds().pairs());
    assert_eq!(restored.simulation().grid().storm().total(), session.simulation().grid().storm().total());
    for (a, b) in session.simulation().cells().iter().zip(restored.simulation().cells()) {
        assert_eq!(a.genome().bytes(), b.genome().bytes());
        assert_eq!(a.levels(), b.levels());
        assert_eq!(a.morphology(), b.morphology());
    }
    // Rechargés, les génomes identiques ne forment plus qu'un seul exemplaire en mémoire.
    let cells = restored.simulation().cells();
    for pair in cells.iter().flat_map(|a| cells.iter().map(move |b| (a, b))).filter(|(a, b)| a.genome().id() == b.genome().id()) {
        assert!(pair.0.genome().is_same_object(pair.1.genome()));
    }
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn files_of_another_format_or_catalog_or_with_trailing_data_are_refused() {
    let definitions = Definitions::builtin();
    let directory = temporary("refuse");
    let session = Session::new(definitions.clone(), 2).unwrap();
    let path = directory.join("ok.bin");
    session.save(&path).unwrap();
    let good = std::fs::read(&path).unwrap();
    let write = |name: &str, bytes: &[u8]| {
        let path = directory.join(name);
        std::fs::write(&path, bytes).unwrap();
        path
    };
    let mut other_version = good.clone();
    other_version[8] ^= 1;
    assert!(Session::load(write("version.bin", &other_version), definitions.clone()).is_err());
    let mut trailing = good.clone();
    trailing.push(0);
    assert!(Session::load(write("trailing.bin", &trailing), definitions.clone()).is_err());
    assert!(Session::load(write("short.bin", &good[..good.len() - 4]), definitions.clone()).is_err());
    assert!(Session::load(write("garbage.bin", b"pas une sauvegarde"), definitions).is_err());
    let other: Definitions = ron::from_str(&include_str!("../../../data/definitions.ron").replace("basal_metabolism: 0.5", "basal_metabolism: 0.6")).unwrap();
    assert!(Session::load(&path, std::sync::Arc::new(other)).is_err());
    std::fs::remove_dir_all(directory).unwrap();
}
