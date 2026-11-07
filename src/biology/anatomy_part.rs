use crate::biology::PartKind;

/// Un composant anatomique : sa famille, le nombre d'exemplaires, son développement relatif
/// (0 à 1, la quantité de produit qui le justifie) et une variante qui découle de ce développement.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AnatomyPart {
    kind: PartKind,
    count: u8,
    scale: f32,
    variant: u8,
}

impl AnatomyPart {
    pub(crate) fn new(kind: PartKind, count: u8, scale: f32, variant: u8) -> Self {
        Self {
            kind,
            count,
            scale,
            variant,
        }
    }

    /// Le noyau, présent chez tout organisme.
    pub(crate) fn new_nucleus() -> Self {
        Self::new(PartKind::Nucleus, 1, 0.7, 1)
    }

    pub(crate) fn kind(&self) -> PartKind {
        self.kind
    }

    pub(crate) fn count(&self) -> u8 {
        self.count
    }

    /// Développement relatif, de 0 à 1.
    pub(crate) fn scale(&self) -> f32 {
        self.scale
    }

    /// Variante de 0 à 3, d'autant plus élevée que le composant est développé.
    pub(crate) fn variant(&self) -> u8 {
        self.variant
    }
}
