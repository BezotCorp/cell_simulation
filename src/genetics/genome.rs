use crate::genetics::{Gene, Levels, Product, Signal, Signals};
use rand::RngExt;
use serde::{Deserialize, Serialize};

/// Nombre maximal d'unités de calcul d'un réseau neuronal.
pub(crate) const NEURAL_UNITS: usize = 8;
/// Nombre maximal de gènes : le génome peut grandir par duplication et apparition de gènes, pas
/// sans fin.
pub(crate) const MAX_GENES: usize = 24;
/// Mutations attendues par gène et par division, avant l'influence du milieu et de la réparation.
const MUTATION_PER_GENE: f64 = 0.05;
/// Amplitude d'une mutation de valeur de gène, en 1/1024.
const VALUE_STEP: i16 = 154;
/// Itérations d'expression : une cascade de régulation de trois niveaux se propage.
const EXPRESSION_STEPS: usize = 3;

/// Génotype héritable : liste variable de gènes et câblage du réseau neuronal. Il ne dit rien de la
/// forme ou du comportement : ils résultent de son expression dans un milieu, puis de lois physiques.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct Genome {
    genes: Vec<Gene>,
    wiring: [f32; NEURAL_UNITS],
    neural_bias: f32,
    sensor: u8,
}

impl Genome {
    pub(crate) fn new(
        genes: Vec<Gene>,
        wiring: [f32; NEURAL_UNITS],
        neural_bias: f32,
        sensor: u8,
    ) -> Self {
        Self {
            genes,
            wiring,
            neural_bias,
            sensor,
        }
    }

    pub(crate) fn genes(&self) -> &[Gene] {
        &self.genes
    }

    pub(crate) fn wiring(&self) -> &[f32; NEURAL_UNITS] {
        &self.wiring
    }

    pub(crate) fn neural_bias(&self) -> f32 {
        self.neural_bias
    }

    /// Capteur que lit le réseau : 0 température, 1 lumière, 2 nourriture, 3 énergie.
    pub(crate) fn sensor(&self) -> u8 {
        self.sensor
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.genes.is_empty()
            || self.genes.len() > MAX_GENES
            || self.sensor >= 4
            || !self.neural_bias.is_finite()
            || self.wiring.iter().any(|weight| !weight.is_finite())
        {
            return Err("Génome invalide.".into());
        }
        Ok(())
    }

    /// Expression du génome : chaque gène a une activité qui dépend des signaux du milieu et des
    /// produits déjà fabriqués, puis contribue à plusieurs produits. Trois passes laissent une
    /// cascade de régulation se propager. Le résultat est déterministe et reste dans `[0, 1]`.
    pub(crate) fn express(&self, signals: &Signals) -> Levels {
        let mut levels = [0.0_f32; Product::COUNT];
        for _ in 0..EXPRESSION_STEPS {
            let mut next = [0.0_f32; Product::COUNT];
            for gene in &self.genes {
                let activity = gene.activity(signals.values(), &levels);
                if activity > 0.0 {
                    gene.contribute(activity, &mut next);
                }
            }
            levels = next.map(|value| value.clamp(0.0, 1.0));
        }
        Levels::new(levels)
    }

    /// Activation du réseau neuronal à `units` unités pour une entrée donnée : une chaîne dont chaque
    /// unité multiplie le signal par son poids, écrasée par `tanh`. Sans unité, il n'y a pas de signal.
    pub(crate) fn neural_activation(&self, units: usize, input: f32) -> f32 {
        if units == 0 {
            return 0.0;
        }
        let signal = self.wiring[..units.min(NEURAL_UNITS)]
            .iter()
            .fold(input, |signal, weight| signal * weight);
        (signal + self.neural_bias).tanh()
    }

    /// Nombre moyen de mutations d'une division : il croît avec le nombre de gènes (un génome long
    /// offre plus de cibles) et avec l'exposition du milieu, et décroît avec la réparation.
    pub(crate) fn expected_mutations(&self, repair: f32, exposure: f32) -> f64 {
        MUTATION_PER_GENE * self.genes.len() as f64 * f64::from(exposure.max(0.0))
            / (1.0 + 4.0 * f64::from(repair.clamp(0.0, 1.0)))
    }

    /// Génome d'une fille : copie de celui de sa mère à laquelle s'appliquent des mutations en
    /// nombre aléatoire (moyenne `expected_mutations`). Chaque mutation est tirée sans tenir compte
    /// de son utilité : valeur d'un gène, réseau, duplication, perte ou apparition d'un gène.
    pub(crate) fn reproduce(&self, rng: &mut impl rand::Rng, repair: f32, exposure: f32) -> Self {
        let mut daughter = self.clone();
        let mut budget = self.expected_mutations(repair, exposure);
        while budget > 0.0 {
            if rng.random::<f64>() < budget.min(1.0) {
                daughter.mutate_once(rng);
            }
            budget -= 1.0;
        }
        daughter
    }

    fn mutate_once(&mut self, rng: &mut impl rand::Rng) {
        match rng.random_range(0..100) {
            0..=61 => {
                let index = rng.random_range(0..self.genes.len());
                self.genes[index].mutate_value(rng, VALUE_STEP);
            }
            62..=73 => match rng.random_range(0..NEURAL_UNITS + 2) {
                n if n < NEURAL_UNITS => self.wiring[n] += rng.random_range(-0.2..0.2),
                n if n == NEURAL_UNITS => self.neural_bias += rng.random_range(-1.0..1.0),
                _ => self.sensor = rng.random_range(0..4),
            },
            74..=83 => {
                if self.genes.len() < MAX_GENES {
                    let copy = self.genes[rng.random_range(0..self.genes.len())].clone();
                    self.genes.push(copy);
                }
            }
            84..=91 => {
                if self.genes.len() > 1 {
                    self.genes.remove(rng.random_range(0..self.genes.len()));
                }
            }
            _ => {
                if self.genes.len() < MAX_GENES {
                    self.genes.push(Gene::random(rng));
                }
            }
        }
    }

    /// Un résumé lisible pour l'inspection : (gènes, liens de régulation, produits touchés).
    pub(crate) fn complexity(&self) -> (usize, usize, usize) {
        (
            self.genes.len(),
            self.genes.iter().map(Gene::connections).sum(),
            self.genes.iter().map(Gene::reach).sum(),
        )
    }
}

/// Valeur du régulateur numéro `index` : utile aux outils d'inspection.
pub(crate) fn regulator_label(index: usize) -> &'static str {
    if index < Signal::COUNT {
        match Signal::ALL[index] {
            Signal::Light => "lumière",
            Signal::Warmth => "chaleur",
            Signal::Water => "eau",
            Signal::Food => "nourriture",
            Signal::Crowding => "densité",
            Signal::Energy => "énergie",
            Signal::Maturity => "maturité",
        }
    } else {
        Product::ALL[index - Signal::COUNT].label()
    }
}
