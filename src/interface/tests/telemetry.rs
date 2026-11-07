use crate::definitions::Definitions;
use crate::ecosystem::FIXED_STEP;
use crate::interface::Telemetry;
use crate::persistence::Session;

fn session(steps: u64) -> Session {
    let mut session = Session::new(Definitions::builtin(), 42).unwrap();
    session.advance_steps(steps);
    session
}

#[test]
fn history_rebuilt_from_the_lineage_ends_on_the_live_counts() {
    let session = session(120_000);
    let simulation = session.simulation();
    let telemetry = Telemetry::from_lineage(simulation.lineage(), simulation.ticks());
    let last = telemetry.samples().last().unwrap();
    let summary = simulation.summary();
    assert_eq!(last.cells() as usize, summary.cells());
    assert_eq!(last.births() as u64, summary.births());
    assert_eq!(last.deaths() as u64, summary.deaths());
    assert!(last.energy().is_nan(), "l'énergie n'est pas conservée");
    assert_eq!(telemetry.samples().first().unwrap().cells(), 1.0);
    let end = simulation.ticks() as f64 * FIXED_STEP.as_secs_f64();
    assert!((last.time() as f64 - end).abs() < 0.01);
    assert!(
        telemetry
            .samples()
            .windows(2)
            .all(|pair| pair[0].time() <= pair[1].time() && pair[0].births() <= pair[1].births())
    );
}

#[test]
fn live_recording_keeps_the_whole_run_visible_within_a_bounded_history() {
    let mut session = session(0);
    let mut telemetry = Telemetry::new();
    telemetry.record(&session.simulation().summary());
    for _ in 0..2_000 {
        session.advance_steps(625);
        telemetry.record(&session.simulation().summary());
    }
    let samples = telemetry.samples();
    assert!(
        samples.len() < 480,
        "l'historique reste borné : {}",
        samples.len()
    );
    assert_eq!(samples[0].time(), 0.0);
    // Dix mille secondes simulées au total : le dernier point reste proche de la fin.
    assert!(samples.last().unwrap().time() > 19_000.0);
    assert!(
        samples
            .windows(2)
            .all(|pair| pair[0].time() < pair[1].time())
    );
    assert!(samples.iter().all(|sample| sample.energy().is_finite()));
}
