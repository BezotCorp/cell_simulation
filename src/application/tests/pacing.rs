use crate::application::Pacing;
use crate::definitions::Definitions;
use crate::persistence::Session;
use std::time::Duration;

fn state(session: &Session) -> Vec<(usize, usize, Duration, u32)> {
    session
        .simulation()
        .cells()
        .iter()
        .map(|c| (c.x(), c.y(), c.age(), c.energy().to_bits()))
        .collect()
}

#[test]
fn capped_updates_return_the_deferred_time_instead_of_losing_it() {
    let definitions = Definitions::builtin();
    let mut capped = Session::new(definitions.clone(), 3).unwrap();
    let mut uncapped = Session::new(definitions, 3).unwrap();
    let (mut slow, mut fast) = (Pacing::new(1.0), Pacing::new(1.0));
    for _ in 0..4 {
        slow.advance_capped(&mut capped, Duration::from_millis(480), 5);
        fast.advance_capped(&mut uncapped, Duration::from_millis(480), u64::MAX);
    }
    assert_eq!(uncapped.simulation().ticks(), 120);
    assert!(capped.simulation().ticks() < 120, "le plafond doit limiter les pas");
    assert!(slow.debt() > Duration::ZERO);
    // Une fois la contrainte levée, la dette est entièrement exécutée : mêmes pas, même état,
    // même historique d'observation.
    while slow.debt() > Duration::ZERO {
        slow.advance_capped(&mut capped, Duration::ZERO, u64::MAX);
    }
    assert_eq!(capped.simulation().ticks(), uncapped.simulation().ticks());
    assert_eq!(capped.simulation().summary(), uncapped.simulation().summary());
    assert_eq!(state(&capped), state(&uncapped));
    assert_eq!(capped.history().samples(), uncapped.history().samples());
}

#[test]
fn debt_is_bounded_and_pause_clears_it() {
    let mut session = Session::new(Definitions::builtin(), 0).unwrap();
    let mut pacing = Pacing::new(1.0);
    for _ in 0..100 {
        pacing.advance_capped(&mut session, Duration::from_millis(100), 1);
    }
    // Quatre fois la demande d'une image au plus : une machine lente ralentit au lieu de s'endetter.
    assert!(pacing.debt() <= Duration::from_millis(400));
    let ticks = session.simulation().ticks();
    session.toggle_pause();
    pacing.advance_capped(&mut session, Duration::from_millis(100), u64::MAX);
    assert_eq!(pacing.debt(), Duration::ZERO);
    assert_eq!(session.simulation().ticks(), ticks, "une pause n'exécute rien");
}

#[test]
fn requested_speed_scales_the_simulated_time_not_the_step() {
    let mut session = Session::new(Definitions::builtin(), 0).unwrap();
    session.faster();
    session.faster();
    let mut pacing = Pacing::new(4.0);
    pacing.advance_capped(&mut session, Duration::from_millis(160), u64::MAX);
    // 160 ms à × 4 : 640 ms simulées, soit 40 pas fixes de 16 ms.
    assert_eq!(session.simulation().ticks(), 40);
}
