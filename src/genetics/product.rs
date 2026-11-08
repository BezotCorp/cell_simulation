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
    /// Conjugaison : transfert de fragments de génome à une cellule liée.
    Conjugation,
    /// Recombinaison : à la division, le génome de la fille mêle celui de la mère et d'une cellule liée.
    Recombination,
}

impl Product {
    pub(crate) const COUNT: usize = 15;

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
        Self::Conjugation,
        Self::Recombination,
    ];

    pub(crate) fn index(self) -> usize {
        self as usize
    }

    /// Ce que le moteur fait de ce produit : un mécanisme disponible pour les capacités.
    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::Metabolism => "Cadence de la dépense d'énergie (0,5 × 2^(2·niveau) par seconde).",
            Self::Size => "Taille du corps, donc réserves (∝ taille³) et entretien.",
            Self::Fecundity => "Part des réserves nécessaire pour se diviser.",
            Self::Repair => "Réduit la fréquence des mutations, mais rend la réplication plus coûteuse.",
            Self::Hydration => "Adaptation à la vie dans l'eau ; nage contre les courants.",
            Self::Cuticle => "Adaptation à la vie sur terre (résistance à la dessiccation).",
            Self::Neural => "Unités de calcul : un capteur déclenche l'exploration.",
            Self::Digestive => "Ingestion des réserves alimentaires des sites.",
            Self::Pigment => "Capte la lumière du site (photosynthèse).",
            Self::Thermal => "Capte l'excès de chaleur au-dessus de l'ambiante.",
            Self::Raptorial => "Prélève de l'énergie sur un voisin du même site (prédation).",
            Self::Motor => "Déplacements actifs ; sous 0,05 l'organisme est sessile.",
            Self::Adhesin => "Reste lié à ses filles et partage l'énergie (coopération).",
            Self::Conjugation => "Transfère un fragment de génome à une cellule liée.",
            Self::Recombination => "À la division, mêle le génome de la mère à celui d'une cellule liée.",
        }
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
            Self::Conjugation => "Conjugaison",
            Self::Recombination => "Recombinaison",
        }
    }
}
