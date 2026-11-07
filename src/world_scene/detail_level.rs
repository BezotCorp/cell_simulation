/// Finesse du dessin d'un organisme : elle ne change que l'affichage, jamais la simulation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum DetailLevel {
    /// Membrane seule : les grandes foules.
    Bare,
    /// Noyau et deux composants les plus développés.
    Principal,
    /// Anatomie complète.
    Full,
}

impl DetailLevel {
    /// Nombre maximal de composants dessinés (hors noyau).
    pub(super) fn parts(self) -> usize {
        match self {
            Self::Bare => 0,
            Self::Principal => 2,
            Self::Full => usize::MAX,
        }
    }
}
