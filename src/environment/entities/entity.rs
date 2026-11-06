use crate::environment::{Mineral, Terrain};

#[derive(Clone, Debug)]
pub(crate) enum Entity {
    Empty,
    Food(f32),         // Énergie fournie
    HeatSource(f32),   // Intensité thermique
    Obstacle(Terrain), // Type de terrain/obstacle
    Water(f32),        // Profondeur d'eau
    Oxygen(f32),       // Concentration en oxygène
    Minerals(Mineral), // Ressources minérales
}
