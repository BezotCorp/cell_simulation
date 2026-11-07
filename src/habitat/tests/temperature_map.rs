use crate::habitat::TemperatureMap;

/// Balayage complet d'origine, conservé comme référence de comportement.
fn reference(size: usize, sources: &[(usize, usize, f32)]) -> Vec<Vec<f32>> {
    let mut grid = vec![vec![20.0_f32; size]; size];
    for &(x, y, intensity) in sources {
        for dy in 0..size {
            for dx in 0..size {
                let distance =
                    (((dx as i32 - x as i32).pow(2) + (dy as i32 - y as i32).pow(2)) as f32).sqrt();
                if distance < intensity {
                    grid[dy][dx] += (intensity - distance) * 2.0;
                }
            }
        }
    }
    grid
}

#[test]
fn heat_sources_touch_only_their_reach_yet_match_the_full_scan_exactly() {
    for size in [1, 7, 24] {
        let sources: Vec<_> = [
            (0, 0, 5.5),
            (size - 1, size - 1, 9.0),
            (size / 2, size / 3, 14.25),
            (size / 2, size / 2, 0.0),
            (size / 4, size - 1, 400.0),
            (size / 2, 0, 3.0),
        ]
        .into_iter()
        .map(|(x, y, intensity)| (x.min(size - 1), y.min(size - 1), intensity))
        .collect();
        let mut map = TemperatureMap::new(size, 20.0);
        for &(x, y, intensity) in &sources {
            map.apply_heat_source(x, y, intensity);
        }
        let expected = reference(size, &sources);
        for y in 0..size {
            for x in 0..size {
                assert_eq!(map.get(x, y).to_bits(), expected[y][x].to_bits());
            }
        }
    }
}
