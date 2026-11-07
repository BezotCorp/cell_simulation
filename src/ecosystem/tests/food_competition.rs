use crate::ecosystem::FoodCompetition;
use crate::habitat::{Entity, Grid};
use rand::{SeedableRng, rngs::StdRng};

fn food_site(grid: &Grid) -> (usize, usize) {
    (0..grid.size())
        .flat_map(|y| (0..grid.size()).map(move |x| (x, y)))
        .find(|&(x, y)| matches!(grid.get_entity(x, y), Entity::Food(_)))
        .unwrap()
}

#[test]
fn scarce_food_is_shared_proportionally_without_order_priority_or_duplication() {
    for reverse in [false, true] {
        let mut rng = StdRng::seed_from_u64(0);
        let mut grid = Grid::new(world(10), &mut rng);
        let (x, y) = food_site(&grid);
        let mut competition = FoodCompetition::default();
        competition.reset(10);
        for deficit in if reverse { [30.0, 10.0] } else { [10.0, 30.0] } {
            competition.request(x, y, deficit);
        }
        competition.consume_from(&mut grid);
        assert_eq!(competition.share(x, y, 10.0), 5.0);
        assert_eq!(competition.share(x, y, 30.0), 15.0);
        assert_eq!(grid.food_energy(x, y), 0.0);
    }
}

#[test]
fn abundant_food_only_fills_deficits_and_full_cells_leave_food_available() {
    let mut rng = StdRng::seed_from_u64(0);
    let mut grid = Grid::new(world(10), &mut rng);
    let (x, y) = food_site(&grid);
    let mut competition = FoodCompetition::default();
    competition.reset(10);
    competition.request(x, y, 0.0);
    competition.request(x, y, 3.0);
    competition.consume_from(&mut grid);
    assert_eq!(competition.share(x, y, 0.0), 0.0);
    assert_eq!(competition.share(x, y, 3.0), 3.0);
    assert_eq!(grid.food_energy(x, y), 17.0);
}

fn world(size: usize) -> std::sync::Arc<crate::definitions::WorldDefinition> {
    let text =
        include_str!("../../../data/definitions.ron").replace("size: 10", &format!("size: {size}"));
    let definitions: crate::definitions::Definitions = ron::from_str(&text).unwrap();
    definitions.world().clone()
}
