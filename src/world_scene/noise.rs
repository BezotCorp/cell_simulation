//! Bruit de valeur déterministe : relief des textures et formes organiques. Aucune source
//! aléatoire externe, pour que deux lancements d'une même graine produisent le même décor.

fn hash(x: i32, y: i32, z: i32, seed: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x8da6_b343)
        ^ (y as u32).wrapping_mul(0xd816_3841)
        ^ (z as u32).wrapping_mul(0xcb1a_b31f)
        ^ seed.wrapping_mul(0x9e37_79b9);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297a_2d39);
    h ^= h >> 15;
    (h >> 8) as f32 / (1u32 << 24) as f32
}

fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Valeur aléatoire fixe d'une cellule entière, dans [0, 1).
pub(super) fn cell_value(x: i32, y: i32, seed: u32) -> f32 {
    hash(x, y, 0, seed)
}

/// Bruit de valeur lisse en 3D, dans [0, 1].
pub(super) fn value3(x: f32, y: f32, z: f32, seed: u32) -> f32 {
    let (ix, iy, iz) = (x.floor(), y.floor(), z.floor());
    let (fx, fy, fz) = (smooth(x - ix), smooth(y - iy), smooth(z - iz));
    let (ix, iy, iz) = (ix as i32, iy as i32, iz as i32);
    let corner = |dx, dy, dz| hash(ix + dx, iy + dy, iz + dz, seed);
    let along_x = |dy, dz| lerp(corner(0, dy, dz), corner(1, dy, dz), fx);
    lerp(
        lerp(along_x(0, 0), along_x(1, 0), fy),
        lerp(along_x(0, 1), along_x(1, 1), fy),
        fz,
    )
}

/// Bruit de valeur lisse en 2D, dans [0, 1].
pub(super) fn value2(x: f32, y: f32, seed: u32) -> f32 {
    value3(x, y, 0.0, seed)
}

/// Somme d'octaves de bruit 2D, normalisée dans [0, 1].
pub(super) fn fbm2(x: f32, y: f32, seed: u32, octaves: u32) -> f32 {
    let (mut sum, mut amplitude, mut frequency, mut total) = (0.0, 1.0, 1.0, 0.0);
    for octave in 0..octaves {
        sum += amplitude * value2(x * frequency, y * frequency, seed.wrapping_add(octave));
        total += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    sum / total
}

/// Somme d'octaves de bruit 3D, normalisée dans [0, 1].
pub(super) fn fbm3(x: f32, y: f32, z: f32, seed: u32, octaves: u32) -> f32 {
    let (mut sum, mut amplitude, mut frequency, mut total) = (0.0, 1.0, 1.0, 0.0);
    for octave in 0..octaves {
        sum += amplitude
            * value3(
                x * frequency,
                y * frequency,
                z * frequency,
                seed.wrapping_add(octave),
            );
        total += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    sum / total
}
