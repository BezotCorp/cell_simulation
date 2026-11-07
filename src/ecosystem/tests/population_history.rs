use crate::definitions::Definitions;
use crate::ecosystem::{FIXED_STEP, PopulationHistory, Simulation};

fn simulation(steps: u64) -> Simulation {
    let mut simulation = Simulation::from_definitions(Definitions::builtin(), 42).unwrap();
    run(&mut simulation, steps);
    simulation
}

#[test]
fn history_rebuilt_from_the_lineage_ends_on_the_live_counts() {
    let simulation = simulation(120_000);
    let history = PopulationHistory::from_lineage(simulation.lineage(), simulation.ticks());
    let last = history.samples().last().unwrap();
    let summary = simulation.summary();
    assert_eq!(last.cells() as usize, summary.cells());
    assert_eq!(last.births() as u64, summary.births());
    assert_eq!(last.deaths() as u64, summary.deaths());
    assert!(last.energy().is_nan(), "l'énergie n'est pas déductible de la filiation");
    assert_eq!(history.samples().first().unwrap().cells(), 1.0);
    let end = simulation.ticks() as f64 * FIXED_STEP.as_secs_f64();
    assert!((last.time() - end).abs() < 0.01);
    assert!(
        history
            .samples()
            .windows(2)
            .all(|pair| pair[0].time() <= pair[1].time() && pair[0].births() <= pair[1].births())
    );
    assert!(history.verify(end).is_ok());
}

#[test]
fn live_recording_keeps_the_whole_run_visible_within_a_bounded_history() {
    let mut simulation = simulation(0);
    let mut history = PopulationHistory::new();
    history.record(&simulation.summary());
    for _ in 0..2_000 {
        run(&mut simulation, 625);
        let elapsed = simulation.ticks() as f64 * FIXED_STEP.as_secs_f64();
        if history.is_due(elapsed) {
            history.record(&simulation.summary());
        }
    }
    let samples = history.samples();
    assert!(samples.len() < 480, "l'historique reste borné : {}", samples.len());
    assert_eq!(samples[0].time(), 0.0);
    // Vingt mille secondes simulées au total : le dernier point reste proche de la fin.
    assert!(samples.last().unwrap().time() > 19_000.0);
    assert!(samples.windows(2).all(|pair| pair[0].time() < pair[1].time()));
    assert!(samples.iter().all(|sample| sample.energy().is_finite()));
}

#[test]
fn a_corrupted_history_is_rejected() {
    let simulation = simulation(5_000);
    let elapsed = simulation.ticks() as f64 * FIXED_STEP.as_secs_f64();
    let mut history = PopulationHistory::from_lineage(simulation.lineage(), simulation.ticks());
    assert!(history.verify(elapsed).is_ok());
    // Un historique qui dépasse le temps de l'expérience est incohérent.
    assert!(history.verify(elapsed / 2.0).is_err());
    history.record(&simulation.summary());
    assert!(history.verify(elapsed).is_ok());
}

fn run(simulation: &mut Simulation, steps: u64) {
    for _ in 0..steps {
        simulation.step(crate::ecosystem::FIXED_STEP);
    }
}
