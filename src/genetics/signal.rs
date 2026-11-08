/// Informations du milieu et de l'organisme que les gènes peuvent lire pour régler leur expression.
/// Un gène réagit à ces signaux selon ses propres sensibilités, héritées et mutables : l'expression
/// d'un même génome diffère donc d'un milieu à l'autre.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub(crate) enum Signal {
    /// Éclairement du site, de 0 à 1.
    Light,
    /// Excès de température sur l'ambiante, rapporté à 30 °C.
    Warmth,
    /// 1 si le site est submergé, 0 sinon.
    Water,
    /// Réserve alimentaire du site, rapportée à la capacité d'un site.
    Food,
    /// Densité d'occupants du site, rapportée à 8 voisins.
    Crowding,
    /// Réserve d'énergie de l'organisme, en part de sa capacité.
    Energy,
    /// Âge, en part de la maturité sexuelle : permet des stades de vie.
    Maturity,
}

impl Signal {
    pub(crate) const COUNT: usize = 7;

    pub(crate) const ALL: [Signal; Self::COUNT] = [
        Self::Light,
        Self::Warmth,
        Self::Water,
        Self::Food,
        Self::Crowding,
        Self::Energy,
        Self::Maturity,
    ];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Light => "Lumière",
            Self::Warmth => "Chaleur",
            Self::Water => "Eau",
            Self::Food => "Nourriture",
            Self::Crowding => "Densité",
            Self::Energy => "Énergie",
            Self::Maturity => "Maturité",
        }
    }

    pub(crate) fn index(self) -> usize {
        self as usize
    }
}
