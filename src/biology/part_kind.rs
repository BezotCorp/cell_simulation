/// Famille d'un composant anatomique. Chaque famille correspond à un produit du génome et à une
/// fonction réelle de l'organisme : un composant n'existe à l'écran que si la quantité de produit
/// qui le justifie (et qui agit sur la physiologie) est exprimée. Le moteur graphique associe à
/// chaque famille un emplacement (`slot`) de sa bibliothèque d'assets ; le génome n'en sait rien.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum PartKind {
    /// Noyau : tout organisme en a un.
    Nucleus,
    /// Réserves et vacuoles (croissance).
    Vacuole,
    /// Pigments photosynthétiques.
    Chloroplast,
    /// Organites thermiques.
    Thermophore,
    /// Appendices de capture.
    Spine,
    /// Flagelles de l'appareil moteur.
    Flagellum,
    /// Cils de la régulation hydrique.
    Cilia,
    /// Plaques de la cuticule.
    Plate,
    /// Tissu nerveux.
    Ganglion,
    /// Zones d'adhésion.
    Pad,
}

impl PartKind {
    /// Nom stable de l'emplacement dans la bibliothèque d'assets.
    pub(crate) fn slot(self) -> &'static str {
        match self {
            Self::Nucleus => "nucleus",
            Self::Vacuole => "vacuole",
            Self::Chloroplast => "chloroplast",
            Self::Thermophore => "thermophore",
            Self::Spine => "spine",
            Self::Flagellum => "flagellum",
            Self::Cilia => "cilia",
            Self::Plate => "plate",
            Self::Ganglion => "ganglion",
            Self::Pad => "pad",
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Nucleus => "noyau",
            Self::Vacuole => "vacuoles",
            Self::Chloroplast => "chloroplastes",
            Self::Thermophore => "thermophores",
            Self::Spine => "épines",
            Self::Flagellum => "flagelles",
            Self::Cilia => "cils",
            Self::Plate => "plaques",
            Self::Ganglion => "ganglions",
            Self::Pad => "zones d'adhésion",
        }
    }
}
