/// Un composant développé : sa définition (indice dans les composants de l'espèce), son nombre
/// d'exemplaires, son niveau de développement de 0 à 1 et le numéro de sa variante.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AnatomyPart {
    part: usize,
    count: u8,
    level: f32,
    variant: usize,
}

impl AnatomyPart {
    pub(crate) fn new(part: usize, count: u8, level: f32, variant: usize) -> Self {
        Self { part, count, level, variant }
    }

    /// Indice de la définition dans `SpeciesDefinition::parts`.
    pub(crate) fn part(&self) -> usize {
        self.part
    }

    pub(crate) fn count(&self) -> u8 {
        self.count
    }

    pub(crate) fn level(&self) -> f32 {
        self.level
    }

    pub(crate) fn variant(&self) -> usize {
        self.variant
    }
}
