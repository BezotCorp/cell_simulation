use crate::ecosystem::{FIXED_STEP, SimulationClock};
use std::time::Duration;

fn simulated_time(frames: &[u64]) -> Duration {
    let mut clock = SimulationClock::new();
    let mut total = Duration::ZERO;
    for millis in frames {
        clock.advance(Duration::from_millis(*millis), true, u64::MAX, |dt| {
            total += dt
        });
    }
    total
}

#[test]
fn rendering_frequency_does_not_change_simulated_time() {
    assert_eq!(simulated_time(&[10; 100]), simulated_time(&[100; 10]));
    assert_eq!(simulated_time(&[1000]), Duration::from_millis(992));
}

#[test]
fn pause_does_not_advance_or_catch_up_and_preserves_remainder() {
    let mut clock = SimulationClock::new();
    let mut total = Duration::ZERO;
    clock.advance(Duration::from_millis(10), true, u64::MAX, |dt| total += dt);
    let paused = clock.advance(Duration::from_secs(60), false, u64::MAX, |_| {
        panic!("paused")
    });
    assert_eq!(paused.unused, Duration::ZERO);
    clock.advance(Duration::from_millis(6), true, u64::MAX, |dt| total += dt);
    assert_eq!(total, Duration::from_millis(16));
}

#[test]
fn step_cap_returns_the_unexecuted_time_instead_of_losing_it() {
    let mut clock = SimulationClock::new();
    let mut executed = 0;
    // 1 s + 5 ms demandés, 10 pas autorisés : 62 pas dus.
    let first = clock.advance(Duration::from_millis(1005), true, 10, |_| executed += 1);
    assert_eq!((first.steps, executed), (10, 10));
    assert_eq!(first.unused, FIXED_STEP * 52);
    assert!(clock.validate().is_ok());
    // Le temps rendu, présenté à nouveau, est intégralement exécuté sans écart ni doublon.
    let second = clock.advance(first.unused, true, u64::MAX, |_| executed += 1);
    assert_eq!((second.steps, second.unused), (52, Duration::ZERO));
    assert_eq!(executed, 62);
    // Le reste de 13 ms (< un pas) est conservé par l'accumulateur.
    let rest = clock.advance(Duration::from_millis(3), true, u64::MAX, |_| executed += 1);
    assert_eq!((rest.steps, executed), (1, 63));
}
