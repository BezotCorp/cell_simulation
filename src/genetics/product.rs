/// Produits biochimiques et structuraux d'un organisme : ce que les gènes fabriquent. Un gène agit
/// sur plusieurs produits à la fois (pléiotropie) et un produit est fabriqué par plusieurs gènes
/// (polygénie) ; le phénotype se déduit des quantités de ces produits par des lois physiques, jamais
/// d'un gène isolé. Ajouter un produit, c'est ajouter une variante ici et lui donner des effets dans
/// `Physiology`, `Morphology` ou `Anatomy`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub(crate) enum Product {
    /// Enzymes du métabolisme : cadence de la dépense d'énergie.
    Metabolism,
    /// Biosynthèse de la masse : taille de l'organisme.
    Size,
    /// Investissement reproducteur : part des réserves nécessaire pour se diviser.
    Fecundity,
    /// Réparation de l'ADN : abaisse la fréquence des mutations.
    Repair,
    /// Régulation hydrique en milieu aquatique (membrane, osmorégulation).
    Hydration,
    /// Cuticule : résistance à la dessiccation en milieu terrestre.
    Cuticle,
    /// Tissu nerveux : nombre d'unités de calcul.
    Neural,
    /// Enzymes digestives : ingestion des réserves alimentaires.
    Digestive,
    /// Pigments photosynthétiques.
    Pigment,
    /// Capteurs de chaleur et enzymes thermophiles.
    Thermal,
    /// Structures de capture et de prélèvement sur les proies.
    Raptorial,
    /// Appareil moteur (flagelles, cils).
    Motor,
    /// Protéines d'adhésion aux cellules voisines.
    Adhesin,
}

impl Product {
    pub(crate) const COUNT: usize = 13;

    pub(crate) const ALL: [Product; Self::COUNT] = [
        Self::Metabolism,
        Self::Size,
        Self::Fecundity,
        Self::Repair,
        Self::Hydration,
        Self::Cuticle,
        Self::Neural,
        Self::Digestive,
        Self::Pigment,
        Self::Thermal,
        Self::Raptorial,
        Self::Motor,
        Self::Adhesin,
    ];

    pub(crate) fn index(self) -> usize {
        self as usize
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Metabolism => "Métabolisme",
            Self::Size => "Croissance",
            Self::Fecundity => "Fécondité",
            Self::Repair => "Réparation",
            Self::Hydration => "Hydratation",
            Self::Cuticle => "Cuticule",
            Self::Neural => "Tissu nerveux",
            Self::Digestive => "Digestion",
            Self::Pigment => "Pigments",
            Self::Thermal => "Thermosynthèse",
            Self::Raptorial => "Prédation",
            Self::Motor => "Appareil moteur",
            Self::Adhesin => "Adhésion",
        }
    }
}
