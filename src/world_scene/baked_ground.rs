use crate::definitions::MaterialKind;
use crate::ecosystem::Simulation;
use crate::habitat::{Entity, Terrain};
use crate::world_scene::{
    materials::temperature_color,
    noise::{cell_value, fbm2},
};

/// Textures du sol cuites une fois : couleurs du substrat (vue écosystème et vue thermique) et
/// relief. Le sol est un unique plan texturé : des milliers d'habitats ne coûtent ni nœud de scène
/// ni appel de rendu. Aucune rainure ni bordure ne dessine la grille de calcul : le monde est continu.
pub(super) struct BakedGround {
    side: u32,
    ecosystem: Vec<u8>,
    temperature: Vec<u8>,
    normal: Vec<u8>,
}

impl BakedGround {
    /// Nombre de texels par côté d'habitat : fin pour les petites grilles, plafonné pour
    /// que la texture reste raisonnable pour les grandes.
    fn texels_per_tile(size: usize) -> usize {
        (1024 / size.max(1)).clamp(8, 64)
    }

    pub(super) fn bake(simulation: &Simulation) -> Self {
        let grid = simulation.grid();
        let appearance = simulation.definitions().appearance();
        let size = grid.size();
        let per_tile = Self::texels_per_tile(size);
        let side = size * per_tile;
        let color = |kind: MaterialKind| {
            let c = appearance.materials()[&kind].color();
            [c[0] as f32, c[1] as f32, c[2] as f32]
        };
        let (ground, soil, sand, water, rock) = (
            color(MaterialKind::Ground),
            color(MaterialKind::Soil),
            color(MaterialKind::Sand),
            color(MaterialKind::Water),
            color(MaterialKind::Rock),
        );
        // Teinte de fond de chaque site d'après son contenu, avec la part de sol nu qui règle
        // l'éclairage ultérieur. Ces valeurs sont des conditions locales : elles sont fondues entre
        // sites voisins et ne dessinent aucune frontière.
        let substrate: Vec<([f32; 3], f32)> = (0..size * size)
            .map(|index| match grid.get_entity(index % size, index / size) {
                Entity::Obstacle(Terrain::Sand) => (sand, 0.0),
                Entity::Obstacle(Terrain::Soil) => (soil, 0.0),
                Entity::Obstacle(Terrain::Rock | Terrain::Wall) => {
                    ([rock[0] * 0.55, rock[1] * 0.55, rock[2] * 0.6], 0.0)
                }
                Entity::Water(_) => ([water[0] * 0.8, water[1] * 0.8, water[2] * 0.85], 0.0),
                _ => (ground, 1.0),
            })
            .collect();
        let ambient = grid.temperature_map().ambient_temperature();
        let clamp = |v: i64| v.clamp(0, size as i64 - 1) as usize;
        let temperature_at = |x: i64, y: i64| grid.get_temperature(clamp(x), clamp(y));

        let mut heights = vec![0.0_f32; side * side];
        let mut ecosystem = vec![0u8; side * side * 4];
        let mut temperature = vec![0u8; side * side * 4];

        for ty in 0..side {
            for tx in 0..side {
                let (fx, fy) = (
                    (tx as f32 + 0.5) / per_tile as f32,
                    (ty as f32 + 0.5) / per_tile as f32,
                );
                // Frontières des plages de substrat gauchies : le sol évite l'aspect de damier.
                let warp = |axis: f32, salt: f32| {
                    axis + (fbm2(fx * 1.7 + salt, fy * 1.7 - salt, 91, 3) - 0.5) * 0.95
                };
                // Fondu bilinéaire adouci entre les centres de sites : transitions progressives.
                let (wx, wy) = (warp(fx, 0.0) - 0.5, warp(fy, 5.0) - 0.5);
                let (x0, y0) = (wx.floor(), wy.floor());
                let blend = |t: f32| t * t * (3.0 - 2.0 * t);
                let (bx, by) = (blend(wx - x0), blend(wy - y0));
                let (x0, y0) = (x0 as i64, y0 as i64);
                let mut base = [0.0_f32; 3];
                let mut bare = 0.0;
                for (dx, dy, weight) in [
                    (0, 0, (1.0 - bx) * (1.0 - by)),
                    (1, 0, bx * (1.0 - by)),
                    (0, 1, (1.0 - bx) * by),
                    (1, 1, bx * by),
                ] {
                    let (color, share) = substrate[clamp(x0 + dx) + clamp(y0 + dy) * size];
                    for channel in 0..3 {
                        base[channel] += color[channel] * weight;
                    }
                    bare += share * weight;
                }

                let detail = fbm2(fx * 3.1, fy * 3.1, 7, 4);
                let fine = fbm2(fx * 11.0, fy * 11.0, 13, 2);
                let speck = cell_value(tx as i32, ty as i32, 3);
                let shade = (1.05 + 0.85 * bare)
                    * (0.72 + 0.5 * detail + 0.10 * (fine - 0.5))
                    * if speck > 0.985 { 1.35 } else { 1.0 };

                // Température interpolée entre les centres d'habitats : dégradé continu.
                let (cx, cy) = (fx - 0.5, fy - 0.5);
                let (x0, y0) = (cx.floor(), cy.floor());
                let (bx, by) = (cx - x0, cy - y0);
                let (x0, y0) = (x0 as i64, y0 as i64);
                let heat = temperature_at(x0, y0) * (1.0 - bx) * (1.0 - by)
                    + temperature_at(x0 + 1, y0) * bx * (1.0 - by)
                    + temperature_at(x0, y0 + 1) * (1.0 - bx) * by
                    + temperature_at(x0 + 1, y0 + 1) * bx * by;

                let warmth = ((heat - ambient) / 30.0).clamp(0.0, 1.0);
                let glow = [70.0 * warmth, 24.0 * warmth, 0.0];
                let thermal = temperature_color((heat - 15.0) / 45.0);
                let offset = (ty * side + tx) * 4;
                for channel in 0..3 {
                    ecosystem[offset + channel] =
                        (base[channel] * shade + glow[channel]).clamp(0.0, 255.0) as u8;
                    temperature[offset + channel] =
                        (thermal[channel] * (0.6 + 0.5 * detail))
                            .clamp(0.0, 255.0) as u8;
                }
                ecosystem[offset + 3] = 255;
                temperature[offset + 3] = 255;
                heights[ty * side + tx] = 0.45 * detail + 0.15 * fine;
            }
        }

        // Pente apparente constante : le gradient par texel baisse quand les habitats ont plus de texels.
        let strength = 0.875 * per_tile as f32 / 64.0;
        let normal = Self::normal_map(&heights, side, strength);

        Self {
            side: side as u32,
            ecosystem,
            temperature,
            normal,
        }
    }

    /// Carte de normales (octets RGBA) d'un champ de hauteurs carré de côté `side`.
    fn normal_map(heights: &[f32], side: usize, strength: f32) -> Vec<u8> {
        let height = |x: usize, y: usize| heights[y.min(side - 1) * side + x.min(side - 1)];
        let mut normal = vec![0u8; side * side * 4];
        for y in 0..side {
            for x in 0..side {
                let dx = height(x + 1, y) - height(x.saturating_sub(1), y);
                let dy = height(x, y + 1) - height(x, y.saturating_sub(1));
                let n = fyrox::core::algebra::Vector3::new(-dx * strength, -dy * strength, 1.0)
                    .normalize();
                let offset = (y * side + x) * 4;
                for (channel, value) in [n.x, n.y, n.z].into_iter().enumerate() {
                    normal[offset + channel] = ((value * 0.5 + 0.5) * 255.0) as u8;
                }
                normal[offset + 3] = 255;
            }
        }
        normal
    }

    pub(super) fn side(&self) -> u32 {
        self.side
    }

    pub(super) fn ecosystem(&self) -> &[u8] {
        &self.ecosystem
    }

    pub(super) fn temperature(&self) -> &[u8] {
        &self.temperature
    }

    pub(super) fn normal(&self) -> &[u8] {
        &self.normal
    }
}
