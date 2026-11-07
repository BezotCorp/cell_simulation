use crate::definitions::Definitions;
use crate::persistence::Session;
use std::time::Duration;

fn run(session: &mut Session, real: Duration) {
    session.advance(session.simulated_time(real), u64::MAX);
}

fn bytes(session: &Session) -> Vec<u8> {
    let configuration = bincode_next::config::standard();
    let mut bytes = bincode_next::serde::encode_to_vec(session, configuration).unwrap();
    // Les positions fines vivent dans leur propre section du fichier BIN.
    bytes.extend(bincode_next::serde::encode_to_vec(session.positions(), configuration).unwrap());
    bytes.extend(
        bincode_next::serde::encode_to_vec(session.simulation().grid().hydrology(), configuration)
            .unwrap(),
    );
    bytes
}

#[test]
fn serialized_session_restores_random_state_and_fractional_clock() {
    let definitions = Definitions::builtin();
    let mut original = Session::new(definitions.clone(), 42).unwrap();
    original.advance_steps(2500);
    run(&mut original, Duration::from_millis(7));
    let (mut restored, _): (Session, usize) =
        bincode_next::serde::decode_from_slice(&bytes(&original), bincode_next::config::standard())
            .unwrap();
    restored.validate(definitions).unwrap();
    restored.restore_positions(&original.positions()).unwrap();
    restored.restore_hydrology(original.simulation().grid().hydrology().clone()).unwrap();
    // Includes food spreading, stochastic movement and inherited neural mutations.
    for _ in 0..1000 {
        run(&mut original, Duration::from_millis(29));
        run(&mut restored, Duration::from_millis(29));
        assert_eq!(bytes(&original), bytes(&restored));
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

    // Une sauvegarde de version 3 s'arrête avant la section des positions.
    let mut old = std::fs::read(&path).unwrap();
    old[8..12].copy_from_slice(&3u32.to_le_bytes());
    let payload = u64::from_le_bytes(old[20..28].try_into().unwrap()) as usize;
    let history_at = 28 + payload;
    let history = u64::from_le_bytes(old[history_at..history_at + 8].try_into().unwrap()) as usize;
    old.truncate(history_at + 8 + history);
    let legacy = directory.join("legacy.bin");
    std::fs::write(&legacy, old).unwrap();
    let loaded = Session::load(&legacy, definitions).unwrap();
    assert_eq!(loaded.simulation().cells().len(), 
        Session::load(&path, Definitions::builtin()).unwrap().simulation().cells().len());
    for (_, east, south) in loaded.positions() {
        assert!((0.15..=0.85).contains(&east) && (0.15..=0.85).contains(&south));
    }
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

    // Une sauvegarde de version 4 s'arrête avant la section de l'eau.
    let mut old = std::fs::read(&path).unwrap();
    old[8..12].copy_from_slice(&4u32.to_le_bytes());
    let mut at = 28 + u64::from_le_bytes(old[20..28].try_into().unwrap()) as usize;
    for _ in 0..2 {
        at += 8 + u64::from_le_bytes(old[at..at + 8].try_into().unwrap()) as usize;
    }
    let tail = u64::from_le_bytes(old[at..at + 8].try_into().unwrap()) as usize;
    old.truncate(at);
    assert!(tail > 0);
    let legacy = directory.join("legacy.bin");
    std::fs::write(&legacy, old).unwrap();
    let loaded = Session::load(&legacy, definitions).unwrap();
    let summary = water(&loaded);
    assert!(summary.flooded() > 0.0 && summary.flooded() <= 1.0);
    assert!(loaded.simulation().grid().has_hydrology());
    std::fs::remove_dir_all(directory).unwrap();
}
