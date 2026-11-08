use crate::biology::Morphology;
use crate::genetics::{Levels, Product};

/// Niveaux de produits de l'ancêtre : son métabolisme de base en inclut déjà l'entretien, si bien
/// que le surcoût d'un organisme se mesure à l'écart avec eux.
const FOUNDER: [f32; Product::COUNT] = [
    0.5, 0.5, 0.75, 0.25, 1.0, 0.125, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0,
];
/// Entretien de chaque produit, en part du métabolisme par unité de quantité.
const UPKEEP: [f32; Product::COUNT] = [
    0.0, 0.0, 0.0, 0.10, 0.0, 0.0, 0.0, 0.04, 0.20, 0.20, 0.20, 0.06, 0.08, 0.05, 0.05,
];
/// Entretien d'une unité de calcul neuronale.
const NEURON_UPKEEP: f32 = 0.03;
/// Part de l'entretien qui ne dépend pas de la taille : tout organisme, même minuscule, doit
/// entretenir son génome et son appareil de base.
const FIXED_UPKEEP: f32 = 0.4;

/// Fonctionnement d'un organisme : cadence métabolique, capacités et dépenses. Elle suit
/// l'expression *courante* des gènes (donc le milieu) et se déduit de la morphologie par des lois
/// d'échelle : la dépense croît avec la taille comme la masse à la puissance 3/4 (au-dessus d'un
/// socle fixe), la captation de
/// lumière et de chaleur comme la surface, la nage décroît avec la traînée.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Physiology {
    metabolism_rate: f32,
    upkeep_scale: f32,
    machinery: f32,
    water_adaptation: f32,
    land_adaptation: f32,
    ingestion: f32,
    photosynthesis: f32,
    thermosynthesis: f32,
    predation: f32,
    motility: f32,
    adhesion: f32,
    swim_speed: f32,
    repair: f32,
    division_fraction: f32,
}

impl Physiology {
    /// `genome_upkeep` est l'entretien du matériel génétique au-delà de celui de l'ancêtre.
    pub(crate) fn express(levels: &Levels, morphology: &Morphology, genome_upkeep: f32) -> Self {
        let size = morphology.size();
        let metabolism_rate = 0.5 * 2.0_f32.powf(2.0 * levels.get(Product::Metabolism));
        let machinery: f32 = Product::ALL
            .iter()
            .map(|product| UPKEEP[product.index()] * (levels.get(*product) - FOUNDER[product.index()]))
            .sum::<f32>()
            + NEURON_UPKEEP * morphology.neural_units() as f32
            + genome_upkeep;
        let motor = levels.get(Product::Motor);
        let hydration = levels.get(Product::Hydration);
        Self {
            metabolism_rate,
            upkeep_scale: FIXED_UPKEEP + (1.0 - FIXED_UPKEEP) * size.powf(2.25),
            machinery,
            water_adaptation: hydration,
            land_adaptation: levels.get(Product::Cuticle),
            ingestion: levels.get(Product::Digestive),
            photosynthesis: levels.get(Product::Pigment) * size * size,
            thermosynthesis: levels.get(Product::Thermal) * size * size,
            predation: levels.get(Product::Raptorial) * size,
            motility: motor,
            adhesion: levels.get(Product::Adhesin),
            swim_speed: 0.12 * hydration * metabolism_rate * (0.3 + 0.7 * motor) / size.sqrt(),
            repair: levels.get(Product::Repair),
            division_fraction: 0.35 + 0.6 * levels.get(Product::Fecundity),
        }
    }

    /// Énergie dépensée par seconde et par unité de métabolisme, hors inadaptation au milieu.
    pub(crate) fn metabolism_rate(&self) -> f32 {
        self.metabolism_rate
    }

    /// Facteur d'échelle de l'entretien dû à la taille (masse à la puissance 3/4).
    pub(crate) fn upkeep_scale(&self) -> f32 {
        self.upkeep_scale
    }

    /// Surcoût (ou économie) des produits exprimés par rapport à l'ancêtre.
    pub(crate) fn machinery(&self) -> f32 {
        self.machinery
    }

    pub(crate) fn water_adaptation(&self) -> f32 {
        self.water_adaptation
    }

    pub(crate) fn land_adaptation(&self) -> f32 {
        self.land_adaptation
    }

    pub(crate) fn ingestion(&self) -> f32 {
        self.ingestion
    }

    pub(crate) fn photosynthesis(&self) -> f32 {
        self.photosynthesis
    }

    pub(crate) fn thermosynthesis(&self) -> f32 {
        self.thermosynthesis
    }

    pub(crate) fn predation(&self) -> f32 {
        self.predation
    }

    pub(crate) fn motility(&self) -> f32 {
        self.motility
    }

    pub(crate) fn adhesion(&self) -> f32 {
        self.adhesion
    }

    /// Vitesse de nage contre le courant, en sites par seconde.
    pub(crate) fn swim_speed(&self) -> f32 {
        self.swim_speed
    }

    /// Efficacité de la réparation de l'ADN.
    pub(crate) fn repair(&self) -> f32 {
        self.repair
    }

    /// Part de la réserve maximale à atteindre pour se diviser.
    pub(crate) fn division_fraction(&self) -> f32 {
        self.division_fraction
    }
}
