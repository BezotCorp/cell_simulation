use crate::habitat::TemperatureMap;
use crate::hydrology::{GroundKind, Hydrology, INITIAL_FILL};

fn world(size: usize, seed: u64) -> (Hydrology, TemperatureMap) {
    let ground = (0..size * size)
        .map(|i| match (i * 7 + seed as usize) % 11 {
            0 => GroundKind::Bedrock,
            1..=3 => GroundKind::Sand,
            4..=5 => GroundKind::Soil,
            _ => GroundKind::Bare,
        })
        .collect();
    (
        Hydrology::new(size, seed, ground, INITIAL_FILL, &[]),
        TemperatureMap::new(size, 20.0),
    )
}

fn run(hydrology: &mut Hydrology, temperature: &TemperatureMap, ticks: usize) {
    for _ in 0..ticks {
        hydrology.step(temperature, 1.0);
    }
}

#[test]
fn a_closed_world_conserves_its_water_and_its_solid_matter() {
    for size in [10, 32] {
        let (mut hydrology, temperature) = world(size, 3);
        let before = hydrology.summary();
        run(&mut hydrology, &temperature, 3_000);
        let after = hydrology.summary();
        assert!(
            (after.water() - before.water()).abs() < 1e-3 * before.water().max(1.0),
            "eau {} -> {}",
            before.water(),
            after.water()
        );
        assert!(
            (after.solids() - before.solids()).abs() < 1e-3 * before.solids().max(1.0),
            "matière {} -> {}",
            before.solids(),
            after.solids()
        );
    }
}

#[test]
fn the_same_seed_replays_the_same_weather_and_terrain() {
    let (mut a, temperature) = world(16, 5);
    let (mut b, _) = world(16, 5);
    run(&mut a, &temperature, 800);
    run(&mut b, &temperature, 800);
    for y in 0..16 {
        for x in 0..16 {
            assert_eq!(a.elevation(x, y).to_bits(), b.elevation(x, y).to_bits());
            assert_eq!(a.depth(x, y).to_bits(), b.depth(x, y).to_bits());
            assert_eq!(a.cloud(x, y).to_bits(), b.cloud(x, y).to_bits());
        }
    }
    let (mut other, _) = world(16, 6);
    run(&mut other, &temperature, 800);
    assert!(a.summary() != other.summary());
}

#[test]
fn weather_is_regional_and_the_world_keeps_both_wet_and_dry_places() {
    let (mut hydrology, mut temperature) = world(24, 11);
    // Une source de chaleur évapore beaucoup : sa vapeur condense plus loin, sous le vent.
    temperature.apply_heat_source(6, 6, 8.0);
    let (mut rainy_ticks, mut partial_rain, mut flooded_min, mut flooded_max) = (0, 0, 1.0_f32, 0.0_f32);
    for tick in 0..4_000 {
        hydrology.step(&temperature, 1.0);
        let summary = hydrology.summary();
        if tick > 500 {
            flooded_min = flooded_min.min(summary.flooded());
            flooded_max = flooded_max.max(summary.flooded());
            if summary.raining() > 0.0 {
                rainy_ticks += 1;
                if summary.raining() < 0.9 {
                    partial_rain += 1;
                }
            }
        }
        if tick % 500 == 0 {
            eprintln!(
                "t={tick} inondé={:.2} pluie={:.3} nuages={:.2} profondeur={:.3} courant={:.2}",
                summary.flooded(),
                summary.raining(),
                summary.cloud_cover(),
                summary.mean_depth(),
                summary.max_speed()
            );
            let peak = (0..24 * 24).map(|i| hydrology.rain(i % 24, i / 24)).fold(0.0_f32, f32::max);
            eprintln!("   pluie maximale par seconde {peak:.4}");
        }
    }
    assert!(rainy_ticks > 100, "il doit pleuvoir : {rainy_ticks}");
    assert!(partial_rain * 10 >= rainy_ticks * 9, "la pluie doit rester locale");
    assert!(flooded_max < 0.99 && flooded_min > 0.01, "{flooded_min} {flooded_max}");
}

#[test]
fn flowing_water_reshapes_the_terrain_and_the_terrain_redirects_the_water() {
    let (mut hydrology, temperature) = world(24, 2);
    let before: Vec<f32> = (0..24 * 24).map(|i| hydrology.elevation(i % 24, i / 24)).collect();
    run(&mut hydrology, &temperature, 6_000);
    let changed = (0..24 * 24)
        .filter(|i| (hydrology.elevation(i % 24, i / 24) - before[*i]).abs() > 1e-3)
        .count();
    let largest = (0..24 * 24)
        .map(|i| (hydrology.elevation(i % 24, i / 24) - before[i]).abs())
        .fold(0.0_f32, f32::max);
    eprintln!("sites modifiés {changed}, plus forte variation {largest:.3}");
    assert!(changed > 10, "l'érosion doit modifier le relief");
    assert!(largest < 0.5, "sans effondrement du relief");
}

#[test]
fn water_poured_on_a_summit_runs_downhill_instead_of_staying_put() {
    let dry = Hydrology::new(8, 9, vec![GroundKind::Bare; 64], 0.0, &[]);
    let (mut hx, mut hy) = (0, 0);
    for i in 0..64 {
        if dry.elevation(i % 8, i / 8) > dry.elevation(hx, hy) {
            (hx, hy) = (i % 8, i / 8);
        }
    }
    let mut puddles = vec![0.0; 64];
    puddles[hy * 8 + hx] = 1.0;
    // Le monde se règle sur 60 écoulements dès sa création : la flaque a déjà quitté le sommet.
    let poured = Hydrology::new(8, 9, vec![GroundKind::Bare; 64], 0.0, &puddles);
    assert!(poured.depth(hx, hy) < 0.5, "{}", poured.depth(hx, hy));
    let total: f32 = (0..64).map(|i| poured.depth(i % 8, i / 8)).sum();
    assert!((total - 1.0).abs() < 1e-3);
    let lowest = (0..64)
        .min_by(|a, b| poured.elevation(a % 8, a / 8).total_cmp(&poured.elevation(b % 8, b / 8)))
        .unwrap();
    assert!(poured.depth(lowest % 8, lowest / 8) > poured.depth(hx, hy));
}

#[test]
#[ignore = "mesure de coût, lancée à la main"]
fn cost_of_one_second_on_a_large_world() {
    let (mut hydrology, temperature) = world(128, 1);
    let start = std::time::Instant::now();
    for _ in 0..100 {
        hydrology.step(&temperature, 0.9999);
    }
    eprintln!("{:?} par seconde simulée", start.elapsed() / 100);
}

#[test]
#[ignore = "calibration de l'orage, lancée à la main"]
fn how_often_does_lightning_strike() {
    use crate::hydrology::Storm;
    for (size, heat) in [(10, true), (24, true), (24, false)] {
        let (mut hydrology, mut temperature) = world(size, 11);
        if heat {
            temperature.apply_heat_source(size / 2, size / 2, 8.0);
        }
        let mut storm = Storm::new(size);
        let (mut strikes, mut peak) = (0, 0.0_f32);
        for _ in 0..7_200 {
            hydrology.step(&temperature, 1.0);
            strikes += storm.step(&hydrology).len();
            peak = peak.max(storm.total());
        }
        eprintln!("monde {size}² chaleur {heat} : {strikes} éclairs en 7 200 s, charge totale maximale {peak:.2}");
    }
}
