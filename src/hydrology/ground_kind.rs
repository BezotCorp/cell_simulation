use crate::habitat::{Entity, Terrain};

/// Nature du sol d'un site : elle règle l'infiltration, la rétention et la résistance à l'érosion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GroundKind {
    /// Roche et murs : imperméables, inérodables, solides.
    Bedrock,
    Sand,
    Soil,
    /// Sol nu des sites sans substrat particulier.
    Bare,
}

impl GroundKind {
    pub(crate) fn of(entity: &Entity) -> Self {
        match entity {
            Entity::Obstacle(Terrain::Rock | Terrain::Wall) => Self::Bedrock,
            Entity::Obstacle(Terrain::Sand) => Self::Sand,
            Entity::Obstacle(Terrain::Soil) => Self::Soil,
            _ => Self::Bare,
        }
    }

    /// Fraction du débit d'infiltration maximal.
    pub(super) fn permeability(self) -> f32 {
        match self {
            Self::Bedrock => 0.0,
            Self::Sand => 1.0,
            Self::Soil => 0.35,
            Self::Bare => 0.5,
        }
    }

    /// Eau que le sol peut retenir, en hauteur d'eau équivalente.
    pub(super) fn porosity(self) -> f32 {
        match self {
            Self::Bedrock => 0.0,
            Self::Sand => 0.2,
            Self::Soil => 0.35,
            Self::Bare => 0.28,
        }
    }

    /// Facilité d'arrachement par le courant.
    pub(super) fn erodibility(self) -> f32 {
        match self {
            Self::Bedrock => 0.0,
            Self::Sand => 1.0,
            Self::Soil => 0.5,
            Self::Bare => 0.7,
        }
    }

    pub(crate) fn is_solid(self) -> bool {
        self == Self::Bedrock
    }
}
