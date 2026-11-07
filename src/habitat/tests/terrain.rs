use crate::habitat::Terrain;

#[test]
fn solid_materials_block_movement_but_ground_remains_passable() {
    assert!(!Terrain::Rock.is_passable());
    assert!(!Terrain::Wall.is_passable());
    assert!(Terrain::Sand.is_passable());
    assert!(Terrain::Soil.is_passable());
}
