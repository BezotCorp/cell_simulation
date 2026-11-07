use crate::world_scene::materials::{BANDS, Materials};

#[test]
fn bands_cover_the_whole_range_and_tolerate_out_of_range_values() {
    assert_eq!(Materials::band(0.0), 0);
    assert_eq!(Materials::band(1.0), (BANDS - 1) as u8);
    assert_eq!(Materials::band(-5.0), 0);
    assert_eq!(Materials::band(42.0), (BANDS - 1) as u8);
    assert_eq!(Materials::band(f32::NAN), 0);
    assert_eq!(Materials::band(f32::INFINITY), (BANDS - 1) as u8);
    let bands: Vec<_> = (0..=1000)
        .map(|i| Materials::band(i as f32 / 1000.0))
        .collect();
    assert!(bands.windows(2).all(|pair| pair[0] <= pair[1]));
    assert_eq!(
        bands.iter().collect::<std::collections::HashSet<_>>().len(),
        BANDS
    );
}
