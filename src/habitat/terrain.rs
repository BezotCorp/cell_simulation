#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub(crate) enum Terrain {
    Rock,
    Sand,
    Soil,
    Wall,
}

impl Terrain {
    pub(crate) fn is_passable(&self) -> bool {
        matches!(self, Self::Sand | Self::Soil)
    }
}
