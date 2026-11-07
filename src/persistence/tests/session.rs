use crate::definitions::Definitions;
use crate::persistence::Session;
use std::time::Duration;

fn bytes(session: &Session) -> Vec<u8> {
    bincode_next::serde::encode_to_vec(session, bincode_next::config::standard()).unwrap()
}

#[test]
fn serialized_session_restores_random_state_and_fractional_clock() {
    let definitions = Definitions::builtin();
    let mut original = Session::new(definitions.clone(), 42).unwrap();
    original.advance_steps(2500);
    original.update(Duration::from_millis(7), u64::MAX);
    let (mut restored, _): (Session, usize) =
        bincode_next::serde::decode_from_slice(&bytes(&original), bincode_next::config::standard())
            .unwrap();
    restored.validate(definitions).unwrap();
    // Includes food spreading, stochastic movement and inherited neural mutations.
    for _ in 0..1000 {
        original.update(Duration::from_millis(29), u64::MAX);
        restored.update(Duration::from_millis(29), u64::MAX);
        assert_eq!(bytes(&original), bytes(&restored));
    }
}

#[test]
fn pause_preserves_pending_fraction_without_catching_up() {
    let mut session = Session::new(Definitions::builtin(), 0).unwrap();
    session.update(Duration::from_millis(7), u64::MAX);
    session.toggle_pause();
    session.update(Duration::from_secs(60), u64::MAX);
    assert_eq!(session.simulation().ticks(), 0);
    session.toggle_pause();
    session.update(Duration::from_millis(9), u64::MAX);
    assert_eq!(session.simulation().ticks(), 1);
}

#[test]
fn explicit_step_pauses_and_advances_exactly_once() {
    let mut session = Session::new(Definitions::builtin(), 0).unwrap();
    session.step_once();
    assert!(session.view().paused());
    assert_eq!(session.simulation().ticks(), 1);
    session.update(Duration::from_secs(1), u64::MAX);
    assert_eq!(session.simulation().ticks(), 1);
}

#[test]
fn capped_updates_return_the_deferred_time_instead_of_losing_it() {
    let definitions = Definitions::builtin();
    let mut capped = Session::new(definitions.clone(), 3).unwrap();
    let mut uncapped = Session::new(definitions, 3).unwrap();
    for _ in 0..4 {
        capped.update(Duration::from_millis(480), 5);
        uncapped.update(Duration::from_millis(480), u64::MAX);
    }
    assert_eq!(uncapped.simulation().ticks(), 120);
    assert!(
        capped.simulation().ticks() < 120,
        "le plafond doit limiter les pas"
    );
    assert!(capped.backlog() > Duration::ZERO);
    // Une fois la contrainte levée, le temps différé est entièrement exécuté : mêmes pas, même état.
    while capped.backlog() > Duration::ZERO {
        capped.update(Duration::ZERO, u64::MAX);
    }
    assert_eq!(capped.simulation().ticks(), uncapped.simulation().ticks());
    // Les identités persistantes sont tirées à la création : on compare l'état biologique.
    assert_eq!(
        capped.simulation().summary(),
        uncapped.simulation().summary()
    );
    let state = |session: &Session| {
        session
            .simulation()
            .cells()
            .iter()
            .map(|c| (c.x(), c.y(), c.age(), c.energy().to_bits()))
            .collect::<Vec<_>>()
    };
    assert_eq!(state(&capped), state(&uncapped));
}

#[test]
fn backlog_is_bounded_and_pause_clears_it() {
    let mut session = Session::new(Definitions::builtin(), 0).unwrap();
    for _ in 0..100 {
        session.update(Duration::from_millis(100), 1);
    }
    // Quatre fois la demande d'une image au plus : une machine lente ralentit au lieu de s'endetter.
    assert!(session.backlog() <= Duration::from_millis(400));
    session.toggle_pause();
    session.update(Duration::from_millis(100), 1);
    assert_eq!(session.backlog(), Duration::ZERO);
}

#[test]
fn check_rejects_an_inconsistent_session_without_modifying_it() {
    let session = Session::new(Definitions::builtin(), 5).unwrap();
    let text = include_str!("../../../data/definitions.ron").replace("size: 10", "size: 11");
    let other: Definitions = ron::from_str(&text).unwrap();
    assert!(session.check(&other).is_err());
    assert!(session.check(&Definitions::builtin()).is_ok());
}
