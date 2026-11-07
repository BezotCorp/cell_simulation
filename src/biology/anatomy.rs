use crate::biology::{AnatomyPart, PartKind};
use crate::genetics::{Levels, Product};

/// Seuil de développement en deçà duquel un composant n'apparaît pas.
const PRESENT: f32 = 0.08;

/// Composants visibles d'un organisme, déduits des produits qu'il a développés. La liste n'est pas
/// fermée : une combinaison inédite de produits donne une combinaison inédite de composants.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Anatomy {
    parts: Vec<AnatomyPart>,
}

impl Anatomy {
    /// Anatomie correspondant aux produits `levels` d'un corps de taille `size` (en diamètres de
    /// l'ancêtre). Le nombre d'exemplaires croît avec le développement et avec la taille.
    pub(crate) fn of(levels: &Levels, size: f32, neural_units: usize) -> Self {
        let mut parts = Vec::new();
        let mut add = |kind: PartKind, level: f32, most: f32| {
            if level >= PRESENT {
                let count = (level * most * size.max(0.5)).ceil().clamp(1.0, 24.0) as u8;
                let variant = ((level * 4.0) as u8).min(3);
                parts.push(AnatomyPart::new(kind, count, level.min(1.0), variant));
            }
        };
        add(PartKind::Vacuole, levels.get(Product::Size), 4.0);
        add(PartKind::Chloroplast, levels.get(Product::Pigment), 10.0);
        add(PartKind::Thermophore, levels.get(Product::Thermal), 6.0);
        add(PartKind::Spine, levels.get(Product::Raptorial), 8.0);
        add(PartKind::Flagellum, levels.get(Product::Motor), 4.0);
        add(PartKind::Cilia, levels.get(Product::Hydration), 12.0);
        add(PartKind::Plate, levels.get(Product::Cuticle), 8.0);
        add(PartKind::Pad, levels.get(Product::Adhesin), 6.0);
        if neural_units > 0 {
            add(PartKind::Ganglion, neural_units as f32 / 8.0, 4.0);
        }
        Self { parts }
    }

    pub(crate) fn parts(&self) -> &[AnatomyPart] {
        &self.parts
    }
}
