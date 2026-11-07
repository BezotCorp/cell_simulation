use crate::habitat::Grid;
use rand::{SeedableRng, rngs::StdRng};
use std::time::Duration;

fn food_site(grid: &Grid) -> (usize, usize) {
    (0..grid.size())
        .flat_map(|y| (0..grid.size()).map(move |x| (x, y)))
        .find(|&(x, y)| matches!(grid.get_entity(x, y), crate::habitat::Entity::Food(_)))
        .expect("la graine de référence doit contenir un site alimentaire")
}

#[test]
fn environment_retains_fractional_seconds_and_processes_all_elapsed_ticks() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut grid = Grid::new(world(1), &mut rng);
    grid.update(Duration::from_millis(992), &mut rng);
    assert_eq!(grid.hydrology().ticks(), 0);
    grid.update(Duration::from_millis(16), &mut rng);
    assert_eq!(grid.hydrology().ticks(), 1);
    grid.update(Duration::from_millis(1992), &mut rng);
    assert_eq!(grid.hydrology().ticks(), 3);
}

#[test]
fn food_is_consumed_once_and_only_up_to_the_requested_deficit() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut grid = Grid::new(world(10), &mut rng);
    let (x, y) = food_site(&grid);
    assert_eq!(grid.consume_food(x, y, 3.0), 3.0);
    assert_eq!(grid.food_energy(x, y), 17.0);
    assert_eq!(grid.consume_food(x, y, 100.0), 17.0);
    assert_eq!(grid.consume_food(x, y, 100.0), 0.0);
    assert_eq!(grid.consume_food(x, y, -1.0), 0.0);
}

#[test]
fn depleted_site_regenerates_at_finite_rate_without_exceeding_capacity() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut grid = Grid::new(world(10), &mut rng);
    let (x, y) = food_site(&grid);
    grid.consume_food(x, y, 20.0);
    grid.update(Duration::from_secs(1), &mut rng);
    assert_eq!(grid.food_energy(x, y), 2.0);
    grid.update(Duration::from_secs(100), &mut rng);
    assert_eq!(grid.food_energy(x, y), 20.0);
}

#[test]
fn exploration_prefers_the_richest_reachable_site_in_any_direction() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut grid = Grid::new(world(10), &mut rng);
    let (food_x, food_y) = food_site(&grid);
    for y in 0..10 {
        for x in 0..10 {
            if (x, y) != (food_x, food_y) {
                grid.consume_food(x, y, 20.0);
            }
        }
    }
    // Depuis un site voisin, l'organisme atteint le site nourri, qu'il soit à l'est, à l'ouest ou en
    // diagonale : aucune orientation de la grille n'est privilégiée.
    let mut reached = 0;
    for (dx, dy) in [(1, 0), (9, 0), (0, 1), (0, 9), (1, 1), (9, 9)] {
        let origin = (
            ((food_x + dx) % 10) as f32 + 0.5,
            ((food_y + dy) % 10) as f32 + 0.5,
        );
        for _ in 0..16 {
            if let Some(point) = grid.exploration_destination(origin, &vec![0.0; 100], 1.0, &mut rng)
                && grid.site_at(point) == (food_x, food_y)
            {
                reached += 1;
            }
        }
    }
    assert!(reached >= 6 * 16 * 2 / 3, "{reached} arrivées sur 96");
}

#[test]
fn exploration_uses_every_direction_and_never_enters_solid_obstacles() {
    let mut rng = StdRng::seed_from_u64(0);
    let grid = Grid::new(world(10), &mut rng);
    let mut diagonal = 0;
    for y in 0..10 {
        for x in 0..10 {
            for _ in 0..8 {
                let origin = (x as f32 + 0.5, y as f32 + 0.5);
                if let Some(point) =
                    grid.exploration_destination(origin, &vec![0.0; 100], 1.0, &mut rng)
                {
                    let (nx, ny) = grid.site_at(point);
                    assert!(grid.is_passable(nx, ny));
                    assert_ne!((nx, ny), (x, y));
                    diagonal += usize::from(nx != x && ny != y);
                }
            }
        }
    }
    assert!(diagonal > 0, "les diagonales doivent être possibles");
    let single = Grid::new(world(1), &mut rng);
    assert_eq!(
        single.exploration_destination((0.5, 0.5), &[0.0], 1.0, &mut rng),
        None
    );
}

#[test]
fn birth_points_stay_near_the_mother_and_may_cross_into_a_neighbor_site() {
    let mut rng = StdRng::seed_from_u64(1);
    let grid = Grid::new(world(10), &mut rng);
    let mother = (4.95, 5.05);
    let mut other_sites = 0;
    for _ in 0..200 {
        let point = grid.birth_point(mother, &mut rng);
        let wrapped = |a: f32, b: f32| ((a - b).abs()).min(10.0 - (a - b).abs());
        assert!(wrapped(point.0, mother.0) <= 0.5 + 1e-4 && wrapped(point.1, mother.1) <= 0.5 + 1e-4);
        let (x, y) = grid.site_at(point);
        assert!(grid.is_passable(x, y));
        other_sites += usize::from((x, y) != grid.site_at(mother));
    }
    assert!(other_sites > 0);
}

#[test]
fn temperature_is_continuous_between_site_centers() {
    let mut rng = StdRng::seed_from_u64(0);
    let grid = Grid::new(world(10), &mut rng);
    for y in 0..10 {
        for x in 0..10 {
            let center = grid.temperature_at((x as f32 + 0.5, y as f32 + 0.5));
            assert!((center - grid.get_temperature(x, y)).abs() < 1e-3);
            // Aucun saut à la frontière de deux sites.
            let before = grid.temperature_at((x as f32 + 1.0 - 1e-3, y as f32 + 0.5));
            let after = grid.temperature_at((x as f32 + 1.0 + 1e-3, y as f32 + 0.5));
            assert!((before - after).abs() < 0.05);
        }
    }
}

#[test]
fn renewing_food_is_not_abandoned_for_an_empty_neighbor() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut grid = Grid::new(world(10), &mut rng);
    let (x, y) = food_site(&grid);
    for ny in 0..10 {
        for nx in 0..10 {
            grid.consume_food(nx, ny, 100.0);
        }
    }
    grid.update(Duration::from_millis(16), &mut rng);
    // Toutes les réserves présentes se renouvellent au même débit : aucun gain à partir.
    assert!(grid.food_energy(x, y) > 0.0);
    assert_eq!(
        grid.exploration_destination((x as f32 + 0.5, y as f32 + 0.5), &vec![0.0; 100], 1.0, &mut rng),
        None
    );
}

#[test]
fn initial_position_is_accessible_without_removing_an_obstacle() {
    for seed in 0..32 {
        let mut rng = StdRng::seed_from_u64(seed);
        let grid = Grid::new(world(10), &mut rng);
        let (x, y) = grid.initial_position().unwrap();
        assert!(grid.is_passable(x, y));
        // L'ancêtre est aquatique : à distance égale, un site submergé est préféré.
        if grid.is_passable(5, 5) && grid.hydrology().is_submerged(5, 5) {
            assert_eq!((x, y), (5, 5));
        }
        if (0..10).any(|yy| (0..10).any(|xx| grid.is_passable(xx, yy) && grid.hydrology().is_submerged(xx, yy))) {
            assert!(grid.hydrology().is_submerged(x, y));
        }
    }
}

#[test]
fn exploration_remains_possible_when_local_food_cannot_cover_occupants_maintenance() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut grid = Grid::new(world(10), &mut rng);
    let (x, y) = food_site(&grid);
    for ny in 0..10 {
        for nx in 0..10 {
            grid.consume_food(nx, ny, 100.0);
        }
    }
    grid.update(Duration::from_millis(16), &mut rng);
    // Trois cellules exigent 3 unités/s ; le site n'en produit que 2.
    let mut maintenance = vec![0.0; 100];
    maintenance[y * 10 + x] = 3.0;
    assert!(
        grid.exploration_destination((x as f32 + 0.5, y as f32 + 0.5), &maintenance, 1.0, &mut rng)
            .is_some()
    );
}

#[test]
fn human_defined_terrain_and_minerals_override_generation() {
    let text = include_str!("../../../data/definitions.ron").replace("placements: []", "placements: [(x: 0, y: 0, entity: Obstacle(Wall)), (x: 1, y: 0, entity: Minerals(Iron)), (x: 2, y: 0, entity: HeatSource(7.0))]");
    let definitions: crate::definitions::Definitions = ron::from_str(&text).unwrap();
    definitions.validate().unwrap();
    let mut rng = crate::ecosystem::SimulationRng::new(0);
    let grid = crate::habitat::Grid::new(definitions.world().clone(), &mut rng);
    assert!(!grid.is_passable(0, 0));
    assert!(matches!(
        grid.get_entity(1, 0),
        crate::habitat::Entity::Minerals(crate::habitat::Mineral::Iron)
    ));
    assert!(grid.get_temperature(2, 0) >= definitions.world().ambient_temperature() + 7.0);
}

#[test]
fn identities_and_food_sites_stay_indexed_while_resources_spread() {
    let mut rng = StdRng::seed_from_u64(3);
    let mut grid = Grid::new(world(20), &mut rng);
    for second in 0..600 {
        grid.update(Duration::from_secs(1), &mut rng);
        if second % 50 == 0 {
            let scanned: Vec<usize> = (0..400)
                .filter(|index| {
                    matches!(
                        grid.get_entity(index % 20, index / 20),
                        crate::habitat::Entity::Food(_)
                    )
                })
                .collect();
            let mut indexed = grid.food_sites().to_vec();
            indexed.sort_unstable();
            assert_eq!(indexed, scanned);
        }
    }
    assert!(grid.food_sites().len() > 5, "des sites doivent apparaître");
    for y in 0..20 {
        for x in 0..20 {
            assert_eq!(grid.habitat_position(grid.habitat_id(x, y)), Some((x, y)));
        }
    }
    assert_eq!(grid.habitat_position(uuid::Uuid::now_v7()), None);
}

fn world(size: usize) -> std::sync::Arc<crate::definitions::WorldDefinition> {
    let text =
        include_str!("../../../data/definitions.ron").replace("size: 10", &format!("size: {size}"));
    let definitions: crate::definitions::Definitions = ron::from_str(&text).unwrap();
    definitions.world().clone()
}
