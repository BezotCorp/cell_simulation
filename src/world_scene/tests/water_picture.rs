use crate::habitat::TemperatureMap;
use crate::hydrology::{GroundKind, Hydrology, INITIAL_FILL};
use crate::world_scene::water_picture::WaterPicture;

fn weather(size: usize, ticks: usize) -> Hydrology {
    let ground = vec![GroundKind::Bare; size * size];
    let mut hydrology = Hydrology::new(size, 4, ground, INITIAL_FILL, &[]);
    let temperature = TemperatureMap::new(size, 20.0);
    for _ in 0..ticks {
        hydrology.step(&temperature, 1.0);
    }
    hydrology
}

#[test]
fn the_painted_world_has_no_visible_site_boundaries() {
    let hydrology = weather(12, 400);
    let picture = WaterPicture::new(12);
    let side = picture.side() as usize;
    let pixels = picture.surface(&hydrology);
    assert_eq!(pixels.len(), side * side * 4);
    let per_site = side / 12;
    let (mut inside, mut crossing) = (0.0_f32, 0.0_f32);
    for y in 0..side {
        for x in 0..side - 1 {
            let jump = (0..4)
                .map(|c| (f32::from(pixels[(y * side + x) * 4 + c]) - f32::from(pixels[(y * side + x + 1) * 4 + c])).abs())
                .fold(0.0, f32::max);
            // Franchit-on la frontière entre deux sites (au texel multiple de `per_site`) ?
            if (x + 1) % per_site == 0 {
                crossing = crossing.max(jump);
            } else {
                inside = inside.max(jump);
            }
        }
    }
    assert!(inside > 0.0);
    // Une frontière de site ne produit pas de saut plus fort que le reste de l'image.
    assert!(crossing <= inside * 1.5 + 2.0, "frontière {crossing}, intérieur {inside}");
}

#[test]
fn water_is_painted_where_it_stands_and_the_sky_only_where_clouds_form() {
    let hydrology = weather(12, 600);
    let picture = WaterPicture::new(12);
    let side = picture.side() as usize;
    let pixels = picture.surface(&hydrology);
    let deepest = (0..144)
        .max_by(|a, b| hydrology.depth(a % 12, a / 12).total_cmp(&hydrology.depth(b % 12, b / 12)))
        .unwrap();
    let (x, y) = ((deepest % 12) * side / 12 + side / 24, (deepest / 12) * side / 12 + side / 24);
    let at = (y * side + x) * 4;
    // Le bassin le plus profond est un aplat d'eau bleue, bien plus opaque que le sol sec.
    assert!(pixels[at + 3] > 150, "alpha {}", pixels[at + 3]);
    assert!(pixels[at + 2] > pixels[at], "l'eau est plus bleue que rouge");
    let sky = picture.sky(&hydrology);
    let covered = sky.chunks(4).filter(|p| p[3] > 8).count();
    assert!(covered > 0 && covered < sky.len() / 4, "voile partiel : {covered}");
}
